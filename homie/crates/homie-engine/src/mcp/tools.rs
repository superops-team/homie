//! The tool surface, as data.
//!
//! Descriptions are written for a model to read, not a human: they say *when*
//! to reach for a tool, because that is what decides whether an agent uses it
//! correctly. Schemas are JSON Schema objects, as MCP requires.
//!
//! Ported from the Swift `HomieMCPTools`. The spawnable-agent list is filled
//! in from the manifest catalog rather than written out, so a new agent
//! manifest becomes spawnable over MCP with no code change.

use serde_json::{Value, json};

pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

fn tool(name: &str, description: &str, input_schema: Value) -> ToolDefinition {
    ToolDefinition {
        name: name.to_string(),
        description: description.to_string(),
        input_schema,
    }
}

/// The tool surface with a generic agent list. Prefer
/// [`tool_definitions_for`] when a manifest catalog is available.
pub fn tool_definitions() -> Vec<ToolDefinition> {
    tool_definitions_for(&[
        "claude".into(),
        "codex".into(),
        "cursor".into(),
        "gemini".into(),
        "shell".into(),
    ])
}

/// The tool surface, with `spawn_agent` constrained to `kinds`.
pub fn tool_definitions_for(kinds: &[String]) -> Vec<ToolDefinition> {
    let kind_enum: Vec<Value> = kinds.iter().map(|kind| json!(kind)).collect();

    vec![
        tool(
            "spawn_agent",
            "Open a NEW session (tab) in homie running an agent or a shell. USE THIS whenever the \
             user asks to open, start, spawn or launch another agent, session, or terminal. \
             Optionally create a fresh git worktree for it and give it an initial prompt.",
            json!({
                "type": "object",
                "properties": {
                    "kind": { "type": "string", "enum": kind_enum, "description": "Which agent to run." },
                    "cwd": { "type": "string", "description": "Working directory (a repo path when worktree is true)." },
                    "worktree": { "type": "boolean", "description": "Create a fresh git worktree off cwd and run there (local spawns only)." },
                    "prompt": { "type": "string", "description": "Initial prompt to send once the agent is ready." },
                    "name": { "type": "string", "description": "Session title." }
                },
                "required": ["kind", "cwd"]
            }),
        ),
        tool(
            "list_agents",
            "List all active agent sessions with id, kind, title, status, parent, and cwd.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool(
            "get_status",
            "Current status of one session: working, idle, needsInput (with detail), or exited, \
             plus title and cwd.",
            json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "The session id." }
                },
                "required": ["session_id"]
            }),
        ),
        tool(
            "send_prompt",
            "Send text to a session as if typed, then submit it. Use for follow-up instructions \
             and for answering a prompt the agent is blocked on.",
            json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string" },
                    "text": { "type": "string", "description": "The text to send." },
                    "submit": { "type": "boolean", "description": "Press return afterwards. Defaults to true." }
                },
                "required": ["session_id", "text"]
            }),
        ),
        tool(
            "wait_for_agent",
            "Block until a session reaches a state: done (idle after working), needsInput, or \
             exited. Returns as soon as the state is reached or the timeout elapses.",
            json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string" },
                    "until": { "type": "string", "enum": ["done", "needsInput", "exited", "any"], "description": "What to wait for. Defaults to done." },
                    "timeout_seconds": { "type": "number", "description": "Give up after this long. Defaults to 300." }
                },
                "required": ["session_id"]
            }),
        ),
        tool(
            "read_output",
            "Read a session's recent terminal output. Use after wait_for_agent to see what it did.",
            json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string" },
                    "max_bytes": { "type": "number", "description": "How much to read from the end. Defaults to 8000." }
                },
                "required": ["session_id"]
            }),
        ),
        tool(
            "release_agent",
            "End a session and kill its process tree. The record stays in the list.",
            json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string" }
                },
                "required": ["session_id"]
            }),
        ),
        tool(
            "create_worktree",
            "Create a git worktree off a repository, on a new branch, so parallel work does not \
             collide in one checkout.",
            json!({
                "type": "object",
                "properties": {
                    "repo": { "type": "string", "description": "Path to the repository." },
                    "branch": { "type": "string", "description": "Branch to create. Generated when omitted." },
                    "base": { "type": "string", "description": "Commit or branch to start from." }
                },
                "required": ["repo"]
            }),
        ),
        tool(
            "list_worktrees",
            "List a repository's worktrees with their paths and branches.",
            json!({
                "type": "object",
                "properties": {
                    "repo": { "type": "string" }
                },
                "required": ["repo"]
            }),
        ),
        tool(
            "remove_worktree",
            "Remove a git worktree.",
            json!({
                "type": "object",
                "properties": {
                    "repo": { "type": "string" },
                    "worktree": { "type": "string", "description": "Path of the worktree to remove." },
                    "force": { "type": "boolean", "description": "Remove even with uncommitted changes." }
                },
                "required": ["repo", "worktree"]
            }),
        ),
        tool(
            "whoami",
            "Identify the calling session: its id, kind, title, cwd, and parent if it was spawned \
             by another agent.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool(
            "list_children",
            "List the sessions this one spawned, with their statuses.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool(
            "wait_for_children",
            "Block until every session this one spawned is done, or the timeout elapses.",
            json!({
                "type": "object",
                "properties": {
                    "timeout_seconds": { "type": "number", "description": "Defaults to 600." }
                }
            }),
        ),
        tool(
            "summarize_children",
            "Compact screen tails plus status and artifacts for the sessions you spawned, so you can \
             synthesize their results in one call instead of reading each one's full output. Returns \
             what each delegate actually printed; it does not interpret or conclude for you.",
            json!({
                "type": "object",
                "properties": {
                    "session_ids": { "type": "array", "items": { "type": "string" }, "description": "Subset of your children. Default: all of them." },
                    "rows": { "type": "number", "description": "Non-blank screen lines per child (max 60). Defaults to 14." }
                }
            }),
        ),
        tool(
            "report_to_parent",
            "Hand a STRUCTURED result back to the session that spawned you. Use this instead of \
             send_prompt when you finish, stall, or need a decision from your delegator. Fails if this \
             session has no parent.",
            json!({
                "type": "object",
                "properties": {
                    "summary": { "type": "string", "description": "One-line result or status. Required." },
                    "status": { "type": "string", "enum": ["update", "done", "blocked", "failed"], "description": "Default: update." },
                    "details": { "type": "string", "description": "Anything the parent needs beyond the summary." },
                    "blockers": { "type": "array", "items": { "type": "string" }, "description": "What is stopping you: missing context, approvals, broken deps." },
                    "questions": { "type": "array", "items": { "type": "string" }, "description": "Decisions you need from the parent or user." },
                    "next_steps": { "type": "array", "items": { "type": "string" }, "description": "Suggested follow-up work." },
                    "changed_paths": { "type": "array", "items": { "type": "string" }, "description": "Files you created or modified." },
                    "artifacts": { "type": "array", "items": { "type": "string" }, "description": "PR links, preview URLs, generated file paths." },
                    "proof": { "type": "array", "items": { "type": "string" }, "description": "Commands run, tests passed, evidence the work is real." },
                    "submit": { "type": "boolean", "description": "Press Enter after delivering (default true)." }
                },
                "required": ["summary"]
            }),
        ),
        tool(
            "get_artifacts",
            "PR links, issues, preview URLs and ports captured from a session's output. PR artifacts \
             carry live GitHub stats (state, review decision, mergeability, CI checks, comments) under \
             a 'pr' key.",
            json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "The session id." }
                },
                "required": ["session_id"]
            }),
        ),
        tool(
            "browser",
            "Drive a real browser isolated to THIS session: 'open' a URL, read the returned snapshot \
             for element refs like @e1, act by ref, and read the fresh snapshot every action returns. \
             'screenshot' saves a file and returns its path; 'console' shows page errors.",
            json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["open", "snapshot", "click", "fill", "type", "press", "hover", "select", "check", "scroll", "get", "wait", "screenshot", "console", "back", "close", "list"], "description": "open = navigate and snapshot; snapshot = re-read; click/fill/type/press/hover/select/check/scroll = act returning a fresh snapshot; get = read url|title|text|html|value|count; wait = for a selector/state/ms; screenshot = save image, return path; console = recent errors; back = history back; close = end browser; list = open browsers." },
                    "url": { "type": "string", "description": "For open." },
                    "ref": { "type": "string", "description": "Element handle from the last snapshot, e.g. e3 or @e3." },
                    "selector": { "type": "string", "description": "CSS fallback for what a ref can't express." },
                    "text": { "type": "string", "description": "For fill (replace) and type (keystrokes, append)." },
                    "key": { "type": "string", "description": "For press, e.g. Enter, Tab, Escape." },
                    "value": { "type": "string", "description": "For select (option value); for check pass \"false\" to uncheck." },
                    "what": { "type": "string", "enum": ["url", "title", "text", "html", "value", "count"], "description": "For get." },
                    "ms": { "type": "number", "description": "For wait: fixed delay in milliseconds." },
                    "state": { "type": "string", "description": "For wait: load | domcontentloaded | networkidle, or an element state." },
                    "direction": { "type": "string", "enum": ["up", "down", "left", "right"], "description": "For scroll without a ref/selector." },
                    "amount": { "type": "number", "description": "Scroll distance in pixels (default 600), or console lines (default 50)." },
                    "button": { "type": "string", "enum": ["left", "right", "middle"], "description": "For click." },
                    "double": { "type": "boolean", "description": "Double-click." },
                    "full": { "type": "boolean", "description": "snapshot: include headings and landmarks. screenshot: full page." },
                    "annotate": { "type": "boolean", "description": "screenshot: label every ref in the image." },
                    "engine": { "type": "string", "enum": ["chromium", "webkit", "firefox"], "description": "For open. Default chromium." },
                    "profile": { "type": "string", "description": "For open: named profile persisting cookies and logins." }
                },
                "required": ["action"]
            }),
        ),
        tool(
            "test_run",
            "Run a web feature test flow across real browser engines (chromium, webkit, firefox) via \
             Homie's shared browser pool — no per-call browser spawn. Runs the whole flow in ONE call \
             across engines in parallel, returning per-engine pass/fail with a compact accessibility \
             snapshot, console errors, and a screenshot file path only on failure.",
            json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "The page to test, e.g. http://localhost:3000" },
                    "engines": { "type": "array", "items": { "type": "string", "enum": ["chromium", "webkit", "firefox"] }, "description": "Engines to run across. Default: chromium only." },
                    "steps": { "type": "array", "items": { "type": "object" }, "description": "Ordered one-key step objects (click/type/drag/select/assert/…)." },
                    "observe": { "type": "string", "enum": ["a11y", "screenshot"], "description": "a11y (default) or screenshot-every-step." },
                    "profile": { "type": "string", "description": "Named profile to persist cookies + localStorage across runs." },
                    "auth": { "type": "object", "properties": { "cookies": { "type": "array", "items": { "type": "object" } }, "localStorage": { "type": "object" } }, "description": "Auth hand-off seeded into every engine before first navigation." }
                },
                "required": ["url", "steps"]
            }),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tool_has_a_name_a_description_and_an_object_schema() {
        for tool in tool_definitions() {
            assert!(!tool.name.is_empty());
            assert!(
                tool.description.len() > 20,
                "{} needs a description a model can act on",
                tool.name
            );
            assert_eq!(tool.input_schema["type"], "object", "{}", tool.name);
        }
    }

    #[test]
    fn required_arguments_are_declared_in_properties() {
        // A required key that is not in properties is a schema a strict client
        // rejects outright.
        for tool in tool_definitions() {
            let Some(required) = tool.input_schema["required"].as_array() else {
                continue;
            };
            let properties = tool.input_schema["properties"]
                .as_object()
                .unwrap_or_else(|| panic!("{} has required keys but no properties", tool.name));
            for key in required {
                let key = key.as_str().expect("a string");
                assert!(
                    properties.contains_key(key),
                    "{} requires {key:?} but does not declare it",
                    tool.name
                );
            }
        }
    }

    #[test]
    fn the_spawnable_list_comes_from_the_catalog() {
        let kinds = vec!["opencode".to_string(), "shell".to_string()];
        let tools = tool_definitions_for(&kinds);
        let spawn = tools
            .iter()
            .find(|tool| tool.name == "spawn_agent")
            .expect("spawn_agent");

        let enumerated: Vec<&str> = spawn.input_schema["properties"]["kind"]["enum"]
            .as_array()
            .expect("enum")
            .iter()
            .filter_map(Value::as_str)
            .collect();
        assert_eq!(enumerated, vec!["opencode", "shell"]);
    }

    #[test]
    fn tool_names_are_unique() {
        let mut names: Vec<String> = tool_definitions()
            .into_iter()
            .map(|tool| tool.name)
            .collect();
        let total = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), total, "duplicate tool names");
    }
}
