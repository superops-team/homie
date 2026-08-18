//! ACP host end-to-end against a real subprocess: spawn this same test binary
//! in `--acp-fake-server` mode as a fake ACP server, then drive it through the
//! real `AcpHost::spawn` path (child process + stdin/stdout pipes).
//!
//! `harness = false` lets us write our own `main`, which dispatches on
//! `--acp-fake-server` so the helper process behaves as the agent side.

use std::io::{BufRead, BufReader, Write};
use std::time::{Duration, Instant};

use homie_engine::acp::{AcpClient, AcpHost};
use serde_json::{Value, json};

fn write_frame(w: &mut impl Write, value: &Value) {
    let mut bytes = serde_json::to_vec(value).expect("encode frame");
    bytes.push(b'\n');
    w.write_all(&bytes).expect("write frame");
    w.flush().expect("flush frame");
}

/// The agent side of the protocol: read JSON-RPC requests from stdin, reply on
/// stdout, and (for `session/prompt`) push a `session/update` notification.
fn run_fake_server() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();

    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).expect("read request");
        if n == 0 {
            return;
        }
        let req: Value = serde_json::from_str(&line).expect("parse request");
        let id = req["id"].as_i64().expect("request id");
        let method = req["method"].as_str().expect("request method");

        match method {
            "initialize" => {
                write_frame(
                    &mut writer,
                    &json!({"jsonrpc":"2.0","id": id, "result": {"protocolVersion": 1}}),
                );
            }
            "session/new" => {
                write_frame(
                    &mut writer,
                    &json!({"jsonrpc":"2.0","id": id, "result": {"sessionId": "sess-test-1"}}),
                );
            }
            "session/prompt" => {
                // Notification first so it is already queued by the time the
                // response unblocks the pending request.
                write_frame(
                    &mut writer,
                    &json!({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"sess-test-1","sessionUpdate":"agent_message_changed"}}),
                );
                write_frame(
                    &mut writer,
                    &json!({"jsonrpc":"2.0","id": id, "result": {"turnId": "turn-1"}}),
                );
            }
            _ => {
                write_frame(
                    &mut writer,
                    &json!({"jsonrpc":"2.0","id": id, "result": {}}),
                );
            }
        }
    }
}

fn run_host_e2e() {
    let exe = std::env::current_exe().expect("current exe");
    let program = exe.to_str().expect("exe utf8");

    let host = AcpHost::spawn(program, &["--acp-fake-server".to_string()]).expect("spawn acp");

    let init = host.initialize().expect("initialize");
    assert_eq!(init["protocolVersion"], 1);

    let session = host
        .request("session/new", json!({"cwd": "/tmp"}))
        .expect("session/new");
    assert_eq!(session["sessionId"], "sess-test-1");

    let turn = host
        .request(
            "session/prompt",
            json!({"sessionId":"sess-test-1","prompt":[{"type":"text","text":"hello"}]}),
        )
        .expect("session/prompt");
    assert_eq!(turn["turnId"], "turn-1");

    // The fake server emits the notification before the prompt reply, so it is
    // already enqueued; poll briefly to tolerate reader-thread scheduling.
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut got_notification = false;
    while Instant::now() < deadline {
        if let Some(n) = host.try_recv_notification() {
            assert_eq!(n.method, "session/update");
            got_notification = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(got_notification, "expected a session/update notification");

    host.request("session/stop", json!({"sessionId":"sess-test-1"}))
        .expect("session/stop");

    // Dropping `host` kills the child (closing its stdout) and joins the
    // reader thread; no explicit cleanup is needed here.
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--acp-fake-server") {
        run_fake_server();
        return;
    }
    run_host_e2e();
    eprintln!("acp_host: end-to-end host loop passed");
}
