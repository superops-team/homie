//! ACP host loop: spawn an ACP-compliant agent server over stdio and speak
//! JSON-RPC 2.0 to it.
//!
//! Homie is the *host* (JSON-RPC client). It owns a child process, a background
//! reader thread that classifies every inbound frame (routing responses to
//! their pending request and enqueueing notifications), and a writer used for
//! requests. This keeps request/response correlation off the UI thread and
//! matches the engine's synchronous, std-only design (no async runtime).

use std::collections::HashMap;
use std::fmt;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use serde_json::Value;

use super::frame;
use super::protocol::{InboundMessage, JsonRpcNotification, JsonRpcRequest, classify_inbound};

#[derive(Debug)]
pub enum AcpError {
    Io(io::Error),
    /// A frame arrived that is not valid JSON-RPC, or could not be encoded.
    Protocol(String),
    /// The agent returned a JSON-RPC error for a request.
    Rpc {
        code: i64,
        message: String,
    },
    /// The stream ended (or the reader thread died) before a reply arrived.
    Eof,
}

impl fmt::Display for AcpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "acp io error: {e}"),
            Self::Protocol(m) => write!(f, "acp protocol error: {m}"),
            Self::Rpc { code, message } => write!(f, "acp rpc error {code}: {message}"),
            Self::Eof => write!(f, "acp stream ended"),
        }
    }
}

impl std::error::Error for AcpError {}

impl From<io::Error> for AcpError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// The request surface a host exposes. Sealed behind a trait so `AcpDriver` can
/// be unit-tested against a mock without spawning a process.
pub trait AcpClient: Send + Sync {
    fn request(&self, method: &str, params: Value) -> Result<Value, AcpError>;
    fn try_recv_notification(&self) -> Option<JsonRpcNotification>;
}

/// The result routed back to a pending request by the reader thread.
struct PendingResult {
    result: Option<Value>,
    error: Option<super::protocol::RpcError>,
}

/// A connected ACP stream: a buffered reader and a writer, as two owned halves
/// so the background reader thread and the request writer never contend for a
/// single `&mut`.
pub struct AcpStream {
    pub reader: Box<dyn BufRead + Send>,
    pub writer: Box<dyn Write + Send>,
}

/// A live ACP host. Owns the child process, the reader thread, pending-request
/// correlation, and the notification queue.
pub struct AcpHost {
    writer: Mutex<Box<dyn Write + Send>>,
    next_id: AtomicI64,
    pending: Arc<Mutex<HashMap<i64, mpsc::Sender<PendingResult>>>>,
    notifications: Mutex<Receiver<JsonRpcNotification>>,
    reader_handle: Option<JoinHandle<()>>,
    child: Option<Child>,
}

impl AcpHost {
    /// Spawn `program` (plus `args`) as an ACP server and connect over stdio.
    pub fn spawn(program: &str, args: &[String]) -> io::Result<Self> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "acp child has no stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "acp child has no stdout"))?;
        Ok(Self::from_stream(
            Box::new(BufReader::new(stdout)),
            Box::new(stdin),
            Some(child),
        ))
    }

    /// Build a host from an already-connected reader/writer pair (no child).
    /// Used by tests to inject an in-process transport.
    pub fn from_stream(
        reader: Box<dyn BufRead + Send>,
        writer: Box<dyn Write + Send>,
        child: Option<Child>,
    ) -> Self {
        let pending: Arc<Mutex<HashMap<i64, mpsc::Sender<PendingResult>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let (notif_tx, notifications) = mpsc::channel();

        let pending_for_reader = Arc::clone(&pending);
        let reader_handle = std::thread::spawn(move || {
            let mut reader = reader;
            loop {
                match frame::read_line(&mut reader) {
                    Ok(Some(line)) => match classify_inbound(&line) {
                        Ok(InboundMessage::Response(resp)) => {
                            let tx = pending_for_reader
                                .lock()
                                .expect("pending lock")
                                .remove(&resp.id);
                            if let Some(tx) = tx {
                                let _ = tx.send(PendingResult {
                                    result: resp.result,
                                    error: resp.error,
                                });
                            }
                        }
                        Ok(InboundMessage::Notification(notif)) => {
                            let _ = notif_tx.send(notif);
                        }
                        Ok(InboundMessage::Request(_)) => {
                            // Server-initiated requests (e.g. fs/read_text_file)
                            // are out of scope for this slice and are dropped.
                        }
                        Err(_) => {
                            // Malformed frame: keep the loop alive rather than
                            // crashing the whole host on one bad line.
                        }
                    },
                    Ok(None) | Err(_) => break,
                }
            }
        });

        Self {
            writer: Mutex::new(writer),
            next_id: AtomicI64::new(0),
            pending,
            notifications: Mutex::new(notifications),
            reader_handle: Some(reader_handle),
            child,
        }
    }

    /// Perform the ACP `initialize` handshake and return the result params.
    pub fn initialize(&self) -> Result<Value, AcpError> {
        self.request(
            super::protocol::METHOD_INITIALIZE,
            serde_json::json!({ "protocolVersion": 1 }),
        )
    }

    /// Return the child process handle, if one was spawned.
    pub fn child_mut(&mut self) -> Option<&mut Child> {
        self.child.as_mut()
    }
}

impl AcpClient for AcpHost {
    fn request(&self, method: &str, params: Value) -> Result<Value, AcpError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let request = JsonRpcRequest::new(id, method, params);

        let (tx, rx) = mpsc::channel();
        self.pending.lock().expect("pending lock").insert(id, tx);

        let bytes = frame::encode(&request).map_err(|e| AcpError::Protocol(e.to_string()))?;
        {
            let mut writer = self.writer.lock().expect("writer lock");
            writer.write_all(&bytes)?;
            writer.flush()?;
        }

        match rx.recv() {
            Ok(PendingResult {
                result,
                error: None,
            }) => Ok(result.unwrap_or(Value::Null)),
            Ok(PendingResult {
                result: _,
                error: Some(e),
            }) => Err(AcpError::Rpc {
                code: e.code,
                message: e.message,
            }),
            Err(_) => Err(AcpError::Eof),
        }
    }

    fn try_recv_notification(&self) -> Option<JsonRpcNotification> {
        self.notifications
            .lock()
            .expect("notifications lock")
            .try_recv()
            .ok()
    }
}

impl Drop for AcpHost {
    fn drop(&mut self) {
        // Kill the child first so its stdout closes and the reader thread sees
        // EOF and exits; then reap the child and join the reader.
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A trivial mock used by driver tests, not exercised here.
    #[allow(dead_code)]
    struct MockClient;

    impl AcpClient for MockClient {
        fn request(&self, _method: &str, _params: Value) -> Result<Value, AcpError> {
            Ok(Value::Null)
        }
        fn try_recv_notification(&self) -> Option<JsonRpcNotification> {
            None
        }
    }
}
