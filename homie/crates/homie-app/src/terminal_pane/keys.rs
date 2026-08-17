//! GPUI key event → terminal key event adapter.
//!
//! Pure mapping: no `Window`/`Context`/`Entity`/render dependency, so it stays
//! unit-testable in isolation from the GPUI view.

use gpui::KeyDownEvent;
use homie_term::keys::{Key as TermKey, KeyEvent as TermKeyEvent, NamedKey};

pub(crate) fn terminal_key_event(event: &KeyDownEvent) -> Option<TermKeyEvent> {
    let named = match event.keystroke.key.as_str() {
        "up" => Some(NamedKey::ArrowUp),
        "down" => Some(NamedKey::ArrowDown),
        "right" => Some(NamedKey::ArrowRight),
        "left" => Some(NamedKey::ArrowLeft),
        "home" => Some(NamedKey::Home),
        "end" => Some(NamedKey::End),
        "pageup" => Some(NamedKey::PageUp),
        "pagedown" => Some(NamedKey::PageDown),
        "insert" => Some(NamedKey::Insert),
        "delete" => Some(NamedKey::Delete),
        "tab" => Some(NamedKey::Tab),
        "enter" => Some(NamedKey::Enter),
        "escape" => Some(NamedKey::Escape),
        "backspace" => Some(NamedKey::Backspace),
        "f1" => Some(NamedKey::F1),
        "f2" => Some(NamedKey::F2),
        "f3" => Some(NamedKey::F3),
        "f4" => Some(NamedKey::F4),
        "f5" => Some(NamedKey::F5),
        "f6" => Some(NamedKey::F6),
        "f7" => Some(NamedKey::F7),
        "f8" => Some(NamedKey::F8),
        "f9" => Some(NamedKey::F9),
        "f10" => Some(NamedKey::F10),
        "f11" => Some(NamedKey::F11),
        "f12" => Some(NamedKey::F12),
        _ => None,
    };
    if let Some(named) = named {
        return Some(TermKeyEvent::named(named));
    }
    let logical = event.keystroke.key.clone();
    let text = event
        .keystroke
        .key_char
        .clone()
        .unwrap_or_else(|| logical.clone());
    (!logical.is_empty()).then_some(TermKeyEvent {
        key: TermKey::Character(logical),
        text: Some(text),
    })
}
