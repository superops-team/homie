use std::fs;

use homie_remote::{
    HostEntry, PrefsSyncToolReport, mkdir_argv, prefs_sync_specs, present_sync_items, rsync_argv,
    rsync_failure_message,
};
use tempfile::TempDir;

#[test]
fn prefs_sync_uses_fixed_include_list_and_never_credentials() {
    let home = TempDir::new().expect("home");
    write(home.path(), ".claude/CLAUDE.md");
    write(home.path(), ".claude/settings.json");
    write_dir(home.path(), ".claude/commands");
    write_dir(home.path(), ".claude/skills");
    write(home.path(), ".claude/.credentials.json");
    write_dir(home.path(), ".claude/projects");
    write_dir(home.path(), ".claude/todos");
    write(home.path(), ".codex/config.toml");
    write(home.path(), ".codex/auth.json");

    let specs = prefs_sync_specs(home.path());
    let claude = specs.iter().find(|spec| spec.name == "claude").unwrap();
    let codex = specs.iter().find(|spec| spec.name == "codex").unwrap();
    let claude_items = present_sync_items(claude);
    let codex_items = present_sync_items(codex);

    assert_eq!(
        claude_items,
        ["CLAUDE.md", "settings.json", "commands", "skills"]
    );
    assert_eq!(codex_items, ["config.toml"]);

    let host = host();
    let rsyncs = [
        rsync_argv(&host, claude, &claude_items),
        rsync_argv(&host, codex, &codex_items),
    ];
    let flat = rsyncs
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!flat.contains("credentials"));
    assert!(!flat.contains("auth.json"));
    assert!(!flat.contains("projects"));
    assert!(!flat.contains("todos"));
    assert!(!flat.contains("--delete"));
    assert!(rsyncs[0].contains(&"-a".to_string()));
    assert!(rsyncs[0].contains(&"--timeout=60".to_string()));
    assert_eq!(
        rsyncs[0].last().map(String::as_str),
        Some("you@forge:.claude/")
    );
    assert_eq!(
        rsyncs[1].last().map(String::as_str),
        Some("you@forge:.codex/")
    );

    let mkdir = mkdir_argv(&host, claude);
    assert_eq!(mkdir.first().map(String::as_str), Some("ssh"));
    assert!(mkdir.iter().any(|arg| arg.contains("ConnectTimeout=10")));
    assert!(mkdir.iter().any(|arg| arg.contains("mkdir -p .claude")));
}

#[test]
fn prefs_sync_empty_config_is_success_without_commands() {
    let home = TempDir::new().expect("home");
    write(home.path(), ".claude/CLAUDE.md");
    let specs = prefs_sync_specs(home.path());
    let codex = specs.iter().find(|spec| spec.name == "codex").unwrap();
    let items = present_sync_items(codex);

    assert!(items.is_empty());
    let report = PrefsSyncToolReport::skipped(codex.name.clone());
    assert!(report.ok);
    assert!(report.synced.is_empty());
    assert_eq!(report.error, None);
}

#[test]
fn maps_missing_remote_rsync_to_clear_error() {
    let message = rsync_failure_message(127, "bash: rsync: command not found", "Forge");
    assert!(message.contains("rsync is not installed on Forge"));

    let generic = rsync_failure_message(12, "protocol error", "Forge");
    assert_eq!(generic, "rsync failed (exit 12): protocol error");
}

fn host() -> HostEntry {
    HostEntry {
        id: "forge".to_string(),
        name: Some("Forge".to_string()),
        ssh: "you@forge".to_string(),
        default_cwd: Some("~/code".to_string()),
        node: None,
    }
}

fn write(root: &std::path::Path, relative: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).expect("parent");
    fs::write(path, "x").expect("write");
}

fn write_dir(root: &std::path::Path, relative: &str) {
    fs::create_dir_all(root.join(relative)).expect("dir");
}
