use super::*;

#[test]
fn selecting_a_session_wakes_its_artifact_refresh_even_when_already_seen() {
    let (mut store, mut effects) = hydrated(
        vec![session("one", "a", 2.0), session("two", "a", 1.0)],
        vec![project("a", "A")],
        Prefs::default(),
    );
    drain(&mut effects);

    store.select(id("two"));
    assert!(
        drain(&mut effects)
            .into_iter()
            .any(|effect| matches!(effect, StoreEffect::MarkSeen(session) if session == id("two")))
    );
}

#[test]
fn hydrate_restores_the_last_selected_session_instead_of_the_first() {
    let prefs = Prefs {
        last_selected_session: Some(id("two")),
        ..Prefs::default()
    };
    let (store, _) = hydrated(
        vec![session("one", "a", 2.0), session("two", "a", 1.0)],
        vec![project("a", "A")],
        prefs,
    );

    assert_eq!(store.selected_session_id(), Some(&id("two")));
    assert!(store.terminal_residency().contains(&id("two")));
}

#[test]
fn stale_restored_selection_falls_back_and_is_replaced() {
    let prefs = Prefs {
        last_selected_session: Some(id("gone")),
        ..Prefs::default()
    };
    let (store, _) = hydrated(
        vec![session("one", "a", 1.0), session("two", "a", 2.0)],
        vec![project("a", "A")],
        prefs,
    );

    assert_eq!(store.selected_session_id(), Some(&id("one")));
    assert_eq!(store.preferences().last_selected_session, Some(id("one")));
}

#[test]
fn multi_select_matches_finder_command_and_visible_shift_ranges() {
    let mut archived = session("archived", "p", 0.0);
    archived.archived_at = Some(DateMillis(10.0));
    let prefs = Prefs {
        sidebar_expanded_archives: vec![pid("p")],
        ..Prefs::default()
    };
    let (mut store, _) = hydrated(
        vec![
            session("one", "p", 1.0),
            session("two", "p", 2.0),
            session("three", "p", 3.0),
            archived,
        ],
        vec![project("p", "Project")],
        prefs,
    );
    store.select(id("one"));

    store.sidebar_click(
        id("three"),
        ClickModifiers {
            command: true,
            shift: false,
        },
    );
    assert_eq!(
        store.sidebar_selection,
        HashSet::from([id("one"), id("three")])
    );
    assert_eq!(store.selected_session_id, Some(id("one")));

    store.sidebar_click(
        id("archived"),
        ClickModifiers {
            command: false,
            shift: true,
        },
    );
    assert_eq!(
        store.sidebar_selection,
        HashSet::from([id("three"), id("archived")])
    );

    store.sidebar_click(id("two"), ClickModifiers::default());
    assert!(store.sidebar_selection.is_empty());
    assert_eq!(store.selected_session_id, Some(id("two")));
}

#[test]
fn focus_neighbor_prefers_same_project_below_then_above_then_global() {
    let records = vec![
        session("a-top", "a", 1.0),
        session("a-mid", "a", 2.0),
        session("a-low", "a", 3.0),
        session("b-top", "b", 4.0),
    ];
    let projects = vec![project("a", "A"), project("b", "B")];

    let (mut below, _) = hydrated(records.clone(), projects.clone(), Prefs::default());
    below.select(id("a-mid"));
    below.focus_neighbor(&HashSet::from([id("a-mid")]));
    assert_eq!(below.selected_session_id, Some(id("a-low")));

    let (mut above, _) = hydrated(records.clone(), projects.clone(), Prefs::default());
    above.select(id("a-low"));
    above.focus_neighbor(&HashSet::from([id("a-low")]));
    assert_eq!(above.selected_session_id, Some(id("a-mid")));

    let (mut global_below, _) = hydrated(records.clone(), projects.clone(), Prefs::default());
    global_below.select(id("a-mid"));
    global_below.focus_neighbor(&HashSet::from([id("a-top"), id("a-mid"), id("a-low")]));
    assert_eq!(global_below.selected_session_id, Some(id("b-top")));

    let (mut global_above, _) = hydrated(records, projects, Prefs::default());
    global_above.select(id("b-top"));
    global_above.focus_neighbor(&HashSet::from([id("b-top")]));
    assert_eq!(global_above.selected_session_id, Some(id("a-low")));
}

#[test]
fn selection_drives_mru_and_residency_eviction_signals_detach() {
    let (mut store, mut effects) = hydrated(
        vec![
            session("one", "p", 1.0),
            session("two", "p", 2.0),
            session("three", "p", 3.0),
            session("four", "p", 4.0),
        ],
        vec![project("p", "P")],
        Prefs::default(),
    );
    drain(&mut effects);
    store.select(id("two"));
    store.select(id("three"));
    store.select(id("four"));
    assert_eq!(
        store.mru_sessions(),
        vec![id("four"), id("three"), id("two"), id("one")]
    );
    assert!(drain(&mut effects).contains(&StoreEffect::DetachAttachment(id("one"))));

    let mut residency = TerminalResidency::new(3);
    residency.touch(id("a"));
    residency.touch(id("b"));
    residency.touch(id("c"));
    let update = residency.touch(id("d"));
    assert_eq!(update.evicted, Some(id("a")));
    assert_eq!(update.resident, vec![id("d"), id("c"), id("b")]);
}

#[test]
fn default_residency_keeps_only_the_visible_terminal_attached() {
    let mut residency = TerminalResidency::default();
    residency.touch(id("visible"));
    let update = residency.touch(id("next"));

    assert_eq!(update.evicted, Some(id("visible")));
    assert_eq!(update.resident, vec![id("next")]);
}

#[test]
fn spawn_response_focuses_session_when_event_arrives_first() {
    let (mut store, mut effects) = hydrated(
        vec![session("existing", "p", 1.0)],
        vec![project("p", "P")],
        Prefs::default(),
    );
    drain(&mut effects);

    // The daemon can publish session.updated before the spawn RPC response.
    // At event time the old tab is still selected, so the new record is not
    // granted terminal residency yet.
    store.upsert_session(session("spawned", "p", 2.0));
    assert!(!store.terminal_residency().contains(&id("spawned")));

    // Applying the later RPC result must do everything a real tab selection
    // does; otherwise the pane remains on "Preparing terminal…" until clicked.
    store.apply_spawn_result(id("spawned"));

    assert_eq!(store.selected_session_id(), Some(&id("spawned")));
    assert!(store.terminal_residency().contains(&id("spawned")));
}

#[test]
fn synthetic_events_upsert_project_and_remove_with_neighbor_focus() {
    let (mut store, _) = hydrated(
        vec![session("one", "p", 1.0), session("two", "p", 2.0)],
        vec![],
        Prefs::default(),
    );
    store.select(id("one"));
    store.handle_event(EventEnvelope {
        name: homie_proto::EventName::SESSION_UPDATED.to_owned(),
        seq: 1,
        params: serde_json::to_value(session("three", "p", 0.0)).unwrap(),
    });
    assert!(store.sessions.contains_key(&id("three")));

    let updated_project = project("p", "Renamed");
    store.handle_event(EventEnvelope {
        name: homie_proto::EventName::PROJECT_UPDATED.to_owned(),
        seq: 2,
        params: serde_json::to_value(updated_project).unwrap(),
    });
    assert_eq!(
        store.sidebar_projection().projects[0].project.name,
        "Renamed"
    );

    store.handle_event(EventEnvelope {
        name: homie_proto::EventName::SESSION_REMOVED.to_owned(),
        seq: 3,
        params: serde_json::json!({"id": "one"}),
    });
    assert_eq!(store.selected_session_id, Some(id("two")));
    assert!(!store.sessions.contains_key(&id("one")));
}

#[test]
fn identical_or_unrelated_daemon_events_do_not_publish_ui_changes() {
    let existing = session("one", "p", 1.0);
    let (mut store, _) = hydrated(
        vec![existing.clone()],
        vec![project("p", "Project")],
        Prefs::default(),
    );

    assert!(!store.handle_event(EventEnvelope {
        name: homie_proto::EventName::SESSION_UPDATED.to_owned(),
        seq: 1,
        params: serde_json::to_value(existing.clone()).unwrap(),
    }));
    assert!(!store.handle_event(EventEnvelope {
        name: "terminal.grid".to_owned(),
        seq: 2,
        params: serde_json::json!({}),
    }));

    let mut changed = existing;
    changed.title = "Renamed".to_owned();
    assert!(store.handle_event(EventEnvelope {
        name: homie_proto::EventName::SESSION_UPDATED.to_owned(),
        seq: 3,
        params: serde_json::to_value(changed).unwrap(),
    }));
}

#[test]
fn compact_resource_events_patch_only_resource_fields() {
    let existing = session("one", "p", 1.0);
    let original_title = existing.title.clone();
    let (mut store, _) = hydrated(
        vec![existing],
        vec![project("p", "Project")],
        Prefs::default(),
    );

    assert!(store.handle_event(EventEnvelope {
        name: homie_proto::EventName::SESSION_RESOURCES.to_owned(),
        seq: 1,
        params: serde_json::json!({"id":"one","memoryBytes":42000000}),
    }));

    let patched = store.sessions().get(&id("one")).unwrap();
    assert_eq!(patched.memory_bytes, Some(42_000_000));
    assert_eq!(patched.title, original_title);
}

#[test]
fn background_resource_samples_do_not_wake_views() {
    assert_eq!(
        event_publication_policy(StoreEventChange::Resources, false),
        (false, false)
    );
    assert_eq!(
        event_publication_policy(StoreEventChange::Model, false),
        (true, false),
        "model changes still keep the menu snapshot current"
    );
    assert_eq!(
        event_publication_policy(StoreEventChange::Resources, true),
        (true, true)
    );
}
