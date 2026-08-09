use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use homie_proto::{ControlMessage, Method, RequestId};
use serde_json::Value;

const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

pub struct RuntimeGuard {
    data_dir: PathBuf,
}

impl RuntimeGuard {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            data_dir: data_dir.to_path_buf(),
        }
    }
}

impl Drop for RuntimeGuard {
    fn drop(&mut self) {
        terminate_sessions(&self.data_dir);
        shutdown_daemon(&self.data_dir);
    }
}

fn terminate_sessions(data_dir: &Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args(["session", "list", "--data-dir"])
        .arg(data_dir)
        .arg("--json")
        .output();
    let Ok(output) = output else {
        return;
    };
    let Ok(sessions) = serde_json::from_slice::<Vec<Value>>(&output.stdout) else {
        return;
    };
    for session_id in sessions
        .iter()
        .filter_map(|session| session.get("id").and_then(Value::as_str))
    {
        let _ = Command::new(env!("CARGO_BIN_EXE_homie"))
            .args(["session", "kill", "--data-dir"])
            .arg(data_dir)
            .args(["--id", session_id])
            .output();
    }
}

fn shutdown_daemon(data_dir: &Path) {
    let socket_path = data_dir.join("runtime/daemon.sock");
    let request = ControlMessage::request(
        RequestId::from(u64::MAX),
        Method::DAEMON_SHUTDOWN,
        serde_json::json!({}),
    );
    let Ok(mut child) = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args(["control-stdio", "--data-dir"])
        .arg(data_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return;
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = serde_json::to_writer(&mut stdin, &request);
        let _ = stdin.write_all(b"\n");
    }
    wait_for_exit(&mut child, SHUTDOWN_TIMEOUT);
    wait_for_removal(&socket_path, SHUTDOWN_TIMEOUT);
}

fn wait_for_exit(child: &mut Child, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(25));
            }
            Ok(None) | Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return;
            }
        }
    }
}

fn wait_for_removal(path: &Path, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while path.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(25));
    }
}
