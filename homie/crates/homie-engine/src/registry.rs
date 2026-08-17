//! The set of live sessions, and their persisted records.
//!
//! The registry is what a control channel talks to: spawn, list, write, kill.
//! It also owns the additive `{ version, projects, sessions }` persistence
//! envelope. Unknown project fields survive a read/write cycle.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use homie_proto::{DateMillis, ExitInfo, ExitReason, SessionRecord, SessionStatus, TitleSource};

use crate::detect::ManifestEngine;
use crate::holder::{HolderClient, HolderManagerPaths, HolderPaths};
use crate::session::{HolderConfig, RemoteAdoptSpec, Session, SessionSpec, SessionView};
mod flusher;
mod migrate;
mod persisted;
mod store;

pub use flusher::spawn_persist_flusher;
pub use migrate::{SplitMigrationReport, migrate_envelope_to_split};
pub use persisted::PersistedState;
pub use store::{JsonEnvelopeStore, PersistenceStore, SplitJsonStore};

use persisted::{fold_session_view, normalize_agent_title};

pub struct Registry {
    engine: Arc<ManifestEngine>,
    sessions: HashMap<String, Session>,
    /// Records for sessions that are no longer live but still listed.
    records: HashMap<String, SessionRecord>,
    /// Project records are kept as additive JSON so fields outside the
    /// Engine's minimal id/root/name model survive persistence.
    projects: Vec<serde_json::Value>,
    /// Sessions the user closed, newest last — the "reopen closed tab" stack.
    recently_closed: Vec<SessionRecord>,
    state_file: PathBuf,
    /// Trailing-edge persistence: a mutation inside the debounce window marks
    /// dirty instead of rewriting the whole file (mark-seen fires on every
    /// tab switch), and the flusher or the next persist call writes it out.
    dirty: bool,
    last_persist: Option<std::time::Instant>,
}

impl Registry {
    pub fn new(engine: Arc<ManifestEngine>, state_file: impl Into<PathBuf>) -> Self {
        Self {
            engine,
            sessions: HashMap::new(),
            records: HashMap::new(),
            projects: Vec::new(),
            recently_closed: Vec::new(),
            state_file: state_file.into(),
            dirty: false,
            last_persist: None,
        }
    }

    /// Adds (or replaces) a record without a live session — restores,
    /// imports, and tests use this; live sessions come from [`spawn`].
    ///
    /// [`spawn`]: Registry::spawn
    pub fn insert_record(&mut self, record: SessionRecord) {
        self.records.insert(record.id.0.clone(), record);
    }

    /// Starts a session and takes ownership of it.
    pub fn spawn(&mut self, spec: SessionSpec, record: SessionRecord) -> std::io::Result<String> {
        let id = spec.id.clone();
        let session = Session::spawn(spec, Arc::clone(&self.engine))?;
        self.records.insert(id.clone(), record);
        self.sessions.insert(id.clone(), session);
        Ok(id)
    }

    pub fn adopt_remote(
        &mut self,
        spec: SessionSpec,
        remote: RemoteAdoptSpec,
    ) -> std::io::Result<String> {
        let id = spec.id.clone();
        if !self.records.contains_key(&id) {
            return Err(not_found(&id));
        }
        let initial_status = self
            .records
            .get(&id)
            .filter(|record| !matches!(record.status, SessionStatus::Exited(_)))
            .map(|record| (record.status.clone(), record.needs_input.clone()));
        let session = Session::adopt_remote_with_status(
            spec,
            remote,
            Arc::clone(&self.engine),
            initial_status,
        )?;
        self.sessions.insert(id.clone(), session);
        Ok(id)
    }

    /// Adopts every still-live holder-owned session found under
    /// `holder.holders_dir` that has a persisted record. Call after [`load`]:
    /// this is what makes sessions survive a daemon restart — or the switch
    /// from the reference implementation to this one.
    ///
    /// Returns the ids adopted. Local sessions whose holder did not survive
    /// are reconciled to `Exited` by [`reap_orphans`], so a record can never
    /// go on claiming a status only a live holder could report.
    ///
    /// [`load`]: Registry::load
    /// [`reap_orphans`]: Registry::reap_orphans
    pub fn restore(&mut self, holder: &HolderConfig, logs_dir: &Path) -> Vec<String> {
        let adopted = self.adopt_live_holders(holder, logs_dir);
        self.reap_orphans();
        adopted
    }

    /// Marks every local record that no live session backs as exited.
    ///
    /// A record's status is a live holder's claim about a process. When the
    /// machine dies, the holders die with it and nothing is left to retract
    /// the claim — so `load` hands back records still saying `Working`, and
    /// every consumer reads them as running: the app dials a socket that will
    /// never answer and retries "Reconnecting terminal…" forever, offering no
    /// Resume because the conversation still looks live. Retract the claim
    /// here, once, on the only pass that knows which holders answered.
    ///
    /// Remote (`host`-bound) sessions are none of this pass's business: they
    /// live in tmux on another machine and outlive both this daemon and this
    /// Mac, so their records stay untouched.
    fn reap_orphans(&mut self) {
        let orphaned: Vec<String> = self
            .records
            .values()
            .filter(|record| record.host.is_none())
            .filter(|record| !matches!(record.status, SessionStatus::Exited(_)))
            .filter(|record| !self.sessions.contains_key(&record.id.0))
            .map(|record| record.id.0.clone())
            .collect();
        if orphaned.is_empty() {
            return;
        }
        for id in &orphaned {
            if let Some(record) = self.records.get_mut(id) {
                record.status = SessionStatus::Exited(ExitInfo {
                    reason: ExitReason::DaemonRestart,
                    code: None,
                    signal: None,
                });
                record.needs_input = None;
            }
        }
        let _ = self.persist();
    }

    /// Adopts the holders that are still answering. See [`restore`].
    ///
    /// [`restore`]: Registry::restore
    fn adopt_live_holders(&mut self, holder: &HolderConfig, logs_dir: &Path) -> Vec<String> {
        let holders_dir = HolderPaths::new(&holder.holders_dir, "probe").directory;
        let Ok(entries) = std::fs::read_dir(&holders_dir) else {
            return Vec::new();
        };
        let holder_session_ids: Vec<String> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "sock")
                    && !HolderManagerPaths::is_manager_socket(path)
            })
            .filter_map(|path| {
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(str::to_string)
            })
            .collect();

        let mut adopted = Vec::new();
        for session_id in holder_session_ids {
            let Some(record) = self.records.get(&session_id) else {
                continue; // a holder without a record is not ours to run
            };
            if self.sessions.contains_key(&session_id) {
                continue;
            }
            let paths = HolderPaths::new(&holder.holders_dir, &session_id);
            let client = HolderClient::new(paths.socket());
            let Ok(stat) = client.stat() else { continue };
            if !stat.alive {
                continue;
            }
            let manifest_id = record.kind.id().to_string();
            let record_status = record.status.clone();
            let record_needs_input = record.needs_input.clone();
            let record_hibernated = record.hibernation.is_some();
            let spec = SessionSpec {
                id: session_id.clone(),
                // The holder owns the real spec; this one only shapes the
                // emulator until stat's dimensions overwrite it in `adopt`.
                pty: crate::pty::PtySpec::new(Vec::new(), record.cwd.clone()),
                manifest_id: manifest_id.clone(),
                authority: crate::session::authority_for(&manifest_id, &self.engine),
                logs_dir: logs_dir.to_path_buf(),
                holder: Some(holder.clone()),
                remote: None,
                defer_launch: false,
            };
            let seeded = (!matches!(record_status, SessionStatus::Exited(_)))
                .then(|| (record_status.clone(), record_needs_input.clone()));
            let was_hibernated = record_hibernated;
            match Session::adopt_with_status(spec, holder, &stat, Arc::clone(&self.engine), seeded)
            {
                Ok(session) => {
                    if was_hibernated {
                        let _ = session.set_hibernated(true);
                    }
                    self.sessions.insert(session_id.clone(), session);
                    adopted.push(session_id);
                }
                Err(_) => continue,
            }
        }
        adopted
    }

    /// The manifest engine these sessions were started with.
    pub fn engine(&self) -> Arc<ManifestEngine> {
        Arc::clone(&self.engine)
    }

    pub fn get(&self, id: &str) -> Option<&Session> {
        self.sessions.get(id)
    }

    pub fn views(&self) -> Vec<SessionView> {
        let mut views: Vec<_> = self.sessions.values().map(Session::view).collect();
        views.sort_by(|a, b| a.id.cmp(&b.id));
        views
    }

    /// Session records with live status and a provisional Agent-provided PTY
    /// title folded in. Structured titles persisted by hooks remain
    /// authoritative; the PTY fallback exists for Agents without hooks.
    pub fn records(&self) -> Vec<SessionRecord> {
        let mut records: Vec<SessionRecord> = self.records.values().cloned().collect();
        for record in &mut records {
            self.fold_live(record);
        }
        records.sort_by(|a, b| a.id.0.cmp(&b.id.0));
        records
    }

    /// One record with live status folded in, without cloning the whole table.
    pub fn record(&self, id: &str) -> Option<SessionRecord> {
        let mut record = self.records.get(id)?.clone();
        self.fold_live(&mut record);
        Some(record)
    }

    /// Folds what only the live session knows into a stored record: its real
    /// status and Agent-provided title, and the resumability that follows
    /// from that status.
    fn fold_live(&self, record: &mut SessionRecord) {
        if let Some(session) = self.sessions.get(&record.id.0) {
            fold_session_view(record, &session.view());
        }
        // `Live` only records that the agent named its conversation while it
        // was running. Once the session is gone the question every Resume
        // affordance asks is a different one — can that conversation be
        // re-entered — so answer it here rather than leaving a stale `Live`
        // that reads as "not resumable" to each of them.
        if matches!(record.status, SessionStatus::Exited(_))
            && record.resumability == homie_proto::Resumability::Live
        {
            record.resumability = if self.can_reenter(record) {
                homie_proto::Resumability::Resumable
            } else {
                homie_proto::Resumability::NotResumable
            };
        }
    }

    /// Whether this record's agent can be relaunched back into its own
    /// conversation — a known conversation id plus a manifest that declares
    /// how to resume one.
    fn can_reenter(&self, record: &SessionRecord) -> bool {
        let Some(agent_session_id) = record.agent_session_id.as_deref() else {
            return false;
        };
        self.engine
            .manifest(record.kind.id())
            .and_then(|manifest| manifest.agent.as_ref())
            .and_then(|agent| agent.resume_args(Some(agent_session_id)))
            .is_some()
    }

    /// Diffs live sessions' state versions against `published` (updating it in
    /// place) and returns folded records for just the sessions that changed.
    /// The steady-state cost — the events watcher polls this several times a
    /// second — is one integer compare per live session: no clones, no
    /// serialization.
    pub fn changed_since(
        &mut self,
        published: &mut HashMap<String, u64>,
    ) -> Vec<(String, SessionRecord)> {
        published.retain(|id, _| self.sessions.contains_key(id));
        let mut changed = Vec::new();
        let changed_views = self
            .sessions
            .iter()
            .filter_map(|(id, session)| {
                let version = session.state_version();
                (published.get(id) != Some(&version)).then(|| (id.clone(), version, session.view()))
            })
            .collect::<Vec<_>>();
        let mut title_changed = false;
        for (id, version, view) in changed_views {
            published.insert(id.clone(), version);
            if let Some(record) = self.records.get_mut(&id) {
                let previous_title = (record.title.clone(), record.title_source);
                fold_session_view(record, &view);
                let record_title_changed =
                    previous_title != (record.title.clone(), record.title_source);
                if record_title_changed {
                    record.updated_at = DateMillis::from(std::time::SystemTime::now());
                    title_changed = true;
                }
                changed.push((id, record.clone()));
            }
        }
        if title_changed {
            self.dirty = true;
        }
        changed
    }

    /// Ends a session but keeps its record, which is what archiving means here.
    pub fn terminate(
        &mut self,
        id: &str,
        grace: std::time::Duration,
    ) -> std::io::Result<Option<crate::pty::Exit>> {
        let Some(mut session) = self.sessions.remove(id) else {
            return Ok(None);
        };
        let exit = session.terminate(grace)?;
        if let Some(record) = self.records.get_mut(id) {
            record.status = SessionStatus::Exited(homie_proto::ExitInfo {
                reason: match exit {
                    crate::pty::Exit::Signal(_) => homie_proto::ExitReason::Signaled,
                    crate::pty::Exit::Code(_) => homie_proto::ExitReason::Exited,
                },
                code: match exit {
                    crate::pty::Exit::Code(code) => Some(code),
                    crate::pty::Exit::Signal(_) => None,
                },
                signal: match exit {
                    crate::pty::Exit::Signal(signal) => Some(signal),
                    crate::pty::Exit::Code(_) => None,
                },
            });
        }
        Ok(Some(exit))
    }

    /// Drops a record entirely — the session is gone and not coming back.
    pub fn forget(&mut self, id: &str) {
        self.sessions.remove(id);
        self.records.remove(id);
    }

    /// Ends the session (if live), deletes its record AND its output log.
    /// This is the user closing a tab for good, not archiving.
    pub fn remove(&mut self, id: &str, logs_dir: &Path) -> std::io::Result<()> {
        if self.sessions.contains_key(id) {
            let _ = self.terminate(id, std::time::Duration::from_millis(500));
        }
        let Some(record) = self.records.remove(id) else {
            return Err(not_found(id));
        };
        self.recently_closed.push(record);
        if self.recently_closed.len() > 10 {
            self.recently_closed.remove(0);
        }
        self.sessions.remove(id);
        let _ = std::fs::remove_file(logs_dir.join(format!("{id}.bin")));
        Ok(())
    }

    /// Pops the most recently closed session whose folder still exists (a
    /// remote cwd can't be checked locally, so it always qualifies) and
    /// re-lists it. The caller drives the resume path from there.
    pub fn reopen_last_closed(&mut self) -> Option<SessionRecord> {
        while let Some(record) = self.recently_closed.pop() {
            if record.host.is_none() && !Path::new(&record.cwd).exists() {
                continue; // the folder is gone; try the next candidate
            }
            self.records.insert(record.id.0.clone(), record.clone());
            return Some(record);
        }
        None
    }

    /// Respawns a session under an EXISTING record — the resume path.
    pub fn respawn(&mut self, spec: SessionSpec) -> std::io::Result<()> {
        let id = spec.id.clone();
        if !self.records.contains_key(&id) {
            return Err(not_found(&id));
        }
        let session = Session::spawn(spec, Arc::clone(&self.engine))?;
        self.sessions.insert(id.clone(), session);
        let record = self.records.get_mut(&id).expect("checked above");
        record.status = SessionStatus::Starting;
        record.needs_input = None;
        record.updated_at = DateMillis::from(std::time::SystemTime::now());
        Ok(())
    }

    /// SIGCONTs a hibernated session's tree, flushes any input queued while
    /// it was frozen, and clears the record. A no-op for sessions whose
    /// metadata and in-memory state both say awake, so hot input paths can
    /// call it unconditionally.
    pub fn wake_session(&mut self, id: &str) -> std::io::Result<()> {
        let hibernated = self
            .records
            .get(id)
            .is_some_and(|record| record.hibernation.is_some())
            || self.sessions.get(id).is_some_and(Session::is_hibernated);
        if !hibernated {
            return Ok(());
        }
        self.ensure_session_awake(id)
    }

    /// Reconciles a user-visible session with the OS process state even when
    /// its hibernation metadata is stale or missing. Fresh data-channel
    /// attaches call this once: SIGCONT is harmless for a running tree, and
    /// it repairs the otherwise permanent "live record, stopped process"
    /// state without putting a process-tree walk on every keystroke.
    pub fn ensure_session_awake(&mut self, id: &str) -> std::io::Result<()> {
        let known_hibernated = self
            .records
            .get(id)
            .is_some_and(|record| record.hibernation.is_some())
            || self.sessions.get(id).is_some_and(Session::is_hibernated);
        if let Some(session) = self.sessions.get(id) {
            session.signal_tree(libc::SIGCONT)?;
            // Flush AFTER the CONT so the tree is drinking again.
            let _ = session.set_hibernated(false);
        }
        if known_hibernated {
            self.set_hibernation(id, None);
        }
        Ok(())
    }

    /// Folds identity a hook payload carried into the record: the agent-side
    /// conversation id (what makes resume possible), the live transcript path
    /// (it MOVES when the agent enters a worktree), a first-prompt fallback,
    /// and Claude's generated `ai-title` when it becomes available. Returns
    /// whether anything changed.
    pub fn apply_hook_metadata(&mut self, id: &str, meta: &crate::hooks::HookMetadata) -> bool {
        let generated_title = self.records.get(id).and_then(|record| {
            let accepts_generated_title = record.kind == homie_proto::AgentKind::CLAUDE_CODE
                && matches!(
                    record.title_source,
                    TitleSource::Placeholder | TitleSource::FirstPrompt | TitleSource::Unknown
                );
            accepts_generated_title
                .then(|| {
                    meta.transcript_path
                        .as_deref()
                        .or(record.transcript_path.as_deref())
                })
                .flatten()
                .and_then(|path| crate::history::latest_claude_ai_title(Path::new(path)))
                .and_then(|title| normalize_agent_title(&title))
        });
        let Some(record) = self.records.get_mut(id) else {
            return false;
        };
        let mut changed = false;
        if let Some(agent_id) = &meta.agent_session_id
            && record.agent_session_id.as_ref() != Some(agent_id)
        {
            record.agent_session_id = Some(agent_id.clone());
            record.resumability = homie_proto::Resumability::Live;
            changed = true;
        }
        if let Some(transcript) = &meta.transcript_path
            && record.transcript_path.as_ref() != Some(transcript)
        {
            record.transcript_path = Some(transcript.clone());
            changed = true;
        }
        if let Some(title) = &meta.first_prompt_title
            && record.title_source == TitleSource::Placeholder
        {
            record.title = title.clone();
            record.title_source = TitleSource::FirstPrompt;
            changed = true;
        }
        if let Some(title) = generated_title
            && (record.title != title || record.title_source != TitleSource::AgentProvided)
        {
            record.title = title;
            record.title_source = TitleSource::AgentProvided;
            changed = true;
        }
        if changed {
            record.updated_at = DateMillis::from(std::time::SystemTime::now());
        }
        changed
    }

    /// SIGSTOPs a session's whole tree and records it as hibernated. The PTY
    /// and holder stay alive; wake is one SIGCONT away.
    pub fn hibernate(
        &mut self,
        id: &str,
        reason: homie_proto::HibernationReason,
    ) -> std::io::Result<()> {
        let tree = {
            let session = self.sessions.get(id).ok_or_else(|| not_found(id))?;
            let tree = session.signal_tree(libc::SIGSTOP)?;
            let _ = session.set_hibernated(true);
            tree
        };
        self.set_hibernation(
            id,
            Some(homie_proto::HibernationInfo {
                since: std::time::SystemTime::now().into(),
                reason,
                tree_pids: tree.iter().map(|(pid, _)| *pid).collect(),
                tree_start_times: Some(tree.into_iter().collect()),
            }),
        );
        Ok(())
    }

    /// Folds a governor sample into the record; returns the event to publish
    /// when anything actually changed (carrying only the changed facets, as
    /// the reference implementation does).
    pub fn apply_resource_sample(
        &mut self,
        id: &str,
        memory_bytes: Option<u64>,
        ports: Option<Vec<homie_proto::PortInfo>>,
        artifacts: Option<Vec<homie_proto::SessionArtifact>>,
    ) -> Option<homie_proto::SessionResourcesEvent> {
        let record = self.records.get_mut(id)?;
        let mut memory_changed = false;
        let mut ports_changed = false;
        let mut artifacts_changed = false;
        if let Some(memory) = memory_bytes
            && record.memory_bytes != Some(memory)
        {
            record.memory_bytes = Some(memory);
            memory_changed = true;
        }
        if let Some(ports) = ports
            && record.listening_ports.as_deref().unwrap_or_default() != ports
        {
            record.listening_ports = Some(ports);
            ports_changed = true;
        }
        if let Some(artifacts) = artifacts
            && record.artifacts.as_deref().unwrap_or_default() != artifacts
        {
            record.artifacts = Some(artifacts);
            artifacts_changed = true;
        }
        if !(memory_changed || ports_changed || artifacts_changed) {
            return None;
        }
        Some(homie_proto::SessionResourcesEvent {
            id: record.id.clone(),
            memory_bytes: memory_changed.then_some(record.memory_bytes).flatten(),
            listening_ports: if ports_changed {
                record.listening_ports.clone()
            } else {
                None
            },
            artifacts: if artifacts_changed {
                record.artifacts.clone()
            } else {
                None
            },
        })
    }

    /// Replaces the record's PR statuses when they materially changed.
    /// Returns whether they did.
    pub fn apply_pull_request_statuses(
        &mut self,
        id: &str,
        statuses: Vec<homie_proto::PullRequestStatus>,
    ) -> bool {
        let Some(record) = self.records.get_mut(id) else {
            return false;
        };
        let current = record.pull_requests.as_deref().unwrap_or_default();
        let materially_same = current.len() == statuses.len()
            && current.iter().zip(&statuses).all(|(a, b)| {
                // fetched_at always moves; compare everything else.
                let mut b_pinned = b.clone();
                b_pinned.fetched_at = a.fetched_at;
                *a == b_pinned
            });
        if materially_same {
            return false;
        }
        record.pull_requests = (!statuses.is_empty()).then_some(statuses);
        record.updated_at = DateMillis::from(std::time::SystemTime::now());
        true
    }

    /// Applies an arbitrary record mutation (migrate's in-place rewrite).
    pub fn update_record(&mut self, id: &str, mutate: impl FnOnce(&mut SessionRecord)) {
        if let Some(record) = self.records.get_mut(id) {
            mutate(record);
            record.updated_at = DateMillis::from(std::time::SystemTime::now());
        }
    }

    pub fn set_hibernation(&mut self, id: &str, info: Option<homie_proto::HibernationInfo>) {
        if let Some(record) = self.records.get_mut(id) {
            record.hibernation = info;
            record.updated_at = DateMillis::from(std::time::SystemTime::now());
        }
    }

    /// Upserts a local project by its deterministic root-derived id.
    pub fn add_project(&mut self, root: &str) -> serde_json::Value {
        self.ensure_session_project(root, None)
    }

    /// Ensures every Session has a concrete first-level Project record. The
    /// host remains an execution property of Sessions; the project id carries
    /// the location namespace and prevents cross-host path collisions.
    pub fn ensure_session_project(&mut self, root: &str, host: Option<&str>) -> serde_json::Value {
        let id = session_project_id(root, host).0;
        if let Some(existing) = self
            .projects
            .iter()
            .find(|project| project.get("id").and_then(|value| value.as_str()) == Some(&id))
        {
            return existing.clone();
        }
        let name = Path::new(root)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| root.to_string());
        let project = serde_json::json!({ "id": id, "root": root, "name": name });
        self.projects.push(project.clone());
        project
    }

    pub fn rename(&mut self, id: &str, title: &str) -> std::io::Result<()> {
        let record = self.records.get_mut(id).ok_or_else(|| not_found(id))?;
        record.title = title.to_string();
        record.title_source = TitleSource::UserRename;
        record.updated_at = DateMillis::from(std::time::SystemTime::now());
        Ok(())
    }

    pub fn mark_seen(&mut self, id: &str) -> std::io::Result<()> {
        let record = self.records.get_mut(id).ok_or_else(|| not_found(id))?;
        record.last_seen_at = Some(DateMillis::from(std::time::SystemTime::now()));
        Ok(())
    }

    /// Ends the session but keeps its record on the shelf: kill-tree,
    /// keep-record, stamp `archivedAt`.
    pub fn archive(&mut self, id: &str) -> std::io::Result<()> {
        if !self.records.contains_key(id) {
            return Err(not_found(id));
        }
        if self.sessions.contains_key(id) {
            let _ = self.terminate(id, std::time::Duration::from_millis(500));
        }
        let record = self.records.get_mut(id).expect("checked above");
        record.archived_at = Some(DateMillis::from(std::time::SystemTime::now()));
        if !matches!(record.status, SessionStatus::Exited(_)) {
            record.status = SessionStatus::Exited(homie_proto::ExitInfo {
                reason: homie_proto::ExitReason::Archived,
                code: None,
                signal: None,
            });
        }
        record.needs_input = None;
        Ok(())
    }

    pub fn unarchive(&mut self, id: &str) -> std::io::Result<()> {
        let record = self.records.get_mut(id).ok_or_else(|| not_found(id))?;
        if record.archived_at.is_none() {
            return Ok(());
        }
        record.archived_at = None;
        record.updated_at = DateMillis::from(std::time::SystemTime::now());
        Ok(())
    }

    /// Agent-side conversation ids already represented here, so a history
    /// scan can exclude conversations that are live sessions.
    pub fn tracked_agent_session_ids(&self) -> Vec<String> {
        self.records
            .values()
            .filter_map(|record| record.agent_session_id.clone())
            .collect()
    }

    /// The additive project list exposed through the control protocol.
    pub fn projects_raw(&self) -> &[serde_json::Value] {
        &self.projects
    }

    pub fn live_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn state_file(&self) -> &Path {
        &self.state_file
    }
}

fn not_found(id: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::NotFound, format!("no session {id}"))
}

/// Stable FNV-1a-shaped hash over a project location, truncated to 48 bits.
/// The historical multiplier is intentionally retained so existing local
/// project ids remain stable.
fn project_id(root: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in root.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_1000_0000_01B3);
    }
    format!("p_{:012x}", hash & 0xFFFF_FFFF_FFFF)
}

/// Stable project identity for the directory and machine that own a Session.
/// Local IDs remain compatible with `project.add`; remote IDs are namespaced
/// by host id so identical paths on different machines never share a node.
pub(crate) fn session_project_id(root: &str, host: Option<&str>) -> homie_proto::ProjectId {
    let location = host.map_or_else(|| root.to_owned(), |host| format!("ssh\0{host}\0{root}"));
    homie_proto::ProjectId(project_id(&location))
}

#[cfg(test)]
mod tests;
