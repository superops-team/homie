use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Component, Path};

use homie_proto::remote_pty::{HelperProbe, PHASE_ONE_HELPER_CAPABILITIES};
use serde::Deserialize;

use crate::remote::bootstrap::{PackagedArtifact, RemoteTarget};

pub(crate) fn verify_required_helper_probe(
    artifact: &PackagedArtifact,
    probe: &HelperProbe,
) -> io::Result<()> {
    artifact.verify_probe(probe).map_err(io::Error::other)?;
    if let Some(missing) = PHASE_ONE_HELPER_CAPABILITIES
        .iter()
        .find(|capability| !probe.capabilities.contains(capability))
    {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "remote Helper is missing required capability {}",
                missing.wire_name()
            ),
        ));
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct ArtifactCatalog {
    pub(crate) artifacts: HashMap<RemoteTarget, PackagedArtifact>,
}

impl ArtifactCatalog {
    pub fn from_manifest(path: &Path) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        let manifest: ArtifactManifest = serde_json::from_slice(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "artifact manifest has no parent",
            )
        })?;
        let mut artifacts = HashMap::new();
        for entry in manifest.artifacts {
            let target =
                RemoteTarget::from_artifact_name(&entry.target).map_err(io::Error::other)?;
            let relative = Path::new(&entry.path);
            if relative.is_absolute()
                || relative
                    .components()
                    .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "artifact path must be normalized and relative to its manifest",
                ));
            }
            let artifact = PackagedArtifact {
                target,
                protocol_major: manifest.protocol_major,
                build_id: manifest.build_id.clone(),
                length: entry.length,
                sha256: entry.sha256,
                path: parent.join(relative),
            };
            artifact.verify().map_err(io::Error::other)?;
            if artifacts.insert(target, artifact).is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "artifact manifest contains a duplicate target",
                ));
            }
        }
        for target in RemoteTarget::ALL {
            if !artifacts.contains_key(&target) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "artifact manifest is missing required target {}",
                        target.artifact_name()
                    ),
                ));
            }
        }
        Ok(Self { artifacts })
    }

    /// Development and deterministic-test catalog containing the native
    /// Helper binary only. Release builds use the complete supported-target
    /// manifest.
    pub fn from_native_helper(path: &Path) -> io::Result<Self> {
        let output = std::process::Command::new(path)
            .args(["probe", "--format=json"])
            .output()?;
        if !output.status.success() {
            return Err(io::Error::other("native Helper probe failed"));
        }
        let probe: HelperProbe = serde_json::from_slice(&output.stdout)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let target = RemoteTarget::from_artifact_name(&probe.target).map_err(io::Error::other)?;
        let artifact = PackagedArtifact {
            target,
            protocol_major: probe.protocol.major,
            build_id: probe.build_id,
            length: fs::metadata(path)?.len(),
            sha256: probe.artifact_sha256,
            path: path.to_path_buf(),
        };
        artifact.verify().map_err(io::Error::other)?;
        Ok(Self {
            artifacts: HashMap::from([(target, artifact)]),
        })
    }

    pub(crate) fn artifact(&self, target: RemoteTarget) -> io::Result<&PackagedArtifact> {
        self.artifacts.get(&target).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                format!("no packaged Helper for {}", target.artifact_name()),
            )
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactManifest {
    protocol_major: u16,
    build_id: String,
    artifacts: Vec<ArtifactEntry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactEntry {
    target: String,
    path: String,
    length: u64,
    sha256: String,
}
