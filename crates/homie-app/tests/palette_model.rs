use homie_app::palette::{PaletteCommand, PaletteView};

#[test]
fn palette_entries_expose_action_metadata() {
    let palette = PaletteView::new();
    let entries = palette.matches();
    assert!(entries.iter().any(|entry| {
        entry.title == "New Terminal"
            && entry.shortcut == Some("Cmd+T")
            && entry.keywords.contains("shell")
            && entry.command == PaletteCommand::SpawnShell
    }));
}
