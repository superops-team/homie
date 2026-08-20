use super::*;

#[test]
fn auto_resume_is_attempted_once_per_run() {
    let mut record = session("restart", "p", 1.0);
    record.status = SessionStatus::Exited(ExitInfo {
        reason: ExitReason::DaemonRestart,
        code: None,
        signal: None,
    });
    record.resumability = Resumability::Resumable;
    let (mut store, mut effects) = hydrated(
        vec![record.clone()],
        vec![project("p", "P")],
        Prefs::default(),
    );

    assert_eq!(
        drain(&mut effects)
            .into_iter()
            .filter(|effect| matches!(
                effect,
                StoreEffect::Resume {
                    automatic: true,
                    ..
                }
            ))
            .count(),
        1
    );
    store.upsert_session(record);
    assert!(!store.auto_resume_if_needed(&id("restart")));
    assert!(drain(&mut effects).is_empty());
}

#[test]
fn cold_boot_only_auto_resumes_the_selected_session() {
    let restart_session = |value: &str, created: f64| {
        let mut record = session(value, "p", created);
        record.status = SessionStatus::Exited(ExitInfo {
            reason: ExitReason::DaemonRestart,
            code: None,
            signal: None,
        });
        record.resumability = Resumability::Resumable;
        record
    };
    let (mut store, mut effects) = hydrated(
        vec![
            restart_session("newest", 3.0),
            restart_session("middle", 2.0),
            restart_session("oldest", 1.0),
        ],
        vec![project("p", "P")],
        Prefs::default(),
    );

    // Nothing was remembered, so the selection falls to the top of the sidebar
    // — which is the oldest session now that rows arrive at the bottom.
    assert_eq!(store.selected_session_id(), Some(&id("oldest")));
    let automatic_resumes: Vec<_> = drain(&mut effects)
        .into_iter()
        .filter_map(|effect| match effect {
            StoreEffect::Resume {
                id,
                automatic: true,
            } => Some(id),
            _ => None,
        })
        .collect();
    assert_eq!(
        automatic_resumes,
        vec![id("oldest")],
        "cold boot must not revive every previously running agent"
    );

    store.select(id("middle"));
    let selected_resumes: Vec<_> = drain(&mut effects)
        .into_iter()
        .filter_map(|effect| match effect {
            StoreEffect::Resume {
                id,
                automatic: true,
            } => Some(id),
            _ => None,
        })
        .collect();
    assert_eq!(
        selected_resumes,
        vec![id("middle")],
        "an offline conversation should revive when the user selects it"
    );
}

#[test]
fn close_confirmation_only_gates_running_sessions() {
    let running = session("running", "p", 2.0);
    let mut exited = session("exited", "p", 1.0);
    exited.status = SessionStatus::Exited(ExitInfo {
        reason: ExitReason::Exited,
        code: Some(0),
        signal: None,
    });
    let (mut store, mut effects) = hydrated(
        vec![running, exited],
        vec![project("p", "P")],
        Prefs::default(),
    );
    drain(&mut effects);

    store.request_close(vec![id("exited")]);
    assert!(store.pending_close.is_none());
    assert!(drain(&mut effects).contains(&StoreEffect::Remove(id("exited"))));

    store.request_close(vec![id("running")]);
    assert_eq!(
        store.pending_close.as_ref().map(|pending| &pending.ids),
        Some(&vec![id("running")])
    );
    assert!(drain(&mut effects).is_empty());
    store.confirm_pending_close();
    assert!(drain(&mut effects).contains(&StoreEffect::Remove(id("running"))));
}

#[test]
fn a_real_process_exit_immediately_detaches_and_removes_the_agent() {
    let (mut store, mut effects) = hydrated(
        vec![session("one", "p", 1.0)],
        vec![project("p", "P")],
        Prefs::default(),
    );
    drain(&mut effects);

    let mut exited = session("one", "p", 1.0);
    exited.status = SessionStatus::Exited(ExitInfo {
        reason: ExitReason::Exited,
        code: Some(0),
        signal: None,
    });
    store.upsert_session(exited);

    let emitted = drain(&mut effects);
    assert!(emitted.contains(&StoreEffect::DetachAttachment(id("one"))));
    assert!(emitted.contains(&StoreEffect::Remove(id("one"))));
    assert!(store.ordered_sessions().is_empty());
}

#[test]
fn a_signalled_agent_keeps_its_row_and_its_scrollback() {
    let (mut store, mut effects) = hydrated(
        vec![session("one", "p", 1.0)],
        vec![project("p", "P")],
        Prefs::default(),
    );
    drain(&mut effects);

    let mut killed = session("one", "p", 1.0);
    killed.status = SessionStatus::Exited(ExitInfo {
        reason: ExitReason::Signaled,
        code: None,
        signal: Some(15),
    });
    store.upsert_session(killed);

    let emitted = drain(&mut effects);
    assert!(
        !emitted.contains(&StoreEffect::Remove(id("one"))),
        "a signalled exit must not delete the session or its log"
    );
    assert_eq!(store.ordered_sessions().len(), 1);
}

#[test]
fn an_exited_but_resumable_agent_stays_listed_for_resume() {
    let (mut store, mut effects) = hydrated(
        vec![session("one", "p", 1.0)],
        vec![project("p", "P")],
        Prefs::default(),
    );
    drain(&mut effects);

    let mut exited = session("one", "p", 1.0);
    exited.status = SessionStatus::Exited(ExitInfo {
        reason: ExitReason::Exited,
        code: Some(0),
        signal: None,
    });
    exited.resumability = Resumability::Resumable;
    store.upsert_session(exited);

    let emitted = drain(&mut effects);
    assert!(
        !emitted.contains(&StoreEffect::Remove(id("one"))),
        "a resumable conversation must stay listed"
    );
    assert_eq!(store.ordered_sessions().len(), 1);
}

#[test]
fn a_nonzero_exit_keeps_its_row() {
    let (mut store, mut effects) = hydrated(
        vec![session("one", "p", 1.0)],
        vec![project("p", "P")],
        Prefs::default(),
    );
    drain(&mut effects);

    let mut crashed = session("one", "p", 1.0);
    crashed.status = SessionStatus::Exited(ExitInfo {
        reason: ExitReason::Exited,
        code: Some(1),
        signal: None,
    });
    store.upsert_session(crashed);

    let emitted = drain(&mut effects);
    assert!(
        !emitted.contains(&StoreEffect::Remove(id("one"))),
        "a failed exit must not delete the session or its log"
    );
    assert_eq!(store.ordered_sessions().len(), 1);
}

#[test]
fn daemon_restart_exit_remains_available_for_automatic_resume() {
    let (mut store, mut effects) = hydrated(
        vec![session("one", "p", 1.0)],
        vec![project("p", "P")],
        Prefs::default(),
    );
    drain(&mut effects);

    let mut restart = session("one", "p", 1.0);
    restart.status = SessionStatus::Exited(ExitInfo {
        reason: ExitReason::DaemonRestart,
        code: None,
        signal: None,
    });
    restart.resumability = Resumability::Resumable;
    store.upsert_session(restart);

    assert!(
        !drain(&mut effects)
            .iter()
            .any(|effect| matches!(effect, StoreEffect::Remove(_)))
    );
}

#[test]
fn stale_directory_responses_cannot_overwrite_newer_navigation() {
    let (mut store, mut effects) = SessionStore::headless(Prefs::default());
    store.request_directory_listing(Some("forge".into()), "/srv".into());
    let first = drain(&mut effects)
        .into_iter()
        .find_map(|effect| match effect {
            StoreEffect::ListDirectories { request_id, .. } => Some(request_id),
            _ => None,
        })
        .expect("first request");
    store.request_directory_listing(Some("forge".into()), "/srv/app".into());
    let second = drain(&mut effects)
        .into_iter()
        .find_map(|effect| match effect {
            StoreEffect::ListDirectories { request_id, .. } => Some(request_id),
            _ => None,
        })
        .expect("second request");

    store.finish_directory_listing(first, Err("stale".into()));
    assert_eq!(
        store.directory_listing(Some("forge"), "/srv/app"),
        Some(&super::super::DirectoryListingState::Loading)
    );
    store.finish_directory_listing(second, Err("latest".into()));
    assert_eq!(
        store.directory_listing(Some("forge"), "/srv/app"),
        Some(&super::super::DirectoryListingState::Error("latest".into()))
    );
}

#[test]
fn a_closing_row_leaves_at_once_and_ignores_further_clicks() {
    let (mut store, mut effects) = hydrated(
        vec![session("one", "p", 2.0), session("two", "p", 1.0)],
        vec![project("p", "P")],
        Prefs {
            confirm_before_closing_session: false,
            ..Prefs::default()
        },
    );
    drain(&mut effects);

    store.request_close(vec![id("one")]);
    // The daemon still has to terminate the process tree, but the row is gone.
    assert_eq!(
        store
            .ordered_sessions()
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>(),
        vec![id("two")]
    );
    assert!(drain(&mut effects).contains(&StoreEffect::Remove(id("one"))));

    // A second ✕ on the same row is a no-op rather than a repeat request.
    store.request_close(vec![id("one")]);
    assert!(drain(&mut effects).is_empty());

    // A resync that still lists the session means the close never landed.
    store.hydrate(SessionListResult {
        sessions: vec![session("one", "p", 2.0), session("two", "p", 1.0)],
        projects: vec![project("p", "P")],
    });
    assert_eq!(store.ordered_sessions().len(), 2);
}

#[test]
fn a_pending_confirmation_hides_the_row_only_once_confirmed() {
    let (mut store, mut effects) = hydrated(
        vec![session("one", "p", 2.0)],
        vec![project("p", "P")],
        Prefs::default(),
    );
    drain(&mut effects);

    store.request_close(vec![id("one")]);
    assert_eq!(store.ordered_sessions().len(), 1);
    store.confirm_pending_close();
    assert!(store.ordered_sessions().is_empty());
}

#[test]
fn auxiliary_terminal_inherits_context_without_becoming_sidebar_selection() {
    let mut primary = session("one", "p", 2.0);
    primary.cwd = "/work/p/subdir".to_owned();
    primary.host = Some("forge".to_owned());
    let (mut store, mut effects) = hydrated(
        vec![primary],
        vec![project("p", "Project")],
        Prefs::default(),
    );
    store.select(id("one"));
    drain(&mut effects);

    assert!(store.spawn_auxiliary_terminal(id("one")));
    let effects = drain(&mut effects);
    let Some(StoreEffect::SpawnAuxiliary(params)) = effects.first() else {
        panic!("expected auxiliary spawn, got {effects:?}");
    };
    assert_eq!(params.kind, AgentKind::SHELL);
    assert_eq!(params.cwd, "/work/p/subdir");
    assert_eq!(params.host.as_deref(), Some("forge"));
    assert_eq!(params.parent, Some(id("one")));
    assert_eq!(store.selected_session_id(), Some(&id("one")));
}

#[test]
fn auxiliary_terminal_is_hidden_and_removed_with_its_parent() {
    let primary = session("one", "p", 2.0);
    let mut terminal = session("terminal", "p", 1.0);
    terminal.kind = AgentKind::SHELL;
    terminal.parent = Some(id("one"));
    terminal.title = super::super::AUXILIARY_TERMINAL_TITLE.to_owned();
    let (mut store, mut effects) = hydrated(
        vec![primary, terminal],
        vec![project("p", "Project")],
        Prefs::default(),
    );
    drain(&mut effects);

    assert_eq!(
        store
            .ordered_sessions()
            .into_iter()
            .map(|session| session.id)
            .collect::<Vec<_>>(),
        vec![id("one")]
    );
    assert_eq!(
        store
            .auxiliary_terminal_for(&id("one"))
            .map(|s| s.id.clone()),
        Some(id("terminal"))
    );

    store.remove_sessions(vec![id("one")]);
    let removed: HashSet<_> = drain(&mut effects)
        .into_iter()
        .filter_map(|effect| match effect {
            StoreEffect::Remove(id) => Some(id),
            _ => None,
        })
        .collect();
    assert_eq!(removed, HashSet::from([id("one"), id("terminal")]));
}
