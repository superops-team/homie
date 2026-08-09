//! Platform-independent keyboard and paste encoding for terminal input.
//!
//! Ported from diri-term. Follows `TerminalKeyEncoding.swift` and
//! `GridTerminalView+Input.swift`.

const ESC: u8 = 0x1b;

/// A keyboard event's logical key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Key {
    Character(String),
    Named(NamedKey),
}

/// Non-text keys understood by the terminal encoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamedKey {
    ArrowUp,
    ArrowDown,
    ArrowRight,
    ArrowLeft,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
    Tab,
    Enter,
    Escape,
    Backspace,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

/// A logical key plus the text produced by the platform keyboard layout.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyEvent {
    pub key: Key,
    pub text: Option<String>,
}

impl KeyEvent {
    pub fn character(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            key: Key::Character(text.clone()),
            text: Some(text),
        }
    }

    pub fn composed(logical: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            key: Key::Character(logical.into()),
            text: Some(text.into()),
        }
    }

    pub fn named(key: NamedKey) -> Self {
        Self {
            key: Key::Named(key),
            text: named_text(key).map(str::to_owned),
        }
    }
}

/// Keyboard modifiers relevant to terminal input.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub cmd: bool,
}

/// Terminal modes and input preferences that change emitted bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TermInputModes {
    pub application_cursor_keys: bool,
    pub bracketed_paste: bool,
    pub backspace_sends_delete: bool,
}

impl Default for TermInputModes {
    fn default() -> Self {
        Self {
            application_cursor_keys: false,
            bracketed_paste: false,
            backspace_sends_delete: true,
        }
    }
}

/// Encodes one key event as bytes to write to the PTY.
pub fn encode_key(event: &KeyEvent, modifiers: Modifiers, modes: TermInputModes) -> Vec<u8> {
    let mut buf = Vec::new();

    let ctrl = modifiers.ctrl;
    let alt = modifiers.alt;
    let shift = modifiers.shift;

    if alt {
        buf.push(ESC);
    }

    match &event.key {
        Key::Named(named) => {
            encode_named_key(&mut buf, *named, ctrl, shift, modes);
        }
        Key::Character(_ch) => {
            if let Some(text) = &event.text {
                if ctrl && !text.is_empty() {
                    let first = text.chars().next().unwrap();
                    if first.is_ascii_alphabetic() {
                        let c = first.to_ascii_lowercase() as u8;
                        buf.push(c - b'a' + 1);
                        // Push remaining chars after the ctrl-prefix byte
                        for ch in text.chars().skip(1) {
                            let mut tmp = [0u8; 4];
                            let encoded = ch.encode_utf8(&mut tmp);
                            buf.extend_from_slice(encoded.as_bytes());
                        }
                    } else {
                        buf.extend_from_slice(text.as_bytes());
                    }
                } else {
                    buf.extend_from_slice(text.as_bytes());
                }
            }
        }
    }

    buf
}

fn encode_named_key(
    buf: &mut Vec<u8>,
    key: NamedKey,
    ctrl: bool,
    shift: bool,
    modes: TermInputModes,
) {
    let modifier = ctrl_modifier(ctrl) + shift_modifier(shift);
    match key {
        NamedKey::ArrowUp => {
            if modes.application_cursor_keys && modifier == 0 {
                buf.extend_from_slice(b"\x1bOA");
            } else {
                buf.extend_from_slice(format!("\x1b[1;{}A", 1 + modifier).as_bytes());
            }
        }
        NamedKey::ArrowDown => {
            if modes.application_cursor_keys && modifier == 0 {
                buf.extend_from_slice(b"\x1bOB");
            } else {
                buf.extend_from_slice(format!("\x1b[1;{}B", 1 + modifier).as_bytes());
            }
        }
        NamedKey::ArrowRight => {
            if modes.application_cursor_keys && modifier == 0 {
                buf.extend_from_slice(b"\x1bOC");
            } else {
                buf.extend_from_slice(format!("\x1b[1;{}C", 1 + modifier).as_bytes());
            }
        }
        NamedKey::ArrowLeft => {
            if modes.application_cursor_keys && modifier == 0 {
                buf.extend_from_slice(b"\x1bOD");
            } else {
                buf.extend_from_slice(format!("\x1b[1;{}D", 1 + modifier).as_bytes());
            }
        }
        NamedKey::Home => buf.extend_from_slice(format!("\x1b[1;{}H", 1 + modifier).as_bytes()),
        NamedKey::End => buf.extend_from_slice(format!("\x1b[1;{}F", 1 + modifier).as_bytes()),
        NamedKey::PageUp => buf.extend_from_slice(format!("\x1b[5;{}~", 1 + modifier).as_bytes()),
        NamedKey::PageDown => buf.extend_from_slice(format!("\x1b[6;{}~", 1 + modifier).as_bytes()),
        NamedKey::Insert => buf.extend_from_slice(format!("\x1b[2;{}~", 1 + modifier).as_bytes()),
        NamedKey::Delete => buf.extend_from_slice(format!("\x1b[3;{}~", 1 + modifier).as_bytes()),
        NamedKey::Tab => {
            if shift {
                buf.extend_from_slice(b"\x1b[Z");
            } else {
                buf.push(b'\t');
            }
        }
        NamedKey::Enter => {
            if ctrl {
                buf.push(b'\n');
            } else {
                buf.push(b'\r');
            }
        }
        NamedKey::Escape => buf.push(ESC),
        NamedKey::Backspace => {
            if modes.backspace_sends_delete {
                buf.push(0x7f);
            } else {
                buf.push(0x08);
            }
        }
        NamedKey::F1 => buf.extend_from_slice(b"\x1bOP"),
        NamedKey::F2 => buf.extend_from_slice(b"\x1bOQ"),
        NamedKey::F3 => buf.extend_from_slice(b"\x1bOR"),
        NamedKey::F4 => buf.extend_from_slice(b"\x1bOS"),
        NamedKey::F5 => buf.extend_from_slice(b"\x1b[15~"),
        NamedKey::F6 => buf.extend_from_slice(b"\x1b[17~"),
        NamedKey::F7 => buf.extend_from_slice(b"\x1b[18~"),
        NamedKey::F8 => buf.extend_from_slice(b"\x1b[19~"),
        NamedKey::F9 => buf.extend_from_slice(b"\x1b[20~"),
        NamedKey::F10 => buf.extend_from_slice(b"\x1b[21~"),
        NamedKey::F11 => buf.extend_from_slice(b"\x1b[23~"),
        NamedKey::F12 => buf.extend_from_slice(b"\x1b[24~"),
    }
}

fn ctrl_modifier(ctrl: bool) -> u8 {
    if ctrl { 4 } else { 0 }
}

fn shift_modifier(shift: bool) -> u8 {
    if shift { 1 } else { 0 }
}

fn named_text(key: NamedKey) -> Option<&'static str> {
    match key {
        NamedKey::Tab => Some("\t"),
        NamedKey::Enter => Some("\r"),
        NamedKey::Escape => Some("\x1b"),
        NamedKey::Backspace => Some("\x7f"),
        _ => None,
    }
}

/// Encodes a paste payload, optionally wrapping in bracketed-paste sequences.
pub fn paste(text: &str, bracketed: bool) -> Vec<u8> {
    if !bracketed {
        return text.as_bytes().to_vec();
    }
    let mut buf = Vec::with_capacity(text.len() + 12);
    buf.extend_from_slice(b"\x1b[200~");
    buf.extend_from_slice(text.as_bytes());
    buf.extend_from_slice(b"\x1b[201~");
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow_keys_use_application_mode_when_set() {
        let modes = TermInputModes {
            application_cursor_keys: true,
            ..Default::default()
        };
        assert_eq!(
            encode_key(
                &KeyEvent::named(NamedKey::ArrowUp),
                Modifiers::default(),
                modes
            ),
            b"\x1bOA"
        );
        assert_eq!(
            encode_key(
                &KeyEvent::named(NamedKey::ArrowDown),
                Modifiers::default(),
                modes
            ),
            b"\x1bOB"
        );
    }

    #[test]
    fn ctrl_a_sends_byte_1() {
        assert_eq!(
            encode_key(
                &KeyEvent::character("a"),
                Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                TermInputModes::default()
            ),
            &[1]
        );
    }

    #[test]
    fn paste_matches_bracketed_extension() {
        let text = "one\ntwo λ";
        assert_eq!(paste(text, false), text.as_bytes());
        assert_eq!(paste(text, true), b"\x1b[200~one\ntwo \xce\xbb\x1b[201~");
        assert_eq!(paste("", false), b"");
        assert_eq!(paste("", true), b"\x1b[200~\x1b[201~");
    }
}
