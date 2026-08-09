//! Unix PTY implementation.
//!
//! Ported from diri-engine. Uses `openpty` for pseudo-terminal allocation.

use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};

use super::{Exit, PtySpec};

/// One past the highest signal to reset in the child.
#[cfg(target_os = "macos")]
const MAX_SIGNAL: libc::c_int = 32;
#[cfg(not(target_os = "macos"))]
const MAX_SIGNAL: libc::c_int = 65;

pub struct Pty {
    master: OwnedFd,
    child: Child,
}

impl Pty {
    /// Spawns `spec` on a new pseudo-terminal.
    pub fn spawn(spec: &PtySpec) -> io::Result<Self> {
        let program = spec
            .argv
            .first()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "argv is empty"))?;

        let mut winsize = libc::winsize {
            ws_row: spec.rows,
            ws_col: spec.cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let mut master: RawFd = -1;
        let mut slave: RawFd = -1;
        // SAFETY: both out-params are valid locals; winsize is fully initialized.
        let rc = unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut winsize,
            )
        };
        if rc != 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: openpty succeeded, so both are fresh owned descriptors.
        let master = unsafe { OwnedFd::from_raw_fd(master) };
        let slave = unsafe { OwnedFd::from_raw_fd(slave) };

        let mut command = Command::new(OsStr::new(program));
        command.args(&spec.argv[1..]);
        command.env_clear();
        for (key, value) in &spec.env {
            command.env(key, value);
        }
        command.current_dir(&spec.cwd);

        command.stdin(Stdio::from(slave.try_clone()?));
        command.stdout(Stdio::from(slave.try_clone()?));
        command.stderr(Stdio::from(slave.try_clone()?));

        let slave_fd = slave.as_raw_fd();
        // SAFETY: the closure runs between fork and exec and uses only
        // async-signal-safe syscalls.
        unsafe {
            command.pre_exec(move || {
                let mut empty: libc::sigset_t = std::mem::zeroed();
                libc::sigemptyset(&mut empty);
                libc::sigprocmask(libc::SIG_SETMASK, &empty, std::ptr::null_mut());
                for signal in 1..MAX_SIGNAL {
                    libc::signal(signal, libc::SIG_DFL);
                }

                if libc::setsid() < 0 {
                    return Err(io::Error::last_os_error());
                }
                if libc::ioctl(slave_fd, libc::TIOCSCTTY as _, 0) < 0 {
                    return Err(io::Error::last_os_error());
                }

                let max = libc::getdtablesize();
                for fd in 3..max {
                    libc::close(fd);
                }
                Ok(())
            });
        }

        let child = command.spawn()?;
        drop(slave); // the parent must not hold the slave open
        Ok(Self { master, child })
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
        let winsize = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        // SAFETY: the fd is owned and winsize is initialized.
        let rc = unsafe { libc::ioctl(self.master.as_raw_fd(), libc::TIOCSWINSZ as _, &winsize) };
        if rc < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn size(&self) -> io::Result<(u16, u16)> {
        // SAFETY: the fd is owned; the kernel fills the struct.
        let mut winsize: libc::winsize = unsafe { std::mem::zeroed() };
        let rc =
            unsafe { libc::ioctl(self.master.as_raw_fd(), libc::TIOCGWINSZ as _, &mut winsize) };
        if rc < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok((winsize.ws_col, winsize.ws_row))
    }

    pub fn reader(&self) -> io::Result<PtyStream> {
        Ok(PtyStream(File::from(self.master.try_clone()?)))
    }

    pub fn writer(&self) -> io::Result<PtyStream> {
        Ok(PtyStream(File::from(self.master.try_clone()?)))
    }

    pub fn wait(&mut self) -> io::Result<Exit> {
        let status = self.child.wait()?;
        Ok(exit_from(status))
    }

    pub fn try_wait(&mut self) -> io::Result<Option<Exit>> {
        Ok(self.child.try_wait()?.map(exit_from))
    }

    pub fn kill_group(&self, signal: i32) -> io::Result<()> {
        let pid = self.child.id() as i32;
        // SAFETY: plain kill(2) on a group we created.
        let rc = unsafe { libc::kill(-pid, signal) };
        if rc < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(error);
            }
        }
        Ok(())
    }

    pub fn terminate(&mut self, grace: std::time::Duration) -> io::Result<Exit> {
        self.kill_group(libc::SIGTERM)?;
        let deadline = std::time::Instant::now() + grace;
        while std::time::Instant::now() < deadline {
            if let Some(exit) = self.try_wait()? {
                return Ok(exit);
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        self.kill_group(libc::SIGKILL)?;
        self.wait()
    }
}

fn exit_from(status: std::process::ExitStatus) -> Exit {
    use std::os::unix::process::ExitStatusExt;
    match status.signal() {
        Some(signal) => Exit::Signal(signal),
        None => Exit::Code(status.code().unwrap_or(-1)),
    }
}

/// Read/write handle on the PTY master.
pub struct PtyStream(File);

impl PtyStream {
    pub fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }

    pub fn wait_readable(&self, timeout: std::time::Duration) -> io::Result<bool> {
        let mut poll_fd = libc::pollfd {
            fd: self.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: one initialized pollfd with a millisecond timeout.
        let ready = unsafe { libc::poll(&mut poll_fd, 1, timeout.as_millis() as libc::c_int) };
        if ready < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(poll_fd.revents & (libc::POLLIN | libc::POLLHUP | libc::POLLERR) != 0)
    }

    pub fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl Read for PtyStream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        match self.0.read(buffer) {
            Err(error) if error.raw_os_error() == Some(libc::EIO) => Ok(0),
            other => other,
        }
    }
}

impl Write for PtyStream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        loop {
            match self.0.write(buffer) {
                Err(error)
                    if error.kind() == io::ErrorKind::WouldBlock
                        || error.kind() == io::ErrorKind::Interrupted =>
                {
                    std::thread::yield_now();
                    continue;
                }
                other => return other,
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}
