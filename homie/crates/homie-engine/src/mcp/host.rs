//! Executing MCP tools against a live registry.
//!
//! The calling agent's own session id arrives in its environment
//! (`HOMIE_SESSION_ID`), which is what lets `whoami` and `list_children`
//! answer questions about *this* session and the ones it spawned.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use homie_proto::{SessionId, SessionRecord, SessionStatus};
use serde_json::{Value, json};

use super::ToolHost;
use crate::git;
use crate::registry::Registry;

/// Environment variable carrying the calling session's id.
pub const SESSION_ID_ENV: &str = "HOMIE_SESSION_ID";

const DEFAULT_READ_BYTES: usize = 8_000;
const DEFAULT_WAIT_SECONDS: f64 = 300.0;
const DEFAULT_CHILDREN_WAIT_SECONDS: f64 = 600.0;
/// How often a wait re-checks. Long enough not to spin, short enough that a
/// state change is noticed promptly.
const WAIT_POLL: Duration = Duration::from_millis(100);

pub struct RegistryHost {
    registry: Arc<Mutex<Registry>>,
    logs_dir: PathBuf,
    holder: Option<crate::session::HolderConfig>,
    /// The session calling these tools, when it identified itself.
    caller: Option<String>,
    /// Lazily-launched Playwright sidecar for `browser` / `test_run`.
    browser: std::sync::OnceLock<crate::browser::BrowserPool>,
}

impl RegistryHost {
    pub fn new(registry: Arc<Mutex<Registry>>, logs_dir: impl Into<PathBuf>) -> Self {
        Self {
            registry,
            logs_dir: logs_dir.into(),
            holder: None,
            caller: std::env::var(SESSION_ID_ENV).ok(),
            browser: std::sync::OnceLock::new(),
        }
    }

    /// Spawn sessions through holders, so they survive this process.
    pub fn with_holder(mut self, holder: crate::session::HolderConfig) -> Self {
        self.holder = Some(holder);
        self
    }

    /// Overrides the calling session, for tests and for hosts that know the
    /// caller by other means.
    pub fn with_caller(mut self, caller: Option<String>) -> Self {
        self.caller = caller;
        self
    }

    fn registry(&self) -> Result<std::sync::MutexGuard<'_, Registry>, String> {
        self.registry
            .lock()
            .map_err(|_| "engine state is poisoned".to_string())
    }
}

fn required_str(arguments: &Value, key: &str) -> Result<String, String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("{key} is required"))
}

/// Optional string-array argument helper (empty when absent or non-array).
fn opt_strings(arguments: &Value, key: &str) -> Vec<String> {
    arguments
        .get(key)
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn relation_word(relation: Relation) -> &'static str {
    match relation {
        Relation::Caller => "self",
        Relation::Parent => "parent",
        Relation::Child => "child",
        Relation::Ancestor => "ancestor",
        Relation::Descendant => "descendant",
        Relation::Sibling => "sibling",
        Relation::Unrelated => "unrelated",
    }
}

/// Render a structured child -> parent report as terminal prose with labelled
/// sections; the parent is a language model reading its own screen.
fn render_report(
    record: &SessionRecord,
    caller: &str,
    status: &str,
    summary: &str,
    arguments: &Value,
) -> String {
    let who = if record.title.is_empty() {
        format!("id:{caller}")
    } else {
        format!("id:{caller} ({})", record.title)
    };
    let mut lines = vec![
        format!("[report from {who} · status: {status}]"),
        String::new(),
        format!("Summary: {summary}"),
    ];
    if let Some(details) = arguments.get("details").and_then(Value::as_str)
        && !details.is_empty()
    {
        lines.push(String::new());
        lines.push(details.to_string());
    }
    for (title, key) in [
        ("Blockers", "blockers"),
        ("Questions", "questions"),
        ("Next steps", "next_steps"),
        ("Changed", "changed_paths"),
        ("Artifacts", "artifacts"),
        ("Proof", "proof"),
    ] {
        let items = opt_strings(arguments, key);
        let kept: Vec<&str> = items
            .iter()
            .map(String::as_str)
            .filter(|s| !s.is_empty())
            .collect();
        if kept.is_empty() {
            continue;
        }
        lines.push(String::new());
        lines.push(format!("{title}:"));
        lines.extend(kept.into_iter().map(|s| format!("- {s}")));
    }
    lines.join("\n")
}

fn status_word(status: &SessionStatus) -> &'static str {
    match status {
        SessionStatus::Starting => "starting",
        SessionStatus::Idle => "idle",
        SessionStatus::Working => "working",
        SessionStatus::NeedsInput(_) => "needsInput",
        SessionStatus::Exited(_) => "exited",
        SessionStatus::Unknown => "unknown",
    }
}

/// The spawn graph, read from `SessionRecord.parent`. Reads across the graph
/// are open; writes to a session that is not your parent or your own child get
/// a provenance header so the receiving agent can tell it apart from its user.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Relation {
    Caller,
    Parent,
    Child,
    Ancestor,
    Descendant,
    Sibling,
    Unrelated,
}

impl Relation {
    /// Your parent and your direct children are the delegation channel; both
    /// ends already know who the other is, so extra framing only confuses an
    /// agent mid-task.
    fn delivers_verbatim(self) -> bool {
        matches!(self, Relation::Parent | Relation::Child)
    }
}

struct Lineage {
    records: Vec<SessionRecord>,
    caller: Option<SessionId>,
}

impl Lineage {
    fn new(records: Vec<SessionRecord>, caller: Option<SessionId>) -> Self {
        Self { records, caller }
    }

    fn record(&self, id: &SessionId) -> Option<&SessionRecord> {
        self.records.iter().find(|r| &r.id == id)
    }

    fn children_of(&self, id: &SessionId) -> Vec<&SessionRecord> {
        self.records
            .iter()
            .filter(|r| r.parent.as_ref() == Some(id))
            .collect()
    }

    /// Breadth-first descendants with a visited set so a corrupted or
    /// hand-edited state file that describes a cycle degrades to a short
    /// answer instead of hanging the daemon call.
    fn descendants_of(&self, id: &SessionId) -> Vec<&SessionRecord> {
        let mut seen: std::collections::HashSet<&SessionId> = std::collections::HashSet::new();
        seen.insert(id);
        let mut queue = self.children_of(id);
        let mut out = Vec::new();
        while let Some(next) = queue.first().copied() {
            queue.remove(0);
            if !seen.insert(&next.id) {
                continue;
            }
            out.push(next);
            queue.extend(self.children_of(&next.id));
        }
        out
    }

    /// Walk to the root, nearest ancestor first, with the same cycle guard.
    fn ancestors_of(&self, id: &SessionId) -> Vec<&SessionRecord> {
        let mut seen: std::collections::HashSet<&SessionId> = std::collections::HashSet::new();
        seen.insert(id);
        let mut out = Vec::new();
        let mut cursor = self.record(id).and_then(|r| r.parent.as_ref());
        while let Some(current) = cursor {
            if !seen.insert(current) {
                break;
            }
            let Some(record) = self.record(current) else {
                break;
            };
            out.push(record);
            cursor = record.parent.as_ref();
        }
        out
    }

    fn relation_to(&self, target: &SessionId) -> Relation {
        let Some(caller) = &self.caller else {
            return Relation::Unrelated;
        };
        if caller == target {
            return Relation::Caller;
        }
        if self.record(caller).and_then(|r| r.parent.as_ref()) == Some(target) {
            return Relation::Parent;
        }
        if self.record(target).and_then(|r| r.parent.as_ref()) == Some(caller) {
            return Relation::Child;
        }
        if self.ancestors_of(caller).iter().any(|r| &r.id == target) {
            return Relation::Ancestor;
        }
        if self.descendants_of(caller).iter().any(|r| &r.id == target) {
            return Relation::Descendant;
        }
        let mine = self.record(caller).and_then(|r| r.parent.as_ref());
        let theirs = self.record(target).and_then(|r| r.parent.as_ref());
        if mine.is_some() && mine == theirs {
            return Relation::Sibling;
        }
        Relation::Unrelated
    }

    /// Attribution for a cross-session write. Verbatim for the delegation
    /// channel or when there is no caller; otherwise prefixes one line naming
    /// the sender.
    fn frame(&self, text: &str, relation: Relation) -> String {
        if relation.delivers_verbatim() {
            return text.to_string();
        }
        let Some(caller) = &self.caller else {
            return text.to_string();
        };
        let who = self
            .record(caller)
            .map(|r| format!("id:{} ({})", r.id.0, r.title))
            .unwrap_or_else(|| format!("id:{}", caller.0));
        format!(
            "[message from {who}, channel: homie — reply with send_prompt to that id]\n\n{text}"
        )
    }
}

impl ToolHost for RegistryHost {
    fn call(&self, tool: &str, arguments: &Value) -> Result<Value, String> {
        match tool {
            "list_agents" => {
                let registry = self.registry()?;
                let agents: Vec<Value> = registry
                    .records()
                    .into_iter()
                    .map(|record| {
                        json!({
                            "id": record.id.0,
                            "kind": record.kind.id(),
                            "title": record.title,
                            "status": status_word(&record.status),
                            "cwd": record.cwd,
                            "parent": record.parent.map(|parent| parent.0),
                        })
                    })
                    .collect();
                Ok(json!({ "agents": agents }))
            }

            "get_status" => {
                let id = required_str(arguments, "session_id")?;
                let registry = self.registry()?;
                let record = registry
                    .records()
                    .into_iter()
                    .find(|record| record.id.0 == id)
                    .ok_or_else(|| format!("no session {id}"))?;
                Ok(json!({
                    "id": record.id.0,
                    "status": status_word(&record.status),
                    "title": record.title,
                    "cwd": record.cwd,
                    "needsInput": record.needs_input.map(|detail| json!({
                        "kind": format!("{:?}", detail.kind),
                        "summary": detail.summary,
                        "options": detail.options,
                    })),
                }))
            }

            "send_prompt" => {
                let id = required_str(arguments, "session_id")?;
                let text = required_str(arguments, "text")?;
                let submit = arguments
                    .get("submit")
                    .and_then(Value::as_bool)
                    .unwrap_or(true);

                let registry = self.registry()?;
                let lineage = Lineage::new(registry.records(), self.caller.clone().map(SessionId));
                let target = SessionId(id.clone());
                let relation = lineage.relation_to(&target);
                if relation == Relation::Caller {
                    return Err("you cannot send_prompt to yourself".to_string());
                }
                let framed = lineage.frame(&text, relation);
                let session = registry
                    .get(&id)
                    .ok_or_else(|| format!("no session {id}"))?;
                let payload = if submit {
                    format!("{framed}\r")
                } else {
                    framed.clone()
                };
                session
                    .write_input(payload.as_bytes())
                    .map_err(|error| error.to_string())?;
                Ok(
                    json!({ "sent": text.len(), "submitted": submit, "relation": relation_word(relation) }),
                )
            }

            "read_output" => {
                let id = required_str(arguments, "session_id")?;
                let max_bytes = arguments
                    .get("max_bytes")
                    .and_then(Value::as_u64)
                    .map(|value| value as usize)
                    .unwrap_or(DEFAULT_READ_BYTES);

                let registry = self.registry()?;
                let session = registry
                    .get(&id)
                    .ok_or_else(|| format!("no session {id}"))?;
                // Read from the end: the recent screen is what a caller wants,
                // and the whole log can be megabytes.
                let tail = session.view().tail_offset;
                let from = tail.saturating_sub(max_bytes as u64);
                let (offset, bytes) = session.read_output(from, max_bytes);
                Ok(json!({
                    "offset": offset,
                    "output": String::from_utf8_lossy(&bytes),
                }))
            }

            "release_agent" => {
                let id = required_str(arguments, "session_id")?;
                let registry = self.registry()?;
                let lineage = Lineage::new(registry.records(), self.caller.clone().map(SessionId));
                let target = SessionId(id.clone());
                let relation = lineage.relation_to(&target);
                if matches!(relation, Relation::Caller | Relation::Ancestor) {
                    return Err(
                        "release_agent refuses to kill you or any of your ancestors".to_string()
                    );
                }
                drop(registry);
                let mut registry = self.registry()?;
                let exit = registry
                    .terminate(&id, Duration::from_secs(3))
                    .map_err(|error| error.to_string())?;
                if exit.is_none() {
                    return Err(format!("no session {id}"));
                }
                let _ = registry.persist();
                Ok(json!({ "released": id }))
            }

            "wait_for_agent" => {
                let id = required_str(arguments, "session_id")?;
                let until = arguments
                    .get("until")
                    .and_then(Value::as_str)
                    .unwrap_or("done")
                    .to_string();
                let timeout = arguments
                    .get("timeout_seconds")
                    .and_then(Value::as_f64)
                    .unwrap_or(DEFAULT_WAIT_SECONDS);
                self.wait_for(&id, &until, timeout)
            }

            "create_worktree" => {
                let repo = required_str(arguments, "repo")?;
                let branch = arguments.get("branch").and_then(Value::as_str);
                let base = arguments.get("base").and_then(Value::as_str);
                let info = git::create_worktree(Path::new(&repo), branch, base)
                    .map_err(|error| error.to_string())?;
                Ok(json!({ "path": info.path, "branch": info.branch }))
            }

            "list_worktrees" => {
                let repo = required_str(arguments, "repo")?;
                let worktrees: Vec<Value> = git::list_worktrees(Path::new(&repo))
                    .map_err(|error| error.to_string())?
                    .into_iter()
                    .map(|info| json!({ "path": info.path, "branch": info.branch }))
                    .collect();
                Ok(json!({ "worktrees": worktrees }))
            }

            "remove_worktree" => {
                let repo = required_str(arguments, "repo")?;
                let worktree = required_str(arguments, "worktree")?;
                let force = arguments
                    .get("force")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                git::remove_worktree(Path::new(&repo), &worktree, force)
                    .map_err(|error| error.to_string())?;
                Ok(json!({ "removed": worktree }))
            }

            "whoami" => {
                let caller = self.caller.clone().ok_or_else(|| {
                    format!("this session did not identify itself; {SESSION_ID_ENV} is unset")
                })?;
                let registry = self.registry()?;
                let record = registry
                    .records()
                    .into_iter()
                    .find(|record| record.id.0 == caller)
                    .ok_or_else(|| format!("no session {caller}"))?;
                Ok(json!({
                    "id": record.id.0,
                    "kind": record.kind.id(),
                    "title": record.title,
                    "cwd": record.cwd,
                    "parent": record.parent.map(|parent| parent.0),
                }))
            }

            "list_children" => {
                let caller = self.caller.clone().ok_or_else(|| {
                    format!("this session did not identify itself; {SESSION_ID_ENV} is unset")
                })?;
                let registry = self.registry()?;
                let children: Vec<Value> = registry
                    .records()
                    .into_iter()
                    .filter(|record| record.parent.as_ref() == Some(&SessionId(caller.clone())))
                    .map(|record| {
                        json!({
                            "id": record.id.0,
                            "kind": record.kind.id(),
                            "title": record.title,
                            "status": status_word(&record.status),
                        })
                    })
                    .collect();
                Ok(json!({ "children": children }))
            }

            "wait_for_children" => {
                let caller = self.caller.clone().ok_or_else(|| {
                    format!("this session did not identify itself; {SESSION_ID_ENV} is unset")
                })?;
                let timeout = arguments
                    .get("timeout_seconds")
                    .and_then(Value::as_f64)
                    .unwrap_or(DEFAULT_CHILDREN_WAIT_SECONDS);
                self.wait_for_children(&caller, timeout)
            }

            // spawn_agent is served by the control layer, which owns log paths
            "spawn_agent" => self.spawn_agent(arguments),

            "get_artifacts" => {
                let id = required_str(arguments, "session_id")?;
                let registry = self.registry()?;
                let record = registry
                    .records()
                    .into_iter()
                    .find(|r| r.id.0 == id)
                    .ok_or_else(|| format!("no session {id}"))?;
                Ok(json!({
                    "session_id": id,
                    "artifacts": record.artifacts,
                    "pull_requests": record.pull_requests,
                    "listening_ports": record.listening_ports,
                }))
            }

            "summarize_children" => {
                let caller = self.require_caller()?;
                let rows = arguments
                    .get("rows")
                    .and_then(Value::as_u64)
                    .map(|v| (v as usize).clamp(1, 60))
                    .unwrap_or(14);
                let registry = self.registry()?;
                let records = registry.records();
                let lineage = Lineage::new(records, Some(SessionId(caller.clone())));
                let requested = opt_strings(arguments, "session_ids");
                let children: Vec<SessionRecord> = lineage
                    .children_of(&SessionId(caller.clone()))
                    .into_iter()
                    .filter(|r| requested.is_empty() || requested.contains(&r.id.0))
                    .cloned()
                    .collect();
                let items = children
                    .iter()
                    .map(|r| {
                        let mut obj = json!({
                            "id": r.id.0,
                            "title": r.title,
                            "status": status_word(&r.status),
                        });
                        if let Some(session) = registry.get(&r.id.0) {
                            let tail = session.view().tail_offset;
                            let from = tail.saturating_sub(4096);
                            let (_, bytes) = session.read_output(from, 4096);
                            let text = String::from_utf8_lossy(&bytes);
                            let lines: Vec<&str> = text
                                .lines()
                                .map(str::trim)
                                .filter(|l| !l.is_empty())
                                .collect();
                            let tail_lines: Vec<&str> =
                                lines.iter().rev().take(rows).rev().copied().collect();
                            obj["screen_tail"] = json!(tail_lines.join("\n"));
                        }
                        if let Some(artifacts) = &r.artifacts {
                            obj["artifacts"] =
                                json!(artifacts.iter().map(|a| a.url.clone()).collect::<Vec<_>>());
                        }
                        obj
                    })
                    .collect::<Vec<_>>();
                Ok(json!({ "children": items, "count": items.len() }))
            }

            "report_to_parent" => {
                let summary = required_str(arguments, "summary")?;
                let status = arguments
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("update")
                    .to_string();
                let registry = self.registry()?;
                let lineage = Lineage::new(registry.records(), self.caller.clone().map(SessionId));
                let caller = self.require_caller()?;
                let record = lineage
                    .record(&SessionId(caller.clone()))
                    .cloned()
                    .ok_or_else(|| format!("no session {caller}"))?;
                let parent_id = record.parent.clone().ok_or_else(|| {
                    "this session has no parent; it was started by the user, so there is nobody to report to".to_string()
                })?;
                let rendered = render_report(&record, &caller, &status, &summary, arguments);
                let submit = arguments
                    .get("submit")
                    .and_then(Value::as_bool)
                    .unwrap_or(true);
                let parent = registry
                    .get(&parent_id.0)
                    .ok_or_else(|| format!("your parent session ({}) is gone", parent_id.0))?;
                let payload = if submit {
                    format!("{rendered}\r")
                } else {
                    rendered.clone()
                };
                parent
                    .write_input(payload.as_bytes())
                    .map_err(|e| e.to_string())?;
                Ok(json!({
                    "ok": true,
                    "parent": parent_id.0,
                    "status": status,
                    "delivered": rendered,
                }))
            }

            "browser" => self.browser_call(false, arguments),

            "test_run" => self.browser_call(true, arguments),

            other => Err(format!("unknown tool {other:?}")),
        }
    }
}

impl RegistryHost {
    /// The session calling these tools, or a clear error when unset.
    fn require_caller(&self) -> Result<String, String> {
        self.caller.clone().ok_or_else(|| {
            format!("this session did not identify itself; {SESSION_ID_ENV} is unset")
        })
    }

    /// `browser` / `test_run`: lazily launch the Playwright sidecar and hand
    /// the request through. Interactive `browser` calls are keyed to the
    /// caller's own session id so pages stay isolated.
    fn browser_call(&self, is_test_run: bool, arguments: &Value) -> Result<Value, String> {
        if !crate::browser::BrowserPool::is_available() {
            return Err("browser pool is unavailable (node or sidecar missing)".to_string());
        }
        let pool = self
            .browser
            .get_or_init(|| crate::browser::BrowserPool::new(&self.logs_dir));
        let mut params = arguments.clone();
        if !is_test_run
            && let Some(caller) = &self.caller
            && let Some(obj) = params.as_object_mut()
        {
            obj.insert("sessionID".into(), Value::String(caller.clone()));
        }
        if is_test_run {
            pool.run(params)
        } else {
            pool.browse(params)
        }
    }

    /// Starts a session on behalf of the calling agent.
    ///
    /// The new session records its caller as `parent`, which is what makes the
    /// lineage tools — `list_children`, `wait_for_children` — mean anything.
    ///
    /// An initial prompt is *not* written here. The agent has not drawn its
    /// input box yet, and typing into a terminal that is still starting loses
    /// the text; delivery waits for readiness, which the caller drives with
    /// `wait_for_agent` then `send_prompt`. The pending prompt is returned so
    /// the caller knows it still owes it.
    fn spawn_agent(&self, arguments: &Value) -> Result<Value, String> {
        let kind = required_str(arguments, "kind")?;
        let cwd = required_str(arguments, "cwd")?;
        if let Some(host) = arguments.get("host").and_then(Value::as_str) {
            return self.spawn_agent_remote(arguments, &kind, &cwd, host);
        }
        let cwd_path = PathBuf::from(&cwd);
        if !cwd_path.is_dir() {
            return Err(format!("cwd {cwd:?} is not a directory"));
        }
        let wants_worktree = arguments
            .get("worktree")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let prompt = arguments
            .get("prompt")
            .and_then(Value::as_str)
            .map(str::to_string);
        let title = arguments
            .get("name")
            .and_then(Value::as_str)
            .map(str::to_string);

        // A worktree is created before the session so the agent starts inside
        // it, rather than in the repo and moving later.
        let (working_dir, worktree_path, branch) = if wants_worktree {
            let info = git::create_worktree(&cwd_path, None, None)
                .map_err(|error| format!("could not create a worktree: {error}"))?;
            let path = PathBuf::from(&info.path);
            (path, Some(info.path), info.branch)
        } else {
            (cwd_path, None, git::branch(Path::new(&cwd)))
        };

        let mut registry = self.registry()?;
        let engine = registry.engine();
        let manifest = engine
            .manifest(&kind)
            .ok_or_else(|| format!("no manifest for agent {kind:?}"))?;
        let descriptor = manifest.agent.clone().unwrap_or_default();
        let authority = descriptor.authority();

        let inherited: Vec<(String, String)> = std::env::vars().collect();
        let pty = descriptor
            .spawn_spec(&working_dir, inherited, &[])
            .ok_or_else(|| {
                format!("agent {kind:?} declares no binary, so it cannot be spawned by name")
            })?;

        let id = crate::control::next_session_id();
        let mut record = crate::control::new_record(&id, &kind, &working_dir.to_string_lossy());
        record.parent = self.caller.clone().map(SessionId);
        record.worktree_path = worktree_path.clone();
        record.git_branch = branch.clone();
        if let Some(title) = title {
            record.title = title;
        }

        let spec = crate::session::SessionSpec {
            id: id.clone(),
            pty,
            manifest_id: kind.clone(),
            authority,
            logs_dir: self.logs_dir.clone(),
            holder: self.holder.clone(),
            remote: None,
            defer_launch: true,
        };
        registry
            .spawn(spec, record)
            .map_err(|error| format!("could not start {kind}: {error}"))?;
        let _ = registry.persist();

        Ok(json!({
            "id": id,
            "kind": kind,
            "cwd": working_dir.to_string_lossy(),
            "worktree": worktree_path,
            "branch": branch,
            "parent": self.caller,
            "pendingPrompt": prompt,
        }))
    }

    /// Remote spawning is not offered by this host. The previous
    /// implementation built a local `ssh … tmux` argv, which the Holder
    /// transport replaced; the equivalent now needs the Helper manager and
    /// binding store that `ControlServer::session_spawn_remote` owns and this
    /// host is not constructed with. Failing here — before a host is resolved
    /// or any code is synced — keeps the path free of external side effects.
    ///
    /// This is a gap, not a removal: `session.spawn` over the control socket
    /// still spawns remotely. Wiring it up means giving `RegistryHost` the
    /// same manager/binding-store dependencies.
    fn spawn_agent_remote(
        &self,
        _arguments: &Value,
        _kind: &str,
        _cwd: &str,
        _host_id: &str,
    ) -> Result<Value, String> {
        Err(format!(
            "{}: {}",
            crate::remote::TRANSPORT_UNAVAILABLE_CODE,
            crate::remote::transport_unavailable().message
        ))
    }

    fn wait_for(&self, id: &str, until: &str, timeout_seconds: f64) -> Result<Value, String> {
        let deadline = Instant::now() + Duration::from_secs_f64(timeout_seconds.max(0.0));
        // "done" means a turn finished: idle *after* having worked. Treating a
        // session that is merely idle as done would return instantly for one
        // that has not started yet.
        let mut has_worked = false;

        loop {
            let status = {
                let registry = self.registry()?;
                let session = registry.get(id).ok_or_else(|| format!("no session {id}"))?;
                session.status()
            };
            if matches!(status, SessionStatus::Working) {
                has_worked = true;
            }

            let reached = match until {
                "done" => has_worked && matches!(status, SessionStatus::Idle),
                "needsInput" => matches!(status, SessionStatus::NeedsInput(_)),
                "exited" => matches!(status, SessionStatus::Exited(_)),
                "any" => !matches!(status, SessionStatus::Starting),
                other => return Err(format!("unknown wait target {other:?}")),
            };
            // A dead session will never reach anything else.
            let dead = matches!(status, SessionStatus::Exited(_));

            if reached || dead {
                return Ok(json!({
                    "id": id,
                    "status": status_word(&status),
                    "reached": reached,
                }));
            }
            if Instant::now() >= deadline {
                return Ok(json!({
                    "id": id,
                    "status": status_word(&status),
                    "reached": false,
                    "timedOut": true,
                }));
            }
            std::thread::sleep(WAIT_POLL);
        }
    }

    fn wait_for_children(&self, caller: &str, timeout_seconds: f64) -> Result<Value, String> {
        let deadline = Instant::now() + Duration::from_secs_f64(timeout_seconds.max(0.0));
        let parent = SessionId(caller.to_string());

        loop {
            let statuses: Vec<(String, SessionStatus)> = {
                let registry = self.registry()?;
                registry
                    .records()
                    .into_iter()
                    .filter(|record| record.parent.as_ref() == Some(&parent))
                    .map(|record| (record.id.0, record.status))
                    .collect()
            };

            let pending: Vec<&String> = statuses
                .iter()
                .filter(|(_, status)| {
                    matches!(status, SessionStatus::Working | SessionStatus::Starting)
                })
                .map(|(id, _)| id)
                .collect();

            if pending.is_empty() || Instant::now() >= deadline {
                return Ok(json!({
                    "children": statuses.iter().map(|(id, status)| json!({
                        "id": id,
                        "status": status_word(status),
                    })).collect::<Vec<_>>(),
                    "allDone": pending.is_empty(),
                }));
            }
            std::thread::sleep(WAIT_POLL);
        }
    }

    /// Where session logs live, for hosts that spawn.
    pub fn logs_dir(&self) -> &Path {
        &self.logs_dir
    }
}
