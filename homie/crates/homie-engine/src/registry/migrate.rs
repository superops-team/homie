//! Envelope → split-store migration: copies the versioned single-file state
//! into the per-project/per-session layout, preserving a backup of the source.

use std::path::{Path, PathBuf};

use super::store::{JsonEnvelopeStore, PersistenceStore, SplitJsonStore, write_json_atomic};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SplitMigrationReport {
    pub dry_run: bool,
    pub project_count: usize,
    pub session_count: usize,
    pub backup_path: Option<PathBuf>,
    pub split_root: PathBuf,
}

pub fn migrate_envelope_to_split(
    state_file: impl AsRef<Path>,
    split_root: impl AsRef<Path>,
    dry_run: bool,
) -> std::io::Result<SplitMigrationReport> {
    let state_file = state_file.as_ref();
    let split_root = split_root.as_ref();
    let envelope = JsonEnvelopeStore::new(state_file);
    let state = envelope.load_state()?;
    let report = SplitMigrationReport {
        dry_run,
        project_count: state.projects.len(),
        session_count: state.sessions.len(),
        backup_path: (!dry_run).then(|| state_file.with_extension("json.backup")),
        split_root: split_root.to_path_buf(),
    };
    if dry_run {
        return Ok(report);
    }

    let backup_path = report.backup_path.as_ref().expect("backup path");
    if let Some(parent) = backup_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(state_file, backup_path)?;
    let split = SplitJsonStore::new(split_root);
    write_json_atomic(&split.projects_file(), &state.projects)?;
    std::fs::create_dir_all(split.sessions_dir())?;
    for session in &state.sessions {
        split.save_session(session)?;
    }
    Ok(report)
}
