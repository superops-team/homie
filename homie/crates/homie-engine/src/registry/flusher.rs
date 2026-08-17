//! Deferred-persist timing: the debounce window, the background flusher
//! thread, and the [`Registry`] read/write methods.

use std::collections::HashMap;
use std::sync::Arc;

use homie_proto::SessionRecord;

use super::persisted::{PersistedState, fold_session_status, repair_persisted_agent_title};
use super::{Registry, session_project_id};

/// How long consecutive persists coalesce. Matches the reference implementation's
/// `PersistenceStore` debounce.
const PERSIST_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(500);

impl Drop for Registry {
    fn drop(&mut self) {
        // A deferred persist must not die with the process: embedders without
        // a flusher thread (tests, short-lived tools) still land their state.
        let _ = self.flush_dirty();
    }
}

/// Flushes deferred persists on a short cadence. One per daemon, next to the
/// events watcher.
pub fn spawn_persist_flusher(
    registry: Arc<std::sync::Mutex<Registry>>,
    stop: Arc<std::sync::atomic::AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("homie-persist-flusher".into())
        .spawn(move || {
            while !stop.load(std::sync::atomic::Ordering::SeqCst) {
                std::thread::sleep(PERSIST_DEBOUNCE);
                let Ok(mut registry) = registry.lock() else {
                    break;
                };
                let _ = registry.flush_dirty();
            }
        })
        .expect("spawn persist flusher")
}

impl Registry {
    /// Loads a persisted state file.
    ///
    /// A file that exists but will not parse is quarantined rather than
    /// ignored: treating it as a fresh install would make the next write
    /// overwrite every session record the user had.
    pub fn load(&mut self) -> std::io::Result<usize> {
        let bytes = match std::fs::read(&self.state_file) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(error) => return Err(error),
        };
        match serde_json::from_slice::<PersistedState>(&bytes) {
            Ok(state) => {
                self.projects = state.projects;
                let project_roots = self
                    .projects
                    .iter()
                    .filter_map(|project| {
                        Some((
                            project.get("id")?.as_str()?.to_owned(),
                            project.get("root")?.as_str()?.to_owned(),
                        ))
                    })
                    .collect::<HashMap<_, _>>();
                let mut locations = Vec::with_capacity(state.sessions.len());
                for mut record in state.sessions {
                    repair_persisted_agent_title(&mut record);
                    // Resolve the owning project before repairing its
                    // location namespace. In particular, a linked worktree's
                    // cwd is not its first-level project root.
                    let project_root = project_roots
                        .get(&record.project_id.0)
                        .cloned()
                        .unwrap_or_else(|| record.cwd.clone());
                    record.project_id = session_project_id(&project_root, record.host.as_deref());
                    locations.push((project_root, record.host.clone()));
                    self.records.insert(record.id.0.clone(), record);
                }
                for (root, host) in locations {
                    self.ensure_session_project(&root, host.as_deref());
                }
                Ok(self.records.len())
            }
            Err(error) => {
                let quarantine = self.state_file.with_extension("json.corrupt");
                let _ = std::fs::rename(&self.state_file, &quarantine);
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "state file did not parse ({error}); quarantined at {}",
                        quarantine.display()
                    ),
                ))
            }
        }
    }

    /// Persists the current state — immediately when the last write is older
    /// than the debounce window, otherwise by marking dirty for the flusher
    /// ([`spawn_persist_flusher`]) or the next call to pick up. Serializing
    /// and atomically rewriting every record used to happen on every single
    /// mutation, including each tab switch's mark-seen.
    pub fn persist(&mut self) -> std::io::Result<()> {
        if let Some(last) = self.last_persist
            && last.elapsed() < PERSIST_DEBOUNCE
        {
            self.dirty = true;
            return Ok(());
        }
        self.persist_now()
    }

    /// Writes out a deferred persist, if one is pending.
    pub fn flush_dirty(&mut self) -> std::io::Result<()> {
        if !self.dirty {
            return Ok(());
        }
        self.persist_now()
    }

    /// Writes the current state atomically, unconditionally.
    fn persist_now(&mut self) -> std::io::Result<()> {
        let state = PersistedState::current(self.records_for_persistence(), self.projects.clone());
        let bytes = serde_json::to_vec(&state)?;
        let temp = self.state_file.with_extension("json.tmp");
        if let Some(parent) = self.state_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&temp, &bytes)?;
        // Rename is atomic, so a crash mid-write cannot truncate the real file.
        std::fs::rename(&temp, &self.state_file)?;
        self.dirty = false;
        self.last_persist = Some(std::time::Instant::now());
        Ok(())
    }

    fn records_for_persistence(&self) -> Vec<SessionRecord> {
        let mut records: Vec<SessionRecord> = self.records.values().cloned().collect();
        for record in &mut records {
            if let Some(session) = self.sessions.get(&record.id.0) {
                fold_session_status(record, &session.view());
            }
        }
        records.sort_by(|a, b| a.id.0.cmp(&b.id.0));
        records
    }
}
