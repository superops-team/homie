use homie_ui::{
    AgentKind, AnimationPhase, Appearance, DESIGN_GALLERY, Fill, FontWeightToken, HOMIE_BRAND,
    MemoryFormat, Metrics, Motion, Radius, STATUS_GLYPHS, Space, StatusState, TextRole, TextTone,
    Typo, status_color_name, status_glyph,
};

#[test]
fn design_tokens_match_reference_contract() {
    assert_eq!(Radius::CHIP, 5.0);
    assert_eq!(Radius::BADGE, 6.0);
    assert_eq!(Radius::ROW, 7.0);
    assert_eq!(Radius::CARD, 10.0);
    assert_eq!(Radius::PANEL, 12.0);
    assert_eq!(Metrics::TITLE_BAR, 42.0);
    assert_eq!(Metrics::TOOLBAR_EDGE_INSET, 12.0);
    assert_eq!(Metrics::TOOLBAR_TRAFFIC_LIGHT_LANE, 66.0);
    assert_eq!(Metrics::TOOLBAR_ITEM_GAP, 8.0);
    assert_eq!(Metrics::TOOLBAR_COMPACT_GAP, 4.0);
    assert_eq!(Metrics::TOOLBAR_CONTROL_SIZE, 26.0);
    assert_eq!(Metrics::TOOLBAR_CHIP_HEIGHT, 24.0);
    assert_eq!(Metrics::ROW_HEIGHT, 28.0);
    assert_eq!(Metrics::NEW_AGENT_FOOTER, 32.0);
    assert_eq!(Metrics::TRAFFIC_LIGHT_X_OFFSET, 12.0);
    assert_eq!(Metrics::TRAFFIC_LIGHT_Y_OFFSET, 6.0);
    assert_eq!(Metrics::SIDEBAR_DEFAULT_WIDTH, 248.0);
    assert_eq!(Metrics::SIDEBAR_MIN_WIDTH, 200.0);
    assert_eq!(Metrics::SIDEBAR_MAX_WIDTH, 400.0);
    assert_eq!(Metrics::MIN_WINDOW_WIDTH, 900.0);
    assert_eq!(Metrics::MIN_WINDOW_HEIGHT, 560.0);
}

#[test]
fn typography_tokens_match_diri_roles() {
    assert_eq!(Typo::META.size, 11.0);
    assert_eq!(Typo::META.weight, FontWeightToken::Medium);
    assert_eq!(Typo::SECTION_HEADER.weight, FontWeightToken::Semibold);
    assert_eq!(Typo::ROW.size, 13.0);
    assert_eq!(Typo::ROW_EMPHASIZED.weight, FontWeightToken::Medium);
    assert_eq!(Typo::TITLE.weight, FontWeightToken::Semibold);
    assert_eq!(Typo::DISPLAY_TITLE.size, 15.0);
    let meta_mono = Typo::META_MONO;
    assert!(meta_mono.monospaced);
    assert_eq!(Typo::ALL.len(), 7);
    assert_eq!(Typo::ALL[0].0, TextRole::Meta);
}

#[test]
fn animation_phase_is_deterministic_from_wall_clock_seconds() {
    let phase = AnimationPhase::at(0.0);
    assert_eq!(phase.breathe, 1.0);
    assert_eq!(phase.sweep_turns, 0.0);
    assert_eq!(phase.pulse, 0.5);

    let later = AnimationPhase::at(Motion::SWEEP_SECONDS);
    assert_eq!(later.sweep_turns, 1.0);
    assert_eq!(Motion::SNAP.response, 0.32);
    assert_eq!(Motion::SNAP.damping_fraction, 0.74);
    assert_eq!(Motion::POP.response, 0.40);
    assert_eq!(Motion::SETTLE.damping_fraction, 0.82);
    assert_eq!(Motion::FOOTER_PIN.response, 0.32);
    assert_eq!(Motion::SEAM_SLIDE_MS, 260);
    assert_eq!(Motion::TICK_HZ, 10);
}

#[test]
fn status_color_names_preserve_state_priority() {
    assert_eq!(
        status_color_name(
            AgentKind::Codex,
            StatusState::NeedsInput { destructive: false }
        ),
        "attention"
    );
    assert_eq!(
        status_color_name(
            AgentKind::Codex,
            StatusState::NeedsInput { destructive: true }
        ),
        "danger"
    );
    assert_eq!(
        status_color_name(AgentKind::ClaudeCode, StatusState::Working),
        "clay"
    );
    assert_eq!(
        status_color_name(AgentKind::Gemini, StatusState::Working),
        "gemini_blue"
    );
    assert_eq!(
        status_color_name(AgentKind::Shell, StatusState::Working),
        "generic_working"
    );
}

#[test]
fn semantic_fill_space_and_memory_tokens_match_diri() {
    let colors = homie_ui::SemanticColors::new(Appearance::Dark);
    assert_eq!(colors.text(TextTone::Selected).a, 1.0);
    assert_eq!(colors.text(TextTone::Unselected).a, 0.75);
    assert_eq!(Fill::HOVER_OPACITY, 0.06);
    assert_eq!(Fill::MULTI_SELECTED_OPACITY, 0.08);
    assert_eq!(Fill::SELECTED_OPACITY, 0.10);
    assert_eq!(Space::INDENT, 12.0);
    assert_eq!(Space::ROW_H, 8.0);
    assert_eq!(Space::INSET, 10.0);
    assert_eq!(MemoryFormat::badge(Some(MemoryFormat::SOFT_BYTES)), None);
    assert_eq!(
        MemoryFormat::badge(Some(MemoryFormat::SOFT_BYTES + 1)),
        Some("2.0 GB".to_string())
    );
}

#[test]
fn brand_glyph_and_gallery_catalogs_are_available() {
    assert_eq!(HOMIE_BRAND.wordmark, "Homie");
    assert_eq!(HOMIE_BRAND.bundle_id, "com.superops.homie");
    assert!(STATUS_GLYPHS.iter().any(|glyph| glyph.name == "attention"));
    assert_eq!(status_glyph("working").expect("working glyph").symbol, "●");
    assert_eq!(status_glyph("danger").expect("danger glyph").tone, "danger");
    assert!(DESIGN_GALLERY.iter().any(|entry| entry.id == "workbench"));
    assert!(DESIGN_GALLERY.iter().any(|entry| entry.id == "quick-open"));
}
