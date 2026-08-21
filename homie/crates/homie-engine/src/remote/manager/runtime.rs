use std::collections::HashMap;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use homie_proto::HostEntry;
use homie_proto::remote_pty::{
    DirectoryListRequest, DirectoryListResult, EnvironmentCaptureRequest, EnvironmentCaptureResult,
    GcResult, HelperProbe, LaunchRequest, LaunchResult, PersistenceCapability,
    PersistenceProbeAction, PersistenceProbeRequest, PersistenceProbeResult, ProtocolVersion,
    SessionInspection, SessionSelector,
};
use sha2::{Digest, Sha256};

use super::catalog::{ArtifactCatalog, verify_required_helper_probe};
use super::control_dir::{normalized_control_dir, validate_control_dir_if_present};
use super::util::{classify_persistence, parse_json_line, persistence_key, random_hex};
use super::{MAX_ARTIFACT_BYTES, MAX_RPC_OUTPUT, PROBE_TIMEOUT, RPC_TIMEOUT, UPLOAD_TIMEOUT};
use crate::remote::bootstrap::{PlatformProbe, RemoteInstallLayout, RemoteTarget};
use crate::remote::executor::{CommandOutput, ProcessExecutor, SshChannel};
use crate::remote::ssh::{HelperCommand, SshTransport};

#[derive(Clone, Debug)]
pub struct RemoteManager {
    executor: ProcessExecutor,
    artifacts: ArtifactCatalog,
    control_dir: PathBuf,
    persistence: Arc<Mutex<HashMap<String, PersistenceCapability>>>,
    /// Process-local target discovery cache. Every new remote action still
    /// probes the exact packaged Build ID before use; this cache only removes
    /// the redundant uname/platform round trip.
    current_helpers: Arc<Mutex<HashMap<String, CurrentHelper>>>,
}

#[derive(Clone, Debug)]
struct CurrentHelper {
    target: RemoteTarget,
    helper: InstalledHelper,
}

impl RemoteManager {
    pub fn new(
        executor: ProcessExecutor,
        artifacts: ArtifactCatalog,
        control_dir: PathBuf,
    ) -> io::Result<Self> {
        let control_dir = normalized_control_dir(&control_dir);
        validate_control_dir_if_present(&control_dir)?;
        fs::create_dir_all(&control_dir)?;
        // Recheck after creation to close the `/tmp` create race before chmod
        // or any OpenSSH socket creation follows an attacker-placed symlink.
        validate_control_dir_if_present(&control_dir)?;
        fs::set_permissions(&control_dir, fs::Permissions::from_mode(0o700))?;
        Ok(Self {
            executor,
            artifacts,
            control_dir,
            persistence: Arc::new(Mutex::new(HashMap::new())),
            current_helpers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Ensures this Engine build's exact Helper is installed and valid. A
    /// warm host needs one multiplexed `probe` round trip; a version change or
    /// failed probe falls through to the full platform-select/install path.
    pub fn ensure_helper(&self, host: &HostEntry) -> io::Result<InstalledHelper> {
        if let Some(helper) = self.verify_cached_current(host)? {
            return Ok(helper);
        }
        self.bootstrap_helper(host, false)
    }

    /// Forces the packaged bytes through upload, temporary verification and
    /// activation. Activation is content-addressed: an existing identical
    /// build is retained, while an incompatible occupant is never replaced.
    pub fn reinstall_helper(&self, host: &HostEntry) -> io::Result<InstalledHelper> {
        self.current_helpers
            .lock()
            .expect("current Helper cache")
            .remove(&host.ssh);
        self.persistence
            .lock()
            .expect("persistence cache")
            .remove(&persistence_key(host));
        self.bootstrap_helper(host, true)
    }

    /// Closes finite-lived OpenSSH multiplexers after the owning Engine has
    /// become fully idle. Live remote sessions never call this path.
    pub fn close_control_masters(&self) {
        let transports = self
            .current_helpers
            .lock()
            .expect("current Helper cache")
            .values()
            .map(|current| current.helper.transport.clone())
            .collect::<Vec<_>>();
        for transport in transports {
            let _ = self.executor.run(
                transport.control_exit(),
                Vec::new(),
                Duration::from_secs(2),
                4 * 1024,
            );
        }
    }

    fn verify_cached_current(&self, host: &HostEntry) -> io::Result<Option<InstalledHelper>> {
        let cached = self
            .current_helpers
            .lock()
            .expect("current Helper cache")
            .get(&host.ssh)
            .cloned();
        let Some(cached) = cached else {
            return Ok(None);
        };
        let artifact = self.artifacts.artifact(cached.target)?;
        artifact.verify().map_err(io::Error::other)?;
        if artifact.build_id != cached.helper.build_id {
            self.forget_current_helper(host);
            return Ok(None);
        }
        let output = self.executor.run(
            cached
                .helper
                .transport
                .helper_probe(&artifact.build_id)
                .map_err(io::Error::other)?,
            Vec::new(),
            PROBE_TIMEOUT,
            MAX_RPC_OUTPUT,
        )?;
        if !output.status.success() || output.stdout_truncated {
            self.forget_current_helper(host);
            return Ok(None);
        }
        let probe = match parse_json_line::<HelperProbe>(&output.stdout) {
            Ok(probe) => probe,
            Err(_) => {
                self.forget_current_helper(host);
                return Ok(None);
            }
        };
        if verify_required_helper_probe(artifact, &probe).is_err() || !probe.holder_available {
            self.forget_current_helper(host);
            return Ok(None);
        }
        let helper = InstalledHelper {
            build_id: artifact.build_id.clone(),
            protocol: probe.protocol,
            transport: cached.helper.transport,
        };
        self.remember_current_helper(host, cached.target, &helper);
        Ok(Some(helper))
    }

    fn bootstrap_helper(
        &self,
        host: &HostEntry,
        force_upload: bool,
    ) -> io::Result<InstalledHelper> {
        let transport = self.transport(host);
        let platform = self
            .executor
            .run(
                transport.platform_probe(),
                Vec::new(),
                PROBE_TIMEOUT,
                MAX_RPC_OUTPUT,
            )?
            .require_success("remote platform probe")?;
        if platform.stdout_truncated {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "remote platform probe output was truncated",
            ));
        }
        let platform = PlatformProbe::parse(&platform.stdout).map_err(io::Error::other)?;
        let artifact = self.artifacts.artifact(platform.target)?;
        artifact.verify().map_err(io::Error::other)?;

        if !force_upload {
            let final_probe = self.executor.run(
                transport
                    .helper_probe(&artifact.build_id)
                    .map_err(io::Error::other)?,
                Vec::new(),
                PROBE_TIMEOUT,
                MAX_RPC_OUTPUT,
            )?;
            if final_probe.status.success() {
                let probe = parse_json_line::<HelperProbe>(&final_probe.stdout)?;
                verify_required_helper_probe(artifact, &probe)?;
                if !probe.holder_available {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "remote Helper does not provide a Holder",
                    ));
                }
                let helper = InstalledHelper {
                    build_id: artifact.build_id.clone(),
                    protocol: probe.protocol,
                    transport,
                };
                self.remember_current_helper(host, platform.target, &helper);
                return Ok(helper);
            }
        }

        if artifact.length > MAX_ARTIFACT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "packaged Helper exceeds 64 MiB",
            ));
        }
        let nonce = random_hex(16)?;
        let layout =
            RemoteInstallLayout::new(&artifact.build_id, nonce).map_err(io::Error::other)?;
        let upload = fs::read(&artifact.path)?;
        let install: io::Result<HelperProbe> = (|| {
            self.executor
                .run(
                    transport.upload(&layout),
                    upload,
                    UPLOAD_TIMEOUT,
                    MAX_RPC_OUTPUT,
                )?
                .require_success("remote Helper upload")?;
            let temporary = self
                .executor
                .run(
                    transport.temporary_probe(&layout),
                    Vec::new(),
                    PROBE_TIMEOUT,
                    MAX_RPC_OUTPUT,
                )?
                .require_success("temporary Helper verification")?;
            let temporary = parse_json_line::<HelperProbe>(&temporary.stdout)?;
            verify_required_helper_probe(artifact, &temporary)?;
            let activated = self
                .executor
                .run(
                    transport.commit_upload(&layout),
                    Vec::new(),
                    PROBE_TIMEOUT,
                    MAX_RPC_OUTPUT,
                )?
                .require_success("Helper activation")?;
            let activated = parse_json_line::<HelperProbe>(&activated.stdout)?;
            verify_required_helper_probe(artifact, &activated)?;
            let final_probe = self
                .executor
                .run(
                    transport
                        .helper_probe(&artifact.build_id)
                        .map_err(io::Error::other)?,
                    Vec::new(),
                    PROBE_TIMEOUT,
                    MAX_RPC_OUTPUT,
                )?
                .require_success("activated Helper verification")?;
            let final_probe = parse_json_line::<HelperProbe>(&final_probe.stdout)?;
            verify_required_helper_probe(artifact, &final_probe)?;
            if !final_probe.holder_available {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "activated remote Helper does not provide a Holder",
                ));
            }
            Ok(final_probe)
        })();
        if install.is_err() {
            let _ = self.executor.run(
                transport.cleanup_upload(&layout),
                Vec::new(),
                PROBE_TIMEOUT,
                MAX_RPC_OUTPUT,
            );
        }
        let probe = install?;
        let helper = InstalledHelper {
            build_id: artifact.build_id.clone(),
            protocol: probe.protocol,
            transport,
        };
        self.remember_current_helper(host, platform.target, &helper);
        Ok(helper)
    }

    /// Lists one bounded directory level through the exact installed Helper.
    /// Version verification and the bounded listing each use one multiplexed
    /// round trip. A failed action clears the target cache and retries through
    /// the full verified bootstrap path once.
    pub fn list_directories(
        &self,
        host: &HostEntry,
        request: &DirectoryListRequest,
    ) -> io::Result<DirectoryListResult> {
        let helper = self.ensure_helper(host)?;
        match self.rpc(&helper, HelperCommand::Directories, request, RPC_TIMEOUT) {
            Ok(result) => Ok(result),
            Err(_) => {
                self.forget_current_helper(host);
                let helper = self.ensure_helper(host)?;
                self.rpc(&helper, HelperCommand::Directories, request, RPC_TIMEOUT)
            }
        }
    }

    /// Executes an Engine-owned fixed POSIX script over the host's multiplexed
    /// no-PTY SSH transport. Dynamic values must travel in `input`, never in
    /// the command text. This seam is reserved for bounded host orchestration
    /// such as Git inspection; Agent argv never passes through it.
    pub(crate) fn run_fixed_script(
        &self,
        host: &HostEntry,
        script: &'static str,
        input: Vec<u8>,
        timeout: Duration,
        output_limit: usize,
    ) -> io::Result<CommandOutput> {
        // Fixed orchestration scripts do not execute through the Helper, but
        // they are still remote actions. Gate them on the exact packaged
        // Helper so an app update synchronizes the host before doing work.
        self.ensure_helper(host)?;
        self.executor.run(
            self.transport(host).channel(script),
            input,
            timeout,
            output_limit,
        )
    }

    fn remember_current_helper(
        &self,
        host: &HostEntry,
        target: RemoteTarget,
        helper: &InstalledHelper,
    ) {
        self.current_helpers
            .lock()
            .expect("current Helper cache")
            .insert(
                host.ssh.clone(),
                CurrentHelper {
                    target,
                    helper: helper.clone(),
                },
            );
    }

    fn forget_current_helper(&self, host: &HostEntry) {
        self.current_helpers
            .lock()
            .expect("current Helper cache")
            .remove(&host.ssh);
    }

    /// Reopens the exact version referenced by a live session. Old builds are
    /// intentionally allowed to coexist with the current packaged artifact.
    pub fn existing_helper(
        &self,
        host: &HostEntry,
        build_id: &str,
        protocol: ProtocolVersion,
    ) -> io::Result<InstalledHelper> {
        let transport = self.transport(host);
        let output = self
            .executor
            .run(
                transport.helper_probe(build_id).map_err(io::Error::other)?,
                Vec::new(),
                PROBE_TIMEOUT,
                MAX_RPC_OUTPUT,
            )?
            .require_success("existing remote Helper verification")?;
        let probe = parse_json_line::<HelperProbe>(&output.stdout)?;
        if probe.build_id != build_id
            || probe.protocol.major != protocol.major
            || !probe.holder_available
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "existing remote Helper identity or protocol does not match",
            ));
        }
        Ok(InstalledHelper {
            build_id: build_id.to_string(),
            protocol: probe.protocol,
            transport,
        })
    }

    pub fn capture_environment(
        &self,
        helper: &InstalledHelper,
        request: &EnvironmentCaptureRequest,
    ) -> io::Result<EnvironmentCaptureResult> {
        self.rpc(
            helper,
            HelperCommand::Environment,
            request,
            Duration::from_secs(12),
        )
    }

    /// Tests survival across two distinct SSH command channels. The result is
    /// cached only for this Engine lifetime and never inferred from `setsid`.
    pub fn probe_persistence(
        &self,
        host: &HostEntry,
        helper: &InstalledHelper,
    ) -> io::Result<PersistenceCapability> {
        let key = persistence_key(host);
        if let Some(capability) = self
            .persistence
            .lock()
            .expect("persistence cache")
            .get(&key)
            .copied()
        {
            return Ok(capability);
        }

        let native = self.probe_persistence_mode(helper, PersistenceProbeAction::BeginNative)?;
        let supervised = if native {
            false
        } else {
            self.probe_persistence_mode(helper, PersistenceProbeAction::BeginSupervisor)?
        };
        let capability = classify_persistence(native, supervised);
        self.persistence
            .lock()
            .expect("persistence cache")
            .insert(key, capability);
        Ok(capability)
    }

    fn probe_persistence_mode(
        &self,
        helper: &InstalledHelper,
        action: PersistenceProbeAction,
    ) -> io::Result<bool> {
        let nonce = random_hex(16)?;
        let begin: PersistenceProbeResult = self.rpc(
            helper,
            HelperCommand::Persistence,
            &PersistenceProbeRequest {
                nonce: nonce.clone(),
                action,
            },
            RPC_TIMEOUT,
        )?;
        let checked = if begin.alive {
            // The begin SSH command has exited before `rpc` returns. This
            // margin lets login-scope cleanup finish before a separate
            // channel checks the witness.
            std::thread::sleep(Duration::from_millis(100));
            self.rpc::<_, PersistenceProbeResult>(
                helper,
                HelperCommand::Persistence,
                &PersistenceProbeRequest {
                    nonce: nonce.clone(),
                    action: PersistenceProbeAction::Check,
                },
                RPC_TIMEOUT,
            )?
        } else {
            PersistenceProbeResult { alive: false }
        };
        let cleanup = self.rpc::<_, PersistenceProbeResult>(
            helper,
            HelperCommand::Persistence,
            &PersistenceProbeRequest {
                nonce,
                action: PersistenceProbeAction::Cleanup,
            },
            RPC_TIMEOUT,
        );
        if let Err(error) = cleanup {
            return Err(io::Error::other(format!(
                "persistence probe cleanup failed: {error}"
            )));
        }
        Ok(checked.alive)
    }

    pub fn launch(
        &self,
        helper: &InstalledHelper,
        request: &LaunchRequest,
    ) -> io::Result<LaunchResult> {
        match self.rpc(helper, HelperCommand::Launch, request, UPLOAD_TIMEOUT) {
            Ok(result) => Ok(result),
            Err(first) => {
                // Launch is authenticated and idempotent for a
                // session/token pair. A channel may fail after the Holder was
                // created but before its response arrived; one fresh SSH
                // command recovers that exact incarnation without spawning a
                // duplicate or leaking an unknown Holder.
                std::thread::sleep(Duration::from_millis(50));
                self.rpc(helper, HelperCommand::Launch, request, UPLOAD_TIMEOUT)
                    .map_err(|second| {
                        io::Error::new(
                            second.kind(),
                            format!(
                                "remote launch failed before and after idempotent retry: {first}; {second}"
                            ),
                        )
                    })
            }
        }
    }

    pub fn inspect(
        &self,
        helper: &InstalledHelper,
        selector: &SessionSelector,
    ) -> io::Result<SessionInspection> {
        self.rpc(helper, HelperCommand::Inspect, selector, RPC_TIMEOUT)
    }

    pub fn list(&self, helper: &InstalledHelper) -> io::Result<Vec<SessionInspection>> {
        self.rpc_empty(helper, HelperCommand::List, RPC_TIMEOUT)
    }

    pub fn kill(
        &self,
        helper: &InstalledHelper,
        selector: &SessionSelector,
    ) -> io::Result<SessionInspection> {
        self.rpc(helper, HelperCommand::Kill, selector, RPC_TIMEOUT)
    }

    pub fn gc(&self, helper: &InstalledHelper) -> io::Result<GcResult> {
        self.rpc_empty(helper, HelperCommand::Gc, RPC_TIMEOUT)
    }

    pub fn attach(&self, helper: &InstalledHelper) -> io::Result<SshChannel> {
        self.executor.open(
            helper
                .transport
                .helper_command(&helper.build_id, HelperCommand::Attach)
                .map_err(io::Error::other)?,
        )
    }

    fn rpc<T: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        helper: &InstalledHelper,
        command: HelperCommand,
        request: &T,
        timeout: Duration,
    ) -> io::Result<R> {
        let input = serde_json::to_vec(request)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        self.rpc_input(helper, command, input, timeout)
    }

    fn rpc_empty<R: serde::de::DeserializeOwned>(
        &self,
        helper: &InstalledHelper,
        command: HelperCommand,
        timeout: Duration,
    ) -> io::Result<R> {
        self.rpc_input(helper, command, Vec::new(), timeout)
    }

    fn rpc_input<R: serde::de::DeserializeOwned>(
        &self,
        helper: &InstalledHelper,
        command: HelperCommand,
        input: Vec<u8>,
        timeout: Duration,
    ) -> io::Result<R> {
        let output = self
            .executor
            .run(
                helper
                    .transport
                    .helper_command(&helper.build_id, command)
                    .map_err(io::Error::other)?,
                input,
                timeout,
                MAX_RPC_OUTPUT,
            )?
            .require_success("remote Helper RPC")?;
        if output.stdout_truncated {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "remote Helper response was truncated",
            ));
        }
        parse_json_line(&output.stdout)
    }

    fn transport(&self, host: &HostEntry) -> SshTransport {
        let digest = Sha256::digest(host.ssh.as_bytes());
        let name = digest[..12]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        SshTransport::new(host, self.control_dir.join(name))
            .with_executable(self.executor.ssh_executable().to_os_string())
    }
}

#[derive(Clone, Debug)]
pub struct InstalledHelper {
    pub build_id: String,
    pub protocol: ProtocolVersion,
    pub transport: SshTransport,
}
