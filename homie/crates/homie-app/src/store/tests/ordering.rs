use super::*;

fn rows(projection: &SidebarProjection, group: usize) -> Vec<SessionId> {
    projection.projects[group]
        .sessions
        .iter()
        .map(|row| row.id().clone())
        .collect()
}

#[test]
fn projection_keeps_manual_ranks_and_appends_the_rest_in_arrival_order() {
    let prefs = Prefs {
        sidebar_project_order: vec![pid("z")],
        sidebar_session_order: vec![id("old-ranked")],
        ..Prefs::default()
    };
    let (mut store, _) = hydrated(
        vec![
            session("old-ranked", "a", 1.0),
            session("new", "a", 3.0),
            session("middle", "a", 2.0),
            session("z-session", "z", 4.0),
        ],
        vec![project("a", "Alpha"), project("z", "Zulu")],
        prefs,
    );

    let projection = store.sidebar_projection();
    assert_eq!(projection.projects[0].project.id, pid("z"));
    assert_eq!(
        rows(&projection, 1),
        // Oldest first behind the one row that was ranked by hand. The newest
        // session is last, which is the whole point: a session created now
        // belongs at the bottom, not wherever its timestamp happens to sort.
        vec![id("old-ranked"), id("middle"), id("new")]
    );
}

#[test]
fn a_new_session_and_a_new_project_land_at_the_bottom() {
    let (mut store, _) = hydrated(
        vec![session("first", "a", 1.0), session("second", "a", 2.0)],
        vec![project("a", "Alpha")],
        Prefs::default(),
    );
    assert_eq!(
        rows(&store.sidebar_projection(), 0),
        vec![id("first"), id("second")]
    );

    // "Zed" sorts after "Alpha" alphabetically and "Ada" sorts before it; the
    // old projection put a new project wherever its name fell, so use a name
    // that would have jumped the queue.
    store.upsert_session(session("third", "a", 3.0));
    store.upsert_session(session("fresh", "ada", 4.0));

    let projection = store.sidebar_projection();
    assert_eq!(
        rows(&projection, 0),
        vec![id("first"), id("second"), id("third")],
        "a new session appends to its project"
    );
    assert_eq!(
        projection
            .projects
            .iter()
            .map(|group| group.project.id.clone())
            .collect::<Vec<_>>(),
        vec![pid("a"), pid("ada")],
        "a new project appends to the list, whatever it is called"
    );
}

#[test]
fn a_session_dragged_to_the_end_stays_at_the_end() {
    let (mut store, _) = hydrated(
        vec![
            session("one", "a", 1.0),
            session("two", "a", 2.0),
            session("three", "a", 3.0),
        ],
        vec![project("a", "Alpha")],
        Prefs::default(),
    );

    let mut order = store.sidebar_session_order();
    super::super::super::sidebar::move_to_end(&mut order, &id("one"));
    store.set_session_order(order).expect("persist order");

    assert_eq!(
        rows(&store.sidebar_projection(), 0),
        vec![id("two"), id("three"), id("one")]
    );
}

#[test]
fn removing_a_session_prunes_it_from_the_persisted_order() {
    let (mut store, _) = hydrated(
        vec![session("one", "a", 1.0), session("two", "a", 2.0)],
        vec![project("a", "Alpha")],
        Prefs::default(),
    );
    store
        .toggle_session_collapsed(id("one"))
        .expect("collapse one");
    assert!(
        store
            .preferences()
            .sidebar_session_order
            .contains(&id("one"))
    );

    store.remove_session_record(&id("one"));

    assert_eq!(store.preferences().sidebar_session_order, vec![id("two")]);
    assert!(store.preferences().sidebar_collapsed_sessions.is_empty());
}

#[test]
fn spawned_sessions_nest_under_their_parent() {
    let child = |name: &str, parent: &str, created: f64| {
        let mut record = session(name, "p", created);
        record.parent = Some(id(parent));
        record
    };
    let (mut store, _) = hydrated(
        vec![
            session("root", "p", 1.0),
            child("child-b", "root", 3.0),
            child("child-a", "root", 2.0),
            child("grandchild", "child-a", 4.0),
            session("sibling", "p", 5.0),
        ],
        vec![project("p", "P")],
        Prefs::default(),
    );

    let projection = store.sidebar_projection();
    let shape: Vec<_> = projection.projects[0]
        .sessions
        .iter()
        .map(|row| (row.id().0.as_str().to_owned(), row.depth))
        .collect();
    assert_eq!(
        shape,
        vec![
            ("root".to_owned(), 0),
            ("child-a".to_owned(), 1),
            ("grandchild".to_owned(), 2),
            ("child-b".to_owned(), 1),
            ("sibling".to_owned(), 0),
        ]
    );
    assert!(projection.projects[0].sessions[0].has_children);
    assert!(!projection.projects[0].sessions[4].has_children);
}

#[test]
fn collapsing_a_parent_folds_away_its_subtree() {
    let mut child = session("child", "p", 2.0);
    child.parent = Some(id("root"));
    let (mut store, _) = hydrated(
        vec![session("root", "p", 1.0), child, session("other", "p", 3.0)],
        vec![project("p", "P")],
        Prefs::default(),
    );

    store
        .toggle_session_collapsed(id("root"))
        .expect("collapse");

    let projection = store.sidebar_projection();
    assert_eq!(rows(&projection, 0), vec![id("root"), id("other")]);
    assert!(projection.projects[0].sessions[0].collapsed);
    assert!(
        !projection
            .ordered_sessions
            .iter()
            .any(|session| session.id == id("child")),
        "a folded row must not consume a ⌘1…⌘9 slot"
    );
    assert!(
        projection.display_order.contains(&id("child")),
        "it is hidden, not gone"
    );
}

#[test]
fn selecting_a_folded_session_reveals_it() {
    let mut child = session("child", "p", 2.0);
    child.parent = Some(id("root"));
    let prefs = Prefs {
        sidebar_collapsed_projects: vec![pid("p")],
        sidebar_collapsed_sessions: vec![id("root")],
        ..Prefs::default()
    };
    let (mut store, _) = hydrated(
        vec![session("root", "p", 1.0), child],
        vec![project("p", "P")],
        prefs,
    );

    store.select(id("child"));

    assert!(store.preferences().sidebar_collapsed_projects.is_empty());
    assert!(store.preferences().sidebar_collapsed_sessions.is_empty());
    assert_eq!(
        rows(&store.sidebar_projection(), 0),
        vec![id("root"), id("child")]
    );
}

#[test]
fn folding_over_the_selection_moves_it_to_the_fold() {
    let mut child = session("child", "p", 2.0);
    child.parent = Some(id("root"));
    let (mut store, _) = hydrated(
        vec![session("root", "p", 1.0), child],
        vec![project("p", "P")],
        Prefs::default(),
    );
    store.select(id("child"));

    store
        .toggle_session_collapsed(id("root"))
        .expect("collapse");

    assert_eq!(store.selected_session_id(), Some(&id("root")));
    assert_eq!(rows(&store.sidebar_projection(), 0), vec![id("root")]);
}

#[test]
fn unusable_parents_leave_the_row_at_the_root() {
    let mut foreign = session("foreign", "p", 2.0);
    foreign.parent = Some(id("elsewhere"));
    let mut left = session("left", "p", 3.0);
    left.parent = Some(id("right"));
    let mut right = session("right", "p", 4.0);
    right.parent = Some(id("left"));
    let (mut store, _) = hydrated(
        vec![session("elsewhere", "other", 1.0), foreign, left, right],
        vec![project("p", "P"), project("other", "Other")],
        Prefs::default(),
    );

    let projection = store.sidebar_projection();
    let group = projection
        .projects
        .iter()
        .find(|group| group.project.id == pid("p"))
        .expect("project p");
    assert_eq!(
        group
            .sessions
            .iter()
            .map(|row| row.depth)
            .collect::<Vec<_>>(),
        vec![0, 0, 1],
        "the cross-project child roots, and the cycle keeps exactly one edge"
    );
    assert_eq!(group.sessions.len(), 3, "no row is lost to a cycle");
}

#[test]
fn pinned_rows_lead_their_siblings() {
    let prefs = Prefs {
        sidebar_pinned_sessions: vec![id("third")],
        sidebar_pinned_projects: vec![pid("b")],
        ..Prefs::default()
    };
    let (mut store, _) = hydrated(
        vec![
            session("first", "a", 1.0),
            session("second", "a", 2.0),
            session("third", "a", 3.0),
            session("only", "b", 4.0),
        ],
        vec![project("a", "Alpha"), project("b", "Beta")],
        prefs,
    );

    let projection = store.sidebar_projection();
    assert_eq!(
        projection
            .projects
            .iter()
            .map(|group| group.project.id.clone())
            .collect::<Vec<_>>(),
        vec![pid("b"), pid("a")],
        "a pinned project leads even though it arrived last"
    );
    assert_eq!(
        rows(&projection, 1),
        vec![id("third"), id("first"), id("second")]
    );
    assert!(projection.projects[1].sessions[0].pinned);
}

#[test]
fn projection_synthesizes_projects_and_handles_archived_selection() {
    let active = session("active", "missing", 2.0);
    let mut archived = session("archived", "missing", 1.0);
    archived.worktree_path = Some("/repo/worktrees/feature-one".to_owned());
    archived.archived_at = Some(DateMillis(20.0));
    let (mut store, _) = hydrated(vec![active, archived], vec![], Prefs::default());

    let first = store.sidebar_projection();
    assert_eq!(first.projects[0].project.id, pid("missing"));
    assert_eq!(first.projects[0].project.root, "/work/missing");
    assert_eq!(first.ordered_sessions.len(), 1);
    assert!(Arc::ptr_eq(&first, &store.sidebar_projection()));

    store.select(id("archived"));
    let selected = store.sidebar_projection();
    assert_eq!(
        selected
            .ordered_sessions
            .iter()
            .map(|session| session.id.clone())
            .collect::<Vec<_>>(),
        vec![id("active"), id("archived")]
    );

    let mut lone = session("lone", "synthetic", 1.0);
    lone.worktree_path = Some("/repo/worktrees/feature-two".to_owned());
    let (mut lone_store, _) = hydrated(vec![lone], vec![], Prefs::default());
    let synthesized = lone_store.sidebar_projection();
    assert_eq!(
        synthesized.projects[0].project.root,
        "/repo/worktrees/feature-two"
    );
    assert_eq!(synthesized.projects[0].project.name, "feature-two");
}

#[test]
fn projection_reuses_one_session_record_per_sidebar_row() {
    let (mut store, _) = hydrated(
        vec![session("one", "p", 1.0)],
        vec![project("p", "P")],
        Prefs::default(),
    );

    let projection = store.sidebar_projection();
    let grouped: &SessionRecord = &projection.projects[0].sessions[0].session;
    let ordered: &SessionRecord = &projection.ordered_sessions[0];
    assert!(
        std::ptr::eq(grouped, ordered),
        "sidebar order must share the row record instead of cloning its transcript metadata"
    );
}
