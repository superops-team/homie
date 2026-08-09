use homie_agents::{AgentCatalog, ResumeStyle, StatusAuthority, load_manifest};
use std::collections::BTreeSet;

const EXPECTED_IDS: [&str; 19] = [
    "claude-code",
    "codex",
    "opencode",
    "gemini",
    "cursor",
    "shell",
    "generic",
    "qoder",
    "pi",
    "kilo",
    "kimi",
    "copilot",
    "kiro",
    "devin",
    "hermes",
    "grok",
    "antigravity",
    "droid",
    "amp",
];

#[test]
fn bundled_catalog_projects_diri_manifest_fields() {
    let catalog = AgentCatalog::new(load_bundled_manifests());
    let ids = catalog
        .ordered()
        .iter()
        .map(|manifest| manifest.id.as_str())
        .collect::<BTreeSet<_>>();
    for expected in EXPECTED_IDS {
        assert!(ids.contains(expected), "missing manifest {expected}");
    }
    assert_eq!(ids.len(), EXPECTED_IDS.len());

    let claude = catalog.descriptor("claude-code");
    assert_eq!(claude.display_name, "Claude Code");
    assert_eq!(claude.short_label, "claude");
    assert_eq!(claude.glyph, "✳");
    assert!(claude.first_class);
    assert_eq!(claude.status_authority, StatusAuthority::Hooks);
    assert_eq!(claude.binary.as_deref(), Some("claude"));
    assert_eq!(claude.session_id_flag.as_deref(), Some("--session-id"));
    assert!(claude.injection.claude_hooks);
    assert!(claude.injection.claude_mcp);
    assert_eq!(
        claude.env.get("CLAUDE_CODE_NO_FLICKER").map(String::as_str),
        Some("1")
    );
    assert_eq!(claude.env_scrub_prefixes, vec!["CLAUDE"]);
    assert_eq!(claude.foreground_exec_names, vec!["claude"]);

    let codex = catalog.descriptor("codex");
    assert_eq!(codex.status_authority, StatusAuthority::Screen);
    assert_eq!(codex.binary.as_deref(), Some("codex"));
    assert!(codex.injection.codex_notify);
    assert!(codex.injection.codex_mcp);
    assert_eq!(
        codex.resume.as_ref().map(|spec| spec.style),
        Some(ResumeStyle::Subcommand)
    );

    let amp = catalog.descriptor("amp");
    assert_eq!(amp.binary.as_deref(), Some("amp"));
    assert!(amp.first_class);
    assert!(!amp.can_resume());
}

#[test]
fn catalog_resolves_aliases_and_falls_back_for_unknown_ids() {
    let catalog = AgentCatalog::new(load_bundled_manifests());

    assert_eq!(
        catalog
            .resolve("claude")
            .map(|manifest| manifest.id.as_str()),
        Some("claude-code")
    );
    assert_eq!(
        catalog
            .resolve("Cursor-Agent")
            .map(|manifest| manifest.id.as_str()),
        Some("cursor")
    );
    assert_eq!(
        catalog
            .resolve("open-code")
            .map(|manifest| manifest.id.as_str()),
        Some("opencode")
    );
    assert!(catalog.resolve("not-an-agent").is_none());

    let ghost = catalog.descriptor("not-an-agent");
    assert_eq!(ghost.id, "not-an-agent");
    assert_eq!(ghost.display_name, "Not An Agent");
    assert_eq!(ghost.status_authority, StatusAuthority::Process);
    assert_eq!(ghost.binary, None);
    assert!(!ghost.first_class);
    assert!(ghost.approve.is_none());
    assert!(ghost.deny.is_none());
    assert!(!ghost.can_resume());
}

#[test]
fn readiness_projects_launchable_agents_only() {
    let catalog = AgentCatalog::new(load_bundled_manifests());
    let readiness = catalog.readiness_with_resolver(|binary| {
        if binary == "codex" {
            Some("/opt/homebrew/bin/codex".into())
        } else {
            None
        }
    });

    assert_eq!(readiness.agents.len(), 17);
    assert!(readiness.agent("shell").is_none());
    assert!(readiness.agent("generic").is_none());

    let codex = readiness.agent("codex").expect("codex readiness");
    assert_eq!(codex.binary, "codex");
    assert_eq!(codex.path.as_deref(), Some("/opt/homebrew/bin/codex"));
    assert!(codex.available);
    assert_eq!(codex.descriptor.display_name, "Codex");

    let claude = readiness.agent("claude-code").expect("claude readiness");
    assert_eq!(claude.binary, "claude");
    assert_eq!(claude.path, None);
    assert!(!claude.available);
}

#[test]
fn resume_specs_preserve_diri_resume_semantics() {
    let catalog = AgentCatalog::new(load_bundled_manifests());

    let claude = catalog.descriptor("claude-code");
    assert!(claude.can_resume());
    assert_eq!(
        claude.resume_argv("session-1"),
        Some(vec!["--resume".into(), "session-1".into()])
    );

    let codex = catalog.descriptor("codex");
    assert!(codex.can_resume());
    assert_eq!(
        codex.resume_argv("thread-1"),
        Some(vec!["resume".into(), "thread-1".into()])
    );

    let cursor = catalog.descriptor("cursor");
    assert!(cursor.can_resume());
    assert_eq!(cursor.resume_argv("ignored"), Some(vec!["resume".into()]));

    for (id, token) in [
        ("antigravity", "--continue"),
        ("copilot", "--continue"),
        ("devin", "--continue"),
        ("droid", "--resume"),
        ("grok", "--continue"),
        ("hermes", "--continue"),
        ("kilo", "--continue"),
        ("kimi", "--continue"),
        ("kiro", "--resume"),
        ("opencode", "--continue"),
        ("pi", "-c"),
        ("qoder", "-c"),
    ] {
        let descriptor = catalog.descriptor(id);
        assert!(descriptor.can_resume(), "{id} lost latest-session resume");
        assert_eq!(descriptor.resume_argv("ignored"), Some(vec![token.into()]));
    }
}

#[test]
fn every_bundled_manifest_decodes_strictly() {
    let mut count = 0usize;
    for path in manifest_paths() {
        count += 1;
        let bytes = std::fs::read(&path).expect("read manifest");
        let manifest =
            serde_json::from_slice::<homie_agents::Manifest>(&bytes).unwrap_or_else(|error| {
                panic!("{} failed strict manifest decode: {error}", path.display())
            });
        assert_eq!(
            manifest.id,
            path.file_stem().unwrap().to_string_lossy(),
            "{} declares mismatched id",
            path.display()
        );
        if manifest.status_model != homie_agents::StatusModel::ProcessOnly {
            assert!(
                !manifest.rules.is_empty(),
                "{} is full status but has no rules",
                manifest.id
            );
        }
        assert!(
            manifest.agent.is_some(),
            "{} missing agent block",
            manifest.id
        );
    }
    assert_eq!(count, EXPECTED_IDS.len());
}

#[test]
fn bundled_manifest_catalog_contains_reference_agents() {
    let manifests = load_bundled_manifests();
    let ids = manifests
        .iter()
        .map(|manifest| manifest.id.as_str())
        .collect::<BTreeSet<_>>();
    for expected in EXPECTED_IDS {
        assert!(ids.contains(expected), "missing manifest {expected}");
    }
    assert_eq!(ids.len(), EXPECTED_IDS.len());
}

#[test]
fn first_class_agents_do_not_fall_back_to_process_only() {
    let manifests = load_bundled_manifests();
    for id in ["claude-code", "codex", "opencode", "gemini", "cursor"] {
        let manifest = manifests
            .iter()
            .find(|manifest| manifest.id == id)
            .expect("manifest");
        assert!(manifest.first_class, "{id} should be first class");
        assert_ne!(
            manifest.status_authority,
            StatusAuthority::Process,
            "{id} must not be process-only"
        );
    }
}

#[test]
fn approval_and_resume_capabilities_are_data_driven() {
    let manifests = load_bundled_manifests();
    let codex = manifests
        .iter()
        .find(|manifest| manifest.id == "codex")
        .expect("codex manifest");
    assert!(codex.resume.is_some());
    assert!(codex.approve.is_some());
    assert!(codex.deny.is_some());

    let claude = manifests
        .iter()
        .find(|manifest| manifest.id == "claude-code")
        .expect("claude manifest");
    assert_eq!(claude.status_authority, StatusAuthority::Hooks);
    assert!(claude.resume.is_some());
}

fn load_bundled_manifests() -> Vec<homie_agents::AgentManifest> {
    manifest_paths()
        .into_iter()
        .map(|path| {
            let bytes = std::fs::read(&path).expect("read manifest");
            load_manifest(&bytes).unwrap_or_else(|error| {
                panic!("{} failed to load: {error}", path.display());
            })
        })
        .collect()
}

fn manifest_paths() -> Vec<std::path::PathBuf> {
    let mut paths = std::fs::read_dir(manifest_dir())
        .expect("read manifest dir")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn manifest_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("agent-descriptors")
}
