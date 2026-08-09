use homie_storage::{HistoryEntrySummary, HistoryEntryUpsert, Storage, StorageError};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashSet;
use std::fs::{self, File, Metadata};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

const MAX_ENTRIES: usize = 500;
const CLAUDE_HEAD_CAP: usize = 8 << 20;
const CLAUDE_TAIL_BYTES: usize = 16 << 10;
const CODEX_FIRST_LINE_CAP: usize = 512 << 10;
const CODEX_FIRST_PROMPT_CAP: usize = 8 << 20;
const DEFAULT_MAX_SCAN_FILES: usize = 10_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryRoots {
    pub claude: PathBuf,
    pub codex: PathBuf,
}

impl HistoryRoots {
    #[must_use]
    pub fn in_home(home: &Path) -> Self {
        Self {
            claude: home.join(".claude/projects"),
            codex: home.join(".codex/sessions"),
        }
    }

    #[must_use]
    pub fn current_user() -> Self {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/nonexistent"));
        Self::in_home(&home)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedHistoryEntry {
    pub agent_kind: String,
    pub external_id: String,
    pub cwd: PathBuf,
    pub title: Option<String>,
    pub title_source: String,
    pub transcript_path: PathBuf,
    pub last_active_at: i64,
    pub created_at: Option<i64>,
    pub cwd_exists: bool,
}

#[derive(Debug, Error)]
pub enum HistoryScanError {
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("history scan interrupted")]
    Interrupted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HistoryScanLimits {
    pub max_files: usize,
}

impl Default for HistoryScanLimits {
    fn default() -> Self {
        Self {
            max_files: DEFAULT_MAX_SCAN_FILES,
        }
    }
}

pub fn scan_history(
    roots: &HistoryRoots,
    tracked: &HashSet<String>,
) -> Result<Vec<ScannedHistoryEntry>, HistoryScanError> {
    scan_history_bounded(roots, tracked, HistoryScanLimits::default(), || Ok(()))
}

pub fn scan_history_bounded(
    roots: &HistoryRoots,
    tracked: &HashSet<String>,
    limits: HistoryScanLimits,
    mut checkpoint: impl FnMut() -> Result<(), HistoryScanError>,
) -> Result<Vec<ScannedHistoryEntry>, HistoryScanError> {
    let mut remaining_files = limits.max_files;
    let mut entries = scan_claude(&roots.claude, &mut remaining_files, &mut checkpoint)?;
    if remaining_files > 0 {
        entries.extend(scan_codex(
            &roots.codex,
            &mut remaining_files,
            &mut checkpoint,
        )?);
    }

    let mut seen = tracked.clone();
    entries.retain(|entry| seen.insert(entry.external_id.clone()));
    entries.sort_by(|left, right| {
        right
            .last_active_at
            .cmp(&left.last_active_at)
            .then_with(|| left.external_id.cmp(&right.external_id))
    });
    entries.truncate(MAX_ENTRIES);
    Ok(entries)
}

pub fn write_history_to_storage(
    storage: &Storage,
    entries: &[ScannedHistoryEntry],
) -> Result<Vec<HistoryEntrySummary>, HistoryScanError> {
    storage
        .connection()
        .execute_batch("SAVEPOINT homie_history_scan")
        .map_err(StorageError::from)?;
    let result = (|| {
        let mut written = Vec::with_capacity(entries.len());
        for entry in entries {
            written.push(storage.upsert_history_entry(HistoryEntryUpsert {
                agent_kind: entry.agent_kind.clone(),
                external_id: entry.external_id.clone(),
                cwd: entry.cwd.clone(),
                title: entry.title.clone(),
                title_source: entry.title_source.clone(),
                transcript_path: entry.transcript_path.clone(),
                last_active_at: entry.last_active_at,
                created_at: entry.created_at,
                cwd_exists: entry.cwd_exists,
                metadata: json!({
                    "source": "transcript_history_scanner",
                    "agent_kind": entry.agent_kind,
                }),
            })?);
        }
        Ok(written)
    })();
    match result {
        Ok(written) => {
            storage
                .connection()
                .execute_batch("RELEASE homie_history_scan")
                .map_err(StorageError::from)?;
            Ok(written)
        }
        Err(error) => {
            storage
                .connection()
                .execute_batch(
                    "ROLLBACK TO homie_history_scan;
                     RELEASE homie_history_scan;",
                )
                .map_err(StorageError::from)?;
            Err(error)
        }
    }
}

#[must_use]
pub fn resume_command(entry: &ScannedHistoryEntry) -> Option<String> {
    if !entry.cwd_exists || !entry.cwd.is_dir() {
        return None;
    }
    if !entry
        .external_id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return None;
    }
    match entry.agent_kind.as_str() {
        "claude_code" => Some(format!("claude --resume {}", entry.external_id)),
        "codex" => Some(format!("codex resume {}", entry.external_id)),
        _ => None,
    }
}

fn scan_claude(
    root: &Path,
    remaining_files: &mut usize,
    checkpoint: &mut impl FnMut() -> Result<(), HistoryScanError>,
) -> Result<Vec<ScannedHistoryEntry>, HistoryScanError> {
    let mut result = Vec::new();
    for project in child_dirs(root) {
        if *remaining_files == 0 {
            break;
        }
        let Ok(files) = fs::read_dir(project) else {
            continue;
        };
        for file in files.flatten() {
            if *remaining_files == 0 {
                break;
            }
            let path = file.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
                continue;
            }
            *remaining_files -= 1;
            checkpoint()?;
            if let Some(entry) = claude_entry(&path) {
                result.push(entry);
            }
        }
    }
    Ok(result)
}

fn claude_entry(path: &Path) -> Option<ScannedHistoryEntry> {
    let external_id = path.file_stem()?.to_str()?.to_owned();
    if external_id.len() < 32 {
        return None;
    }
    let metadata = fs::metadata(path).ok()?;
    let mut cwd = None;
    let mut first_prompt = None;
    for line in read_capped_lines(path, CLAUDE_HEAD_CAP).ok()? {
        let Ok(object) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if first_prompt.is_none() {
            first_prompt = claude_user_text(&object);
        }
        if let Some(value) = object.get("cwd").and_then(Value::as_str)
            && !value.is_empty()
        {
            cwd = Some(PathBuf::from(value));
            break;
        }
    }
    let cwd = cwd?;
    let title = latest_claude_ai_title(path).or_else(|| first_prompt.map(title_from_prompt));
    Some(history_entry(
        "claude_code",
        external_id,
        cwd,
        title,
        "agent_title",
        path,
        &metadata,
    ))
}

fn claude_user_text(object: &Value) -> Option<String> {
    if object.get("type").and_then(Value::as_str) != Some("user") {
        return None;
    }
    let content = object.get("message")?.get("content")?;
    if let Some(text) = content.as_str().filter(|text| !text.is_empty()) {
        return Some(text.to_owned());
    }
    content.as_array()?.iter().find_map(|item| {
        (item.get("type").and_then(Value::as_str) == Some("text"))
            .then(|| item.get("text").and_then(Value::as_str))
            .flatten()
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
    })
}

fn latest_claude_ai_title(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let end = file.seek(SeekFrom::End(0)).ok()?;
    let start = end.saturating_sub(CLAUDE_TAIL_BYTES as u64);
    file.seek(SeekFrom::Start(start)).ok()?;
    let mut bytes = Vec::with_capacity(CLAUDE_TAIL_BYTES + (4 << 10));
    file.take((CLAUDE_TAIL_BYTES + (4 << 10)) as u64)
        .read_to_end(&mut bytes)
        .ok()?;

    let mut newest = None;
    for line in String::from_utf8_lossy(&bytes).lines() {
        if !line.contains("\"ai-title\"") {
            continue;
        }
        let Ok(object) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if object.get("type").and_then(Value::as_str) == Some("ai-title")
            && let Some(title) = object
                .get("aiTitle")
                .and_then(Value::as_str)
                .filter(|title| !title.is_empty())
        {
            newest = Some(title.to_owned());
        }
    }
    newest
}

fn scan_codex(
    root: &Path,
    remaining_files: &mut usize,
    checkpoint: &mut impl FnMut() -> Result<(), HistoryScanError>,
) -> Result<Vec<ScannedHistoryEntry>, HistoryScanError> {
    let mut result = Vec::new();
    'years: for year in child_dirs(root) {
        for month in child_dirs(&year) {
            for day in child_dirs(&month) {
                if *remaining_files == 0 {
                    break 'years;
                }
                let Ok(files) = fs::read_dir(day) else {
                    continue;
                };
                for file in files.flatten() {
                    if *remaining_files == 0 {
                        break 'years;
                    }
                    let path = file.path();
                    let name = path.file_name().and_then(|name| name.to_str());
                    if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl")
                        || !name.is_some_and(|name| name.starts_with("rollout-"))
                    {
                        continue;
                    }
                    *remaining_files -= 1;
                    checkpoint()?;
                    if let Some(entry) = codex_entry(&path) {
                        result.push(entry);
                    }
                }
            }
        }
    }
    Ok(result)
}

fn codex_entry(path: &Path) -> Option<ScannedHistoryEntry> {
    let first = read_first_line(path, CODEX_FIRST_LINE_CAP).ok()??;
    let object: Value = serde_json::from_str(&first).ok()?;
    if object.get("type").and_then(Value::as_str) != Some("session_meta") {
        return None;
    }
    let payload = object.get("payload")?;
    let external_id = payload.get("id")?.as_str()?.to_owned();
    let cwd = PathBuf::from(payload.get("cwd")?.as_str()?);
    if cwd.as_os_str().is_empty() {
        return None;
    }
    let metadata = fs::metadata(path).ok()?;
    let title = first_codex_user_prompt(path)
        .map(title_from_prompt)
        .or_else(|| Some(format!("Codex - {}", folder_name(&cwd))));
    Some(history_entry(
        "codex",
        external_id,
        cwd,
        title,
        "first_user_prompt",
        path,
        &metadata,
    ))
}

fn first_codex_user_prompt(path: &Path) -> Option<String> {
    for line in read_capped_lines(path, CODEX_FIRST_PROMPT_CAP).ok()? {
        if !line.contains("\"user_message\"") {
            continue;
        }
        let Ok(object) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let payload = object.get("payload")?;
        if object.get("type").and_then(Value::as_str) == Some("event_msg")
            && payload.get("type").and_then(Value::as_str) == Some("user_message")
            && let Some(message) = payload
                .get("message")
                .and_then(Value::as_str)
                .filter(|message| !message.trim().is_empty())
        {
            return Some(message.to_owned());
        }
    }
    None
}

fn history_entry(
    agent_kind: &str,
    external_id: String,
    cwd: PathBuf,
    title: Option<String>,
    title_source: &str,
    path: &Path,
    metadata: &Metadata,
) -> ScannedHistoryEntry {
    ScannedHistoryEntry {
        agent_kind: agent_kind.to_string(),
        external_id,
        cwd_exists: cwd.is_dir(),
        cwd,
        title,
        title_source: title_source.to_string(),
        transcript_path: path.to_path_buf(),
        last_active_at: system_time(metadata.modified().ok()),
        created_at: metadata.created().ok().map(|time| system_time(Some(time))),
    }
}

fn system_time(time: Option<SystemTime>) -> i64 {
    time.and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |duration| {
            i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
        })
}

fn read_capped_lines(path: &Path, cap: usize) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file.take(cap as u64));
    let mut result = Vec::new();
    let mut line = String::new();
    while reader.read_line(&mut line)? != 0 {
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        result.push(std::mem::take(&mut line));
    }
    Ok(result)
}

fn read_first_line(path: &Path, cap: usize) -> io::Result<Option<String>> {
    Ok(read_capped_lines(path, cap)?.into_iter().next())
}

fn child_dirs(root: &Path) -> Vec<PathBuf> {
    let Ok(children) = fs::read_dir(root) else {
        return Vec::new();
    };
    children
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect()
}

fn title_from_prompt(prompt: String) -> String {
    let collapsed = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut characters = collapsed.chars();
    if characters.clone().count() <= 60 {
        return collapsed;
    }
    characters.by_ref().take(59).collect::<String>() + "..."
}

fn folder_name(path: &Path) -> String {
    if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
        name.to_string()
    } else {
        path.to_string_lossy().into_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[test]
    fn bounded_scan_stops_after_the_file_limit() {
        let fixture = tempfile::tempdir().expect("tempdir");
        let roots = HistoryRoots {
            claude: fixture.path().join("claude"),
            codex: fixture.path().join("codex"),
        };
        let cwd = fixture.path().join("project");
        fs::create_dir_all(&cwd).expect("cwd");
        let day = roots.codex.join("2026/08/08");
        fs::create_dir_all(&day).expect("day");
        for index in 0..10 {
            fs::write(
                day.join(format!("rollout-{index}.jsonl")),
                format!(
                    "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"thread-{index}\",\"cwd\":{}}}}}\n",
                    serde_json::to_string(&cwd).expect("cwd")
                ),
            )
            .expect("fixture");
        }

        let entries = scan_history_bounded(
            &roots,
            &HashSet::new(),
            HistoryScanLimits { max_files: 3 },
            || Ok(()),
        )
        .expect("scan");

        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn interrupted_scan_returns_an_error_instead_of_partial_entries() {
        let fixture = tempfile::tempdir().expect("tempdir");
        let roots = HistoryRoots {
            claude: fixture.path().join("claude"),
            codex: fixture.path().join("codex"),
        };
        let day = roots.codex.join("2026/08/08");
        fs::create_dir_all(&day).expect("day");
        for index in 0..3 {
            fs::write(
                day.join(format!("rollout-{index}.jsonl")),
                format!(
                    "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"thread-{index}\",\"cwd\":\"/tmp\"}}}}\n"
                ),
            )
            .expect("fixture");
        }
        let checkpoints = AtomicUsize::new(0);

        let result = scan_history_bounded(
            &roots,
            &HashSet::new(),
            HistoryScanLimits { max_files: 10 },
            || {
                if checkpoints.fetch_add(1, Ordering::SeqCst) >= 1 {
                    Err(HistoryScanError::Interrupted)
                } else {
                    Ok(())
                }
            },
        );

        assert!(matches!(result, Err(HistoryScanError::Interrupted)));
    }

    #[test]
    fn failed_history_commit_rolls_back_every_entry() {
        let fixture = tempfile::tempdir().expect("tempdir");
        let storage = homie_storage::open_or_create(homie_storage::StorageConfig {
            data_dir: fixture.path().join("data"),
        })
        .expect("storage");
        storage.migrate().expect("migrate");
        storage.seed_defaults().expect("seed");
        storage
            .connection()
            .execute_batch(
                "CREATE TRIGGER fail_bad_history
                 BEFORE INSERT ON history_entries
                 WHEN NEW.external_id = 'bad'
                 BEGIN
                   SELECT RAISE(ABORT, 'forced history failure');
                 END;",
            )
            .expect("trigger");
        let entry = |external_id: &str| ScannedHistoryEntry {
            agent_kind: "codex".to_string(),
            external_id: external_id.to_string(),
            cwd: fixture.path().to_path_buf(),
            title: None,
            title_source: "test".to_string(),
            transcript_path: fixture.path().join(format!("{external_id}.jsonl")),
            last_active_at: 1,
            created_at: None,
            cwd_exists: true,
        };

        let result = write_history_to_storage(&storage, &[entry("good"), entry("bad")]);

        assert!(result.is_err());
        assert!(
            storage
                .list_history_entries(10)
                .expect("history")
                .is_empty()
        );
    }
}
