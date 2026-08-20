use super::*;

#[test]
fn prefs_round_trip_and_zoom_clamp() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("nested/prefs.json");
    let prefs = Prefs {
        default_agent: DefaultAgent::Gemini,
        default_spawn_host: Some("forge".to_owned()),
        terminal_font_size: 19.5,
        window_placement: Some(WindowPlacement {
            display_uuid: Some("display-one".to_owned()),
            mode: WindowMode::Fullscreen,
            x: 120.0,
            y: 80.0,
            width: 1440.0,
            height: 900.0,
        }),
        sidebar_visible: false,
        sidebar_width: 284.0,
        inspector_width: 516.0,
        inspector_tab: InspectorTab::Artifacts,
        last_selected_session: Some(id("s")),
        quick_open_roots: "~/fun\n~/src".to_owned(),
        sidebar_project_order: vec![pid("p")],
        sidebar_pinned_sessions: vec![id("s")],
        ..Prefs::default()
    };
    prefs.save(&path).unwrap();
    assert_eq!(Prefs::load(&path).unwrap(), prefs);

    let (mut store, _) = SessionStore::load(&path).unwrap();
    store.zoom_terminal(100.0).unwrap();
    assert_eq!(store.prefs.terminal_font_size, 20.0);
    store.zoom_terminal(-100.0).unwrap();
    assert_eq!(store.prefs.terminal_font_size, 10.0);
    store.reset_terminal_zoom().unwrap();
    assert_eq!(Prefs::load(&path).unwrap().terminal_font_size, 13.0);
}

#[test]
fn legacy_last_spawn_host_migrates_to_the_explicit_default() {
    let prefs: Prefs = serde_json::from_str(r#"{"lastSpawnHost":"forge"}"#).unwrap();
    assert_eq!(prefs.default_spawn_host.as_deref(), Some("forge"));
}

#[test]
fn selected_session_persists_across_store_reloads() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("prefs.json");
    Prefs::default().save(&path).unwrap();

    let (mut store, _) = SessionStore::load(&path).unwrap();
    store.hydrate(SessionListResult {
        sessions: vec![session("one", "a", 2.0), session("two", "a", 1.0)],
        projects: vec![project("a", "A")],
    });
    store.select(id("two"));
    drop(store);

    let (mut restored, _) = SessionStore::load(&path).unwrap();
    restored.hydrate(SessionListResult {
        sessions: vec![session("one", "a", 2.0), session("two", "a", 1.0)],
        projects: vec![project("a", "A")],
    });
    assert_eq!(restored.selected_session_id(), Some(&id("two")));
}

#[test]
fn remote_spawn_uses_host_default_cwd_and_drops_worktree() {
    let (mut store, mut effects) = hydrated(
        vec![session("one", "p", 1.0)],
        vec![project("p", "Project")],
        Prefs::default(),
    );
    store.set_hosts(vec![homie_proto::HostEntry {
        id: "forge".into(),
        name: Some("Forge".into()),
        ssh: "cristi@forge".into(),
        default_cwd: Some("~/code".into()),
        node: None,
    }]);
    store.select(id("one"));
    drain(&mut effects);

    fn spawn_params(
        effects: &mut mpsc::UnboundedReceiver<StoreEffect>,
    ) -> homie_proto::SessionSpawnParams {
        let spawned = drain(effects);
        match spawned.first() {
            Some(StoreEffect::Spawn(params)) => params.clone(),
            other => panic!("expected spawn effect, got {other:?}"),
        }
    }

    // Host set + no explicit cwd: the host's defaultCwd wins over the selected
    // session's LOCAL directory, and worktree options are dropped entirely.
    store.spawn_kind(
        AgentKind::CLAUDE_CODE,
        super::super::SpawnOptions {
            host: Some("forge".into()),
            worktree: Some(super::super::WorktreeSpawn {
                create: true,
                branch: None,
            }),
            ..super::super::SpawnOptions::default()
        },
    );
    let params = spawn_params(&mut effects);
    assert_eq!(params.host.as_deref(), Some("forge"));
    assert_eq!(params.cwd, "~/code");
    assert_eq!(params.new_worktree, None);

    // Explicit remote override beats the default cwd.
    store.spawn_kind(
        AgentKind::SHELL,
        super::super::SpawnOptions {
            host: Some("forge".into()),
            cwd: Some("~/deploys".into()),
            ..super::super::SpawnOptions::default()
        },
    );
    assert_eq!(spawn_params(&mut effects).cwd, "~/deploys");

    // Unknown host id (stale picker) still spawns, in the remote home.
    store.spawn_kind(
        AgentKind::SHELL,
        super::super::SpawnOptions {
            host: Some("gone".into()),
            ..super::super::SpawnOptions::default()
        },
    );
    assert_eq!(spawn_params(&mut effects).cwd, "~");

    // Local spawns are untouched: selected session's directory, no host.
    store.spawn_kind(AgentKind::SHELL, super::super::SpawnOptions::default());
    let params = spawn_params(&mut effects);
    assert_eq!(params.host, None);
    assert_eq!(params.cwd, "/work/p");

    // Host badge lookup falls back to the raw id when the entry is gone.
    assert_eq!(store.host_display_name("forge"), "Forge");
    assert_eq!(store.host_display_name("gone"), "gone");
}

#[test]
fn new_agent_target_uses_the_configured_default_instead_of_the_selected_session() {
    let (mut store, mut effects) = hydrated(
        vec![session("local", "p", 1.0)],
        vec![project("p", "P")],
        Prefs {
            default_spawn_host: Some("forge".into()),
            ..Prefs::default()
        },
    );
    store.set_hosts(vec![homie_proto::HostEntry {
        id: "forge".into(),
        name: Some("Forge".into()),
        ssh: "cristi@forge".into(),
        default_cwd: Some("~/code".into()),
        node: None,
    }]);
    store.select(id("local"));
    drain(&mut effects);

    assert_eq!(
        store.begin_repo_targeting().as_deref(),
        Some("forge"),
        "the picker should use the configured shortcut destination"
    );
}

#[test]
fn top_level_shortcuts_spawn_on_the_configured_default_host() {
    let (mut store, mut effects) = hydrated(
        vec![session("local", "p", 1.0)],
        vec![project("p", "P")],
        Prefs {
            default_spawn_host: Some("forge".into()),
            ..Prefs::default()
        },
    );
    store.set_hosts(vec![homie_proto::HostEntry {
        id: "forge".into(),
        name: Some("Forge".into()),
        ssh: "cristi@forge".into(),
        default_cwd: None,
        node: None,
    }]);
    drain(&mut effects);

    store.spawn_default(super::super::SpawnOptions::default());

    let params = match effects.try_recv() {
        Ok(StoreEffect::Spawn(params)) => params,
        other => panic!("expected spawn effect, got {other:?}"),
    };
    assert_eq!(params.host.as_deref(), Some("forge"));
    assert_eq!(params.cwd, "~");
}

#[test]
fn selecting_a_default_host_persists_across_store_reloads() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("prefs.json");
    Prefs::default().save(&path).unwrap();
    let (mut store, _effects) = SessionStore::load(&path).unwrap();
    store.set_hosts(vec![homie_proto::HostEntry {
        id: "forge".into(),
        name: Some("Forge".into()),
        ssh: "cristi@forge".into(),
        default_cwd: Some("~/code".into()),
        node: None,
    }]);

    store.set_default_spawn_host(Some("forge".into()));

    assert_eq!(
        Prefs::load(&path).unwrap().default_spawn_host.as_deref(),
        Some("forge")
    );
}

#[test]
fn a_remote_default_host_round_trips_back_to_local() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("prefs.json");
    Prefs::default().save(&path).unwrap();
    let (mut store, mut effects) = SessionStore::load(&path).unwrap();
    store.set_hosts(vec![homie_proto::HostEntry {
        id: "forge".into(),
        name: Some("Forge".into()),
        ssh: "cristi@forge".into(),
        default_cwd: Some("~/code".into()),
        node: None,
    }]);
    store.set_default_spawn_host(Some("forge".into()));
    assert_eq!(store.default_spawn_host().as_deref(), Some("forge"));

    store.set_default_spawn_host(None);

    assert_eq!(store.default_spawn_host(), None);
    assert_eq!(Prefs::load(&path).unwrap().default_spawn_host, None);
    drain(&mut effects);
    store.spawn_default(super::super::SpawnOptions::default());
    let params = match effects.try_recv() {
        Ok(StoreEffect::Spawn(params)) => params,
        other => panic!("expected spawn effect, got {other:?}"),
    };
    assert_eq!(params.host, None, "⌘T must be local again after a reset");

    let (reloaded, _effects) = SessionStore::load(&path).unwrap();
    assert_eq!(reloaded.default_spawn_host(), None);
}

#[test]
fn migrate_session_guards_kind_target_and_reentry() {
    let mut remote = session("two", "p", 2.0);
    remote.host = Some("forge".into());
    let mut shell = session("three", "p", 1.0);
    shell.kind = AgentKind::SHELL;
    let (mut store, mut effects) = hydrated(
        vec![session("one", "p", 3.0), remote, shell],
        vec![project("p", "P")],
        Prefs::default(),
    );
    drain(&mut effects);

    // Local Claude → forge emits exactly one migrate effect; a second click
    // while in flight is swallowed.
    store.migrate_session(id("one"), Some("forge".into()));
    store.migrate_session(id("one"), Some("forge".into()));
    let emitted = drain(&mut effects);
    assert_eq!(
        emitted,
        vec![StoreEffect::Migrate {
            id: id("one"),
            target_host: Some("forge".into()),
        }]
    );
    assert!(store.migrating().contains(&id("one")));
    store.finish_migration(&id("one"));
    assert!(!store.migrating().contains(&id("one")));

    // No-op moves (already there), non-Claude kinds, and unknown sessions
    // never emit.
    store.migrate_session(id("two"), Some("forge".into()));
    store.migrate_session(id("three"), None);
    store.migrate_session(id("missing"), None);
    assert!(drain(&mut effects).is_empty());

    // Remote Claude → local is eligible.
    store.migrate_session(id("two"), None);
    assert_eq!(
        drain(&mut effects),
        vec![StoreEffect::Migrate {
            id: id("two"),
            target_host: None,
        }]
    );
}

#[test]
fn sync_prefs_emits_once_per_host_until_finished() {
    let (mut store, mut effects) = hydrated(vec![], vec![], Prefs::default());
    store.set_hosts(vec![homie_proto::HostEntry {
        id: "forge".into(),
        name: Some("Forge".into()),
        ssh: "cristi@forge".into(),
        default_cwd: None,
        node: None,
    }]);
    drain(&mut effects);

    store.sync_prefs("forge".into());
    store.sync_prefs("forge".into());
    assert_eq!(
        drain(&mut effects),
        vec![StoreEffect::SyncPrefs {
            host: "forge".into(),
            host_name: "Forge".into(),
        }]
    );
    assert!(store.syncing_prefs().contains("forge"));
    store.finish_prefs_sync("forge");
    store.sync_prefs("forge".into());
    assert_eq!(drain(&mut effects).len(), 1);
}

#[test]
fn repo_targeting_tracks_the_selected_session_and_dedupes_requests() {
    let mut remote = session("one", "p", 2.0);
    remote.host = Some("forge".into());
    let (mut store, mut effects) = hydrated(
        vec![remote, session("two", "p", 1.0)],
        vec![project("p", "P")],
        Prefs {
            default_spawn_host: Some("forge".into()),
            ..Prefs::default()
        },
    );
    store.set_hosts(vec![homie_proto::HostEntry {
        id: "forge".into(),
        name: Some("Forge".into()),
        ssh: "cristi@forge".into(),
        default_cwd: None,
        node: None,
    }]);
    store.select(id("one"));
    drain(&mut effects);

    // Opening the picker restarts repo targeting against the selected session
    // and returns the remembered spawn destination.
    assert_eq!(store.begin_repo_targeting().as_deref(), Some("forge"));
    store.request_repo_target(None);
    store.request_repo_target(None); // deduped while pending
    assert_eq!(
        drain(&mut effects),
        vec![StoreEffect::LocateRepo {
            key: "local".into(),
            host: None,
            session_id: id("one"),
        }]
    );
    assert_eq!(
        store.repo_target(None),
        Some(&super::super::RepoTarget::Pending)
    );

    // The async answer lands under the same key.
    store.set_repo_target(
        "local".into(),
        super::super::RepoTarget::Resolved("/work/p".into()),
    );
    assert_eq!(
        store.repo_target(None),
        Some(&super::super::RepoTarget::Resolved("/work/p".into()))
    );

    // Selection changes the repo reference, not the remembered destination.
    store.select(id("two"));
    assert_eq!(store.begin_repo_targeting().as_deref(), Some("forge"));
    assert_eq!(store.repo_target(None), None);
}
