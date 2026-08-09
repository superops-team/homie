//! Placeholder PTY for platforms without a Unix pseudo-terminal.
//!
//! Windows needs ConPTY. Until implemented, these calls fail at runtime.

use std::io;
use std::time::Duration;

use super::{Exit, PtySpec};

pub struct Pty {
    _private: (),
}

fn unsupported<T>() -> io::Result<T> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "PTY support for this platform is not implemented yet; see pty::unsupported",
    ))
}

impl Pty {
    pub fn spawn(_spec: &PtySpec) -> io::Result<Self> {
        unsupported()
    }
    pub fn pid(&self) -> u32 {
        0
    }
    pub fn resize(&self, _cols: u16, _rows: u16) -> io::Result<()> {
        unsupported()
    }
    pub fn size(&self) -> io::Result<(u16, u16)> {
        unsupported()
    }
    pub fn wait(&mut self) -> io::Result<Exit> {
        unsupported()
    }
    pub fn try_wait(&mut self) -> io::Result<Option<Exit>> {
        unsupported()
    }
    pub fn kill_group(&self, _signal: i32) -> io::Result<()> {
        unsupported()
    }
    pub fn terminate(&mut self, _grace: Duration) -> io::Result<Exit> {
        unsupported()
    }
}
