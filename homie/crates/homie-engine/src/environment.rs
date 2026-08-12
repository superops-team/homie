use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const FALLBACK_SUFFIX: &str =
    ".local/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin";
const DEFAULT_REFRESH_TTL: Duration = Duration::from_secs(300);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartupEnvironment {
    pub shell: String,
    pub path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathSource {
    Override,
    Cache,
    Fallback,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathResolution {
    pub path: String,
    pub source: PathSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefreshStatus {
    Updated,
    Unchanged,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefreshOutcome {
    pub path: String,
    pub status: RefreshStatus,
}

#[cfg(unix)]
pub fn login_shell() -> String {
    // SAFETY: getpwuid returns a pointer to a static per-thread record; it is
    // read immediately and never retained.
    unsafe {
        let record = libc::getpwuid(libc::getuid());
        if !record.is_null() {
            let shell = std::ffi::CStr::from_ptr((*record).pw_shell);
            if let Ok(shell) = shell.to_str()
                && !shell.is_empty()
                && Path::new(shell).exists()
            {
                return shell.to_owned();
            }
        }
    }
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into())
}

#[cfg(not(unix))]
pub fn login_shell() -> String {
    std::env::var("SHELL").unwrap_or_default()
}

pub fn startup_environment(app_support: &Path) -> StartupEnvironment {
    let shell = login_shell();
    let path = startup_path(app_support).path;
    StartupEnvironment { shell, path }
}

pub fn startup_path(app_support: &Path) -> PathResolution {
    if let Some(path) = std::env::var("HOMIE_PATH_OVERRIDE")
        .ok()
        .filter(|value| !value.trim().is_empty())
    {
        return PathResolution {
            path,
            source: PathSource::Override,
        };
    }

    if let Some(path) = read_cached_path(app_support) {
        return PathResolution {
            path,
            source: PathSource::Cache,
        };
    }

    PathResolution {
        path: fallback_path(),
        source: PathSource::Fallback,
    }
}

pub fn fallback_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    format!("{home}/{FALLBACK_SUFFIX}")
}

pub fn path_cache_file(app_support: &Path) -> PathBuf {
    app_support.join("environment/path.txt")
}

pub fn read_cached_path(app_support: &Path) -> Option<String> {
    let contents = std::fs::read_to_string(path_cache_file(app_support)).ok()?;
    let path = contents.lines().next().unwrap_or("").trim();
    (!path.is_empty()).then(|| path.to_owned())
}

pub fn write_cached_path(app_support: &Path, path: &str) -> std::io::Result<()> {
    let parent = path_cache_file(app_support)
        .parent()
        .expect("path cache has parent")
        .to_path_buf();
    std::fs::create_dir_all(parent)?;
    std::fs::write(path_cache_file(app_support), format!("{path}\n"))
}

pub fn refresh_path(
    app_support: &Path,
    shell: &str,
    timeout: Duration,
) -> io::Result<RefreshOutcome> {
    if let Some(path) = std::env::var("HOMIE_PATH_OVERRIDE")
        .ok()
        .filter(|value| !value.trim().is_empty())
    {
        // SAFETY: control requests are served on a dedicated daemon thread;
        // updating PATH here only changes future spawns/readiness probes.
        unsafe { std::env::set_var("PATH", &path) };
        return Ok(RefreshOutcome {
            path,
            status: RefreshStatus::Unchanged,
        });
    }

    if let Some(path) = fresh_cached_path(app_support, refresh_ttl()) {
        // SAFETY: see above.
        unsafe { std::env::set_var("PATH", &path) };
        return Ok(RefreshOutcome {
            path,
            status: RefreshStatus::Unchanged,
        });
    }

    let output = run_path_capture(shell, timeout)?;
    let Some(path) = extract_path(&output) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "path refresh produced no PATH-like output",
        ));
    };
    let previous = read_cached_path(app_support);
    write_cached_path(app_support, &path)?;
    // SAFETY: control requests are served on a dedicated daemon thread; updating
    // PATH here only changes future spawns/readiness probes.
    unsafe { std::env::set_var("PATH", &path) };
    Ok(RefreshOutcome {
        status: if previous.as_deref() == Some(path.as_str()) {
            RefreshStatus::Unchanged
        } else {
            RefreshStatus::Updated
        },
        path,
    })
}

fn refresh_ttl() -> Duration {
    std::env::var("HOMIE_PATH_REFRESH_TTL_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_REFRESH_TTL)
}

fn fresh_cached_path(app_support: &Path, ttl: Duration) -> Option<String> {
    let file = path_cache_file(app_support);
    let metadata = std::fs::metadata(&file).ok()?;
    let modified = metadata.modified().ok()?;
    let age = modified.elapsed().ok()?;
    if age > ttl {
        return None;
    }
    read_cached_path(app_support)
}

fn run_path_capture(shell: &str, timeout: Duration) -> io::Result<String> {
    let mut child = Command::new(shell)
        .args(["-l", "-c", "printenv PATH"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            let mut stdout = String::new();
            if let Some(mut pipe) = child.stdout.take() {
                use std::io::Read;
                let _ = pipe.read_to_string(&mut stdout);
            }
            if status.success() {
                return Ok(stdout);
            }
            return Err(io::Error::other(format!(
                "path refresh exited with {status}"
            )));
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "path refresh timed out",
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

pub fn extract_path(output: &str) -> Option<String> {
    output
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| line.contains('/') && line.contains(':'))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn startup_path_uses_fallback_without_shell_capture() {
        let _guard = env_lock().lock().expect("env lock");
        unsafe { std::env::remove_var("HOMIE_PATH_OVERRIDE") };
        let temp = tempfile::tempdir().expect("temp");
        let resolved = startup_path(temp.path());
        assert_eq!(resolved.source, PathSource::Fallback);
        assert!(resolved.path.contains("/usr/bin"));
    }

    #[test]
    fn startup_path_prefers_override() {
        let _guard = env_lock().lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp");
        unsafe { std::env::set_var("HOMIE_PATH_OVERRIDE", "/custom/bin:/usr/bin") };
        let resolved = startup_path(temp.path());
        unsafe { std::env::remove_var("HOMIE_PATH_OVERRIDE") };
        assert_eq!(resolved.source, PathSource::Override);
        assert_eq!(resolved.path, "/custom/bin:/usr/bin");
    }

    #[test]
    fn startup_path_uses_cached_path_before_fallback() {
        let _guard = env_lock().lock().expect("env lock");
        unsafe { std::env::remove_var("HOMIE_PATH_OVERRIDE") };
        let temp = tempfile::tempdir().expect("temp");
        write_cached_path(temp.path(), "/cached/bin:/usr/bin").expect("write cache");
        let resolved = startup_path(temp.path());
        assert_eq!(resolved.source, PathSource::Cache);
        assert_eq!(resolved.path, "/cached/bin:/usr/bin");
    }

    #[test]
    fn extract_path_uses_the_last_path_like_line() {
        let output = "hello\n/usr/bin\n/custom/bin:/usr/bin:/bin\n";
        assert_eq!(
            extract_path(output).as_deref(),
            Some("/custom/bin:/usr/bin:/bin")
        );
    }

    #[test]
    fn extract_path_ignores_non_path_output() {
        assert_eq!(extract_path("hello\nstill not a path\n"), None);
    }

    #[test]
    fn refresh_path_updates_cache_without_interactive_shell() {
        let _guard = env_lock().lock().expect("env lock");
        unsafe { std::env::remove_var("HOMIE_PATH_OVERRIDE") };
        unsafe { std::env::set_var("HOMIE_PATH_REFRESH_TTL_SECS", "0") };
        let temp = tempfile::tempdir().expect("temp");
        let shell = temp.path().join("fake-shell");
        let argv_log = temp.path().join("argv.log");
        std::fs::write(
            &shell,
            format!(
                "#!/usr/bin/env bash\nprintf '%s\\n' \"$*\" > {}\nprintf '/fresh/bin:/usr/bin:/bin\\n'\n",
                argv_log.display()
            ),
        )
        .expect("write shell");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&shell, std::fs::Permissions::from_mode(0o755))
                .expect("chmod");
        }

        let outcome = refresh_path(
            temp.path(),
            shell.to_str().expect("path"),
            Duration::from_secs(2),
        )
        .expect("refresh");
        unsafe { std::env::remove_var("HOMIE_PATH_REFRESH_TTL_SECS") };
        assert_eq!(outcome.status, RefreshStatus::Updated);
        assert_eq!(outcome.path, "/fresh/bin:/usr/bin:/bin");
        assert_eq!(
            read_cached_path(temp.path()).as_deref(),
            Some("/fresh/bin:/usr/bin:/bin")
        );
        let argv = std::fs::read_to_string(argv_log).expect("argv log");
        assert_eq!(argv.trim(), "-l -c printenv PATH");
    }
}
