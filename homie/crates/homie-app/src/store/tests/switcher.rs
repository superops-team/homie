use super::*;

#[test]
fn switcher_store_integration_commits_only_on_control_release() {
    let sessions = vec![
        session("one", "a", 3.0),
        session("two", "a", 2.0),
        session("three", "a", 1.0),
    ];
    let (mut store, _effects) = hydrated(sessions, vec![project("a", "A")], Prefs::default());
    store.select(id("two"));
    store.select(id("three"));

    assert!(store.handle_switcher_key(SwitcherKey::Tab {
        control: true,
        shift: false,
    }));
    assert_eq!(store.selected_session_id(), Some(&id("three")));
    assert_eq!(store.switcher_state().highlighted(), Some(&id("two")));

    assert!(!store.handle_switcher_modifiers_changed(false));
    assert_eq!(store.selected_session_id(), Some(&id("two")));
    assert!(!store.switcher_state().is_visible());

    store.handle_switcher_key(SwitcherKey::Tab {
        control: true,
        shift: true,
    });
    assert_eq!(store.switcher_state().highlighted(), Some(&id("one")));
    assert!(store.handle_switcher_key(SwitcherKey::Escape));
    assert_eq!(store.selected_session_id(), Some(&id("two")));
}

#[test]
fn overview_store_integration_filters_selects_and_bulk_closes() {
    let live = session("live", "a", 1.0);
    let mut ended = session("ended", "a", 2.0);
    ended.status = SessionStatus::Exited(ExitInfo {
        reason: ExitReason::Exited,
        code: Some(0),
        signal: None,
    });
    let (mut store, mut effects) =
        hydrated(vec![live, ended], vec![project("a", "A")], Prefs::default());
    drain(&mut effects);

    store.toggle_overview();
    store.set_overview_filter(OverviewFilter::Lane(OverviewLane::Ended));
    store.select_all_overview_sessions();
    assert_eq!(
        store.overview_state().selection(),
        &HashSet::from([id("ended")])
    );
    assert!(store.close_overview_selection());
    assert_eq!(drain(&mut effects), vec![StoreEffect::Remove(id("ended"))]);
    assert!(store.overview_state().selection().is_empty());
}
