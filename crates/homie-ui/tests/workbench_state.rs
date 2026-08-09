use homie_ui::{
    Metrics, SidebarSessionModel, SidebarSessionRow, SidebarState, WorkbenchLayout, move_before,
    move_to_end, status_glyph_name,
};

#[test]
fn sidebar_width_is_clamped_and_resettable() {
    let mut state = SidebarState::default();
    state.set_width(999.0);
    assert_eq!(state.width, Metrics::SIDEBAR_MAX_WIDTH);
    state.set_width(120.0);
    assert_eq!(state.width, Metrics::SIDEBAR_MIN_WIDTH);
    state.reset_width();
    assert_eq!(state.width, Metrics::SIDEBAR_DEFAULT_WIDTH);
    state.toggle();
    assert!(!state.visible);
}

#[test]
fn reorder_helpers_move_items_without_duplication() {
    let mut order = vec!["a", "b", "c", "d"];
    move_before(&mut order, &"d", &"b");
    assert_eq!(order, ["a", "d", "b", "c"]);
    move_to_end(&mut order, &"a");
    assert_eq!(order, ["d", "b", "c", "a"]);
}

#[test]
fn workbench_layout_splits_primary_and_auxiliary_panes() {
    let layout = WorkbenchLayout::default();
    let heights = layout.pane_heights(600.0);
    assert_eq!(heights.primary, 372.0);
    assert_eq!(heights.auxiliary, 228.0);
    assert_eq!(WorkbenchLayout::from_fraction(2.0).primary_fraction(), 1.0);
}

#[test]
fn sidebar_session_model_tracks_selection_and_multi_select() {
    let mut model = SidebarSessionModel::new(vec![
        row("s1", "Alpha", "running"),
        row("s2", "Beta", "needs_input"),
        row("s3", "Gamma", "exited"),
    ]);

    model.select("s2");
    assert_eq!(model.selected.as_deref(), Some("s2"));
    model.toggle_multi_select("s1");
    model.toggle_multi_select("s3");
    assert_eq!(model.multi_selected, ["s1", "s3"]);
    model.toggle_multi_select("s1");
    assert_eq!(model.multi_selected, ["s3"]);
}

#[test]
fn sidebar_session_model_renames_pins_archives_and_reorders() {
    let mut model = SidebarSessionModel::new(vec![
        row("s1", "Alpha", "running"),
        row("s2", "Beta", "idle"),
        row("s3", "Gamma", "exited"),
    ]);

    model.rename("s2", "Renamed");
    assert_eq!(model.rows[1].title, "Renamed");
    model.toggle_pin("s3");
    assert_eq!(model.rows[0].id, "s3");
    model.move_before("s2", "s3");
    assert_eq!(ids(&model), ["s2", "s3", "s1"]);
    model.move_to_end("s2");
    assert_eq!(ids(&model), ["s3", "s1", "s2"]);
    model.select("s1");
    model.toggle_multi_select("s2");
    model.archive("s1");
    assert_eq!(model.selected, None);
    assert!(
        model
            .rows
            .iter()
            .find(|row| row.id == "s1")
            .unwrap()
            .archived
    );
}

#[test]
fn sidebar_status_glyph_names_cover_diri_states() {
    assert_eq!(status_glyph_name("needs_input"), "attention");
    assert_eq!(status_glyph_name("running"), "working");
    assert_eq!(status_glyph_name("idle"), "idle");
    assert_eq!(status_glyph_name("hibernated"), "hibernated");
    assert_eq!(status_glyph_name("exited"), "exited");
    assert_eq!(status_glyph_name("future"), "unknown");
}

fn row(id: &str, title: &str, status: &str) -> SidebarSessionRow {
    SidebarSessionRow {
        id: id.to_string(),
        title: title.to_string(),
        status: status.to_string(),
        pinned: false,
        archived: false,
    }
}

fn ids(model: &SidebarSessionModel) -> Vec<&str> {
    model.rows.iter().map(|row| row.id.as_str()).collect()
}
