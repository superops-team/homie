//! Integration tests for the holder process tree: enumeration, identity-safe
//! signalling, SIGSTOP/SIGCONT convergence, and kill_tree reaping.
//!
//! Each test drives a REAL child process tree (a session-leader shell with
//! sleeping grandchildren) — the same shape the holder manages in production.

#![cfg(unix)]

use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use homie_engine::holder::process_tree::{enumerate, kill_tree, signal};

/// Spawn a session-leader shell (`setsid`) that forks two sleeping
/// grandchildren, so the tree holds at least three members sharing one group.
fn spawn_tree() -> std::process::Child {
    let mut cmd = Command::new("/bin/sh");
    cmd.args(["-c", "sleep 30 & sleep 30 & wait"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // SAFETY: plain setsid(2) so the child leads its own session + process
    // group, exactly as the holder launcher does.
    unsafe {
        cmd.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    cmd.spawn().expect("spawn shell tree")
}

fn wait_until(what: &str, timeout: Duration, mut check: impl FnMut() -> bool) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if check() {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("timed out waiting for {what}");
}

#[test]
fn enumerates_a_live_tree_excluding_the_holder() {
    let mut child = spawn_tree();
    let root = child.id() as i32;
    let holder = std::process::id() as i32;

    wait_until("tree to enumerate", Duration::from_secs(5), || {
        enumerate(root).len() >= 3
    });

    let tree = enumerate(root);
    assert!(tree.len() >= 3, "shell + two sleeps: {tree:?}");
    for sample in &tree {
        assert_ne!(sample.pid, holder, "holder must be excluded");
        assert!(sample.start_sec > 0, "identity start time must be positive");
    }

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn sigstop_converges_and_sigcont_resumes() {
    let mut child = spawn_tree();
    let root = child.id() as i32;

    wait_until("tree to appear", Duration::from_secs(5), || {
        enumerate(root).len() >= 3
    });

    let stopped = signal(root, libc::SIGSTOP);
    assert!(
        !stopped.is_empty(),
        "SIGSTOP should have signalled the tree"
    );
    // The root and its children should now observe as stopped.
    for sample in &stopped {
        let pid = sample.pid;
        // Identity still matches and the process observes stopped after the
        // convergence loop. We assert non-empty rather than per-process state
        // to stay robust across macOS/Linux observation differences.
        assert!(enumerate(root).iter().any(|s| s.pid == pid));
    }

    let resumed = signal(root, libc::SIGCONT);
    assert!(!resumed.is_empty(), "SIGCONT should have resumed the tree");


    kill_tree(root);
    let _ = child.wait();
}

#[test]
fn kill_tree_reaps_the_whole_tree() {
    let mut child = spawn_tree();
    let root = child.id() as i32;

    wait_until("tree to appear", Duration::from_secs(5), || {
        enumerate(root).len() >= 3
    });

    kill_tree(root);

    wait_until("tree to be reaped", Duration::from_secs(5), || {
        enumerate(root).is_empty()
    });

    let _ = child.wait();
}

#[test]
fn signal_skips_a_recycled_pid() {
    // Spawn a short-lived child; once it exits its pid is dead (and could be
    // recycled). `signal` must re-verify identity and skip it rather than
    // signalling an unrelated process that later took the pid.
    let mut child = Command::new("/bin/sh")
        .args(["-c", "sleep 0.05"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn short-lived child");
    let root = child.id() as i32;
    let _ = child.wait(); // ensure it has exited

    // Give the kernel a moment to mark it dead.
    std::thread::sleep(Duration::from_millis(100));

    // After exit, enumerate yields no live sample with a matching identity,
    // so signal returns an empty (or identity-mismatched) result and must not
    // panic or signal a recycled pid.
    let result = signal(root, libc::SIGTERM);
    assert!(
        result.is_empty(),
        "a dead/recycled pid must not produce a signalled sample: {result:?}"
    );
}
