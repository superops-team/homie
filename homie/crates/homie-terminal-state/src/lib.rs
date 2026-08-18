//! Headless terminal emulation for status detection.
//!
//! The daemon has to know what an agent *painted*, not just what bytes it
//! wrote: "do you want to proceed?" only means a blocker if it is still on the
//! visible screen after all the cursor movement, erases and redraws that
//! preceded it. So every session runs a real VT emulator with no renderer
//! attached, and detection reads plain text off its grid.
//!
//! The shared Rust implementation wraps `alacritty_terminal`, a portable
//! headless terminal core used by both the local Engine and remote Holder.
//!
//! One gap is filled by hand: OSC 9;4 (progress) is a ConEmu extension that the
//! emulator does not model, so it is scanned out of the byte stream directly —
//! see [`HeadlessScreen::scan_progress`].

mod mirror;
mod screen;
mod wire;

pub use mirror::{GridMirror, MirrorError};
pub use screen::{HeadlessScreen, ScreenSnapshot};

#[cfg(test)]
mod tests;
