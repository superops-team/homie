//! Persistence backends: the versioned single-file envelope and the
//! split per-project/per-session store, plus the shared atomic-write helper.

use std::path::{Path, PathBuf};

use homie_proto::SessionRecord;
use serde::Serialize;

use super::persisted::PersistedState;

pub trait PersistenceStore {
    fn load_projects(&self) -> std::io::Result<Vec<serde_json::Value>>;
    fn load_sessions(&self) -> std::io::Result<Vec<SessionRecord>>;
    fn save_project(&self, project: serde_json::Value) -> std::io::Result<()>;
    fn save_session(&self, session: &SessionRecord) -> std::io::Result<()>;
    fn delete_session(&self, session_id: &str) -> std::io::Result<()>;
    fn flush(&self) -> std::io::Result<()>;
}

#[derive(Clone, Debug)]
pub struct JsonEnvelopeStore {
    state_file: PathBuf,
}

impl JsonEnvelopeStore {
    pub fn new(state_file: impl Into<PathBuf>) -> Self {
        Self {
            state_file: state_file.into(),
        }
    }

    pub(crate) fn load_state(&self) -> std::io::Result<PersistedState> {
        let bytes = match std::fs::read(&self.state_file) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(PersistedState::current(Vec::new(), Vec::new()));
            }
            Err(error) => return Err(error),
        };
        serde_json::from_slice(&bytes).map_err(invalid_data)
    }

    fn write_state(&self, state: &PersistedState) -> std::io::Result<()> {
        write_json_atomic(&self.state_file, state)
    }
}

impl PersistenceStore for JsonEnvelopeStore {
    fn load_projects(&self) -> std::io::Result<Vec<serde_json::Value>> {
        Ok(self.load_state()?.projects)
    }

    fn load_sessions(&self) -> std::io::Result<Vec<SessionRecord>> {
        Ok(self.load_state()?.sessions)
    }

    fn save_project(&self, project: serde_json::Value) -> std::io::Result<()> {
        let mut state = self.load_state()?;
        let id = project.get("id").and_then(serde_json::Value::as_str);
        if let Some(id) = id
            && let Some(existing) = state.projects.iter_mut().find(|candidate| {
                candidate.get("id").and_then(serde_json::Value::as_str) == Some(id)
            })
        {
            *existing = project;
            return self.write_state(&state);
        }
        state.projects.push(project);
        self.write_state(&state)
    }

    fn save_session(&self, session: &SessionRecord) -> std::io::Result<()> {
        let mut state = self.load_state()?;
        if let Some(existing) = state
            .sessions
            .iter_mut()
            .find(|candidate| candidate.id == session.id)
        {
            *existing = session.clone();
        } else {
            state.sessions.push(session.clone());
        }
        self.write_state(&state)
    }

    fn delete_session(&self, session_id: &str) -> std::io::Result<()> {
        let mut state = self.load_state()?;
        state.sessions.retain(|session| session.id.0 != session_id);
        self.write_state(&state)
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SplitJsonStore {
    root: PathBuf,
}

impl SplitJsonStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub(crate) fn projects_file(&self) -> PathBuf {
        self.root.join("projects.json")
    }

    pub(crate) fn sessions_dir(&self) -> PathBuf {
        self.root.join("sessions")
    }

    fn session_file(&self, session_id: &str) -> PathBuf {
        self.sessions_dir().join(format!("{session_id}.json"))
    }
}

impl PersistenceStore for SplitJsonStore {
    fn load_projects(&self) -> std::io::Result<Vec<serde_json::Value>> {
        let path = self.projects_file();
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        serde_json::from_slice(&bytes).map_err(invalid_data)
    }

    fn load_sessions(&self) -> std::io::Result<Vec<SessionRecord>> {
        let dir = self.sessions_dir();
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        let mut sessions = Vec::new();
        for entry in entries {
            let path = entry?.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let bytes = std::fs::read(&path)?;
            match serde_json::from_slice::<SessionRecord>(&bytes) {
                Ok(session) => sessions.push(session),
                Err(_) => {
                    let quarantine = path.with_extension("json.corrupt");
                    let _ = std::fs::rename(&path, quarantine);
                }
            }
        }
        sessions.sort_by(|a, b| a.id.0.cmp(&b.id.0));
        Ok(sessions)
    }

    fn save_project(&self, project: serde_json::Value) -> std::io::Result<()> {
        let mut projects = self.load_projects()?;
        let id = project.get("id").and_then(serde_json::Value::as_str);
        if let Some(id) = id
            && let Some(existing) = projects.iter_mut().find(|candidate| {
                candidate.get("id").and_then(serde_json::Value::as_str) == Some(id)
            })
        {
            *existing = project;
            return write_json_atomic(&self.projects_file(), &projects);
        }
        projects.push(project);
        write_json_atomic(&self.projects_file(), &projects)
    }

    fn save_session(&self, session: &SessionRecord) -> std::io::Result<()> {
        write_json_atomic(&self.session_file(&session.id.0), session)
    }

    fn delete_session(&self, session_id: &str) -> std::io::Result<()> {
        match std::fs::remove_file(self.session_file(session_id)) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) fn write_json_atomic(path: &Path, value: &impl Serialize) -> std::io::Result<()> {
    let bytes = serde_json::to_vec(value).map_err(invalid_data)?;
    let temp = path.with_extension("json.tmp");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&temp, bytes)?;
    std::fs::rename(&temp, path)
}

pub(crate) fn invalid_data(error: impl std::error::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
}
