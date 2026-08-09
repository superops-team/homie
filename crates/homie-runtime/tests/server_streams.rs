#![cfg(unix)]

use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use homie_client::{ClientOptions, EventStreamItem, HomieClient, StreamState, TerminalItem};
use homie_proto::model::{RuntimeEvent, StateSnapshot};
use homie_proto::paths::RuntimeEndpoint;
use homie_proto::stream::{
    EventStreamOpen, StreamKind, StreamOpenRequest, StreamReset, StreamResetReason,
    TerminalStreamOpen,
};
use homie_proto::transport::{
    ClientRole, EndpointRole, Frame, FrameHeader, FrameKind, HelloRequest, HelloResponse, Preface,
    WIRE_MAJOR, WIRE_MINOR,
};
use homie_proto::{ControlMessage, Method, RequestId};
use homie_runtime::event_stream::EventStore;
use homie_runtime::runtime_actor::ServiceError;
use homie_runtime::{
    ControlFuture, ControlHandler, RuntimeServer, RuntimeStreamHandler, ServerConfig,
    ServerIdentity, TerminalBackend, TerminalSourceDescriptor, TerminalSourceStats,
    TerminalStreamError,
};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::task::JoinHandle;
use tokio::time::timeout;

const IO_TIMEOUT: Duration = Duration::from_secs(2);

#[tokio::test]
async fn production_handler_advertises_exact_registry_and_dynamic_event_bounds() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = Arc::new(EventStore::open(temp.path().to_path_buf()).expect("event store"));
    let backend = Arc::new(FakeTerminalBackend::default());
    let handler = RuntimeStreamHandler::new(store.clone(), backend);

    assert_eq!(
        homie_runtime::StreamHandler::capabilities(&handler),
        vec![StreamKind::EventsV1, StreamKind::TerminalV1]
    );
    assert_eq!(
        homie_runtime::StreamHandler::event_bounds(&handler),
        store.bounds()
    );
}

#[tokio::test]
async fn homie_client_replays_events_over_real_uds() {
    let server = StreamServer::start().await;
    let client = server.connect().await;

    let mut stream = client
        .subscribe_events(EventStreamOpen {
            after_seq: 1,
            event_filter: Vec::new(),
        })
        .await
        .expect("event stream");
    let EventStreamItem::Event(event) = stream
        .recv()
        .await
        .expect("event receive")
        .expect("event item")
    else {
        panic!("expected replayed event");
    };

    assert_eq!(
        event,
        RuntimeEvent {
            seq: 2,
            event: "session.updated".to_string(),
            session_id: Some("session-1".to_string()),
            status: None,
        }
    );
    client.close().await.expect("client close");
}

#[tokio::test]
async fn homie_client_accepts_filtered_event_sequence_gaps_over_real_uds() {
    let server = StreamServer::start_with(
        vec![event(1, "keep"), event(2, "ignore"), event(3, "keep")],
        None,
    )
    .await;
    let client = server.connect().await;
    let mut stream = client
        .subscribe_events(EventStreamOpen {
            after_seq: 0,
            event_filter: vec!["keep".to_string()],
        })
        .await
        .expect("event stream");

    let first = timeout(IO_TIMEOUT, stream.recv())
        .await
        .expect("first event timeout")
        .expect("first event receive")
        .expect("first event");
    let second = timeout(IO_TIMEOUT, stream.recv())
        .await
        .expect("second event timeout")
        .expect("second event receive")
        .expect("second event");

    assert!(matches!(
        (first, second),
        (
            EventStreamItem::Event(RuntimeEvent { seq: 1, .. }),
            EventStreamItem::Event(RuntimeEvent { seq: 3, .. })
        )
    ));
    client.close().await.expect("client close");
}

#[tokio::test]
async fn hello_reads_current_event_bounds_from_stream_handler() {
    let server = StreamServer::start().await;

    let (_socket, hello) = server.raw_connect().await;

    assert_eq!((hello.event_oldest_seq, hello.event_latest_seq), (1, 2));
}

#[tokio::test]
async fn homie_client_recovers_too_old_event_cursor_with_snapshot_and_same_stream_reopen() {
    let events = (1..=1025)
        .map(|seq| event(seq, "session.updated"))
        .collect();
    let server = StreamServer::start_with(events, None).await;
    let client = server.connect().await;
    let mut stream = client
        .subscribe_events(EventStreamOpen {
            after_seq: 0,
            event_filter: Vec::new(),
        })
        .await
        .expect("event stream");

    let item = timeout(IO_TIMEOUT, stream.recv())
        .await
        .expect("snapshot timeout")
        .expect("snapshot receive")
        .expect("snapshot item");
    let EventStreamItem::Snapshot(snapshot) = item else {
        panic!("expected recovery snapshot");
    };
    let mut state = stream.state();
    timeout(IO_TIMEOUT, async {
        loop {
            if *state.borrow() == StreamState::Open {
                break;
            }
            state.changed().await.expect("event stream state");
        }
    })
    .await
    .expect("same stream reopen timeout");

    assert_eq!(snapshot.event_cursor, 1025);
    client.close().await.expect("client close");
}

#[tokio::test]
async fn server_reset_producer_is_pruned_so_same_stream_id_can_reopen() {
    let events = (1..=1025)
        .map(|seq| event(seq, "session.updated"))
        .collect();
    let server = StreamServer::start_with(events, None).await;
    let (mut socket, _) = server.raw_connect().await;

    write_frame(&mut socket, event_open_frame(1, 0)).await;
    assert_eq!(
        read_frame(&mut socket).await.header.kind,
        FrameKind::StreamOpened
    );
    let reset = read_frame(&mut socket).await;
    assert_eq!(reset.header.kind, FrameKind::StreamReset);

    write_frame(&mut socket, event_open_frame(1, 1025)).await;
    let reopened = read_frame(&mut socket).await;

    assert_eq!(reopened.header.kind, FrameKind::StreamOpened);
    assert_eq!(reopened.header.stream_id, 1);
}

#[tokio::test]
async fn finished_server_reset_stream_does_not_consume_active_stream_capacity() {
    let events = (1..=1025)
        .map(|seq| event(seq, "session.updated"))
        .collect();
    let server = StreamServer::start_with(events, None).await;
    let (mut socket, _) = server.raw_connect().await;
    write_frame(&mut socket, event_open_frame(1, 0)).await;
    assert_eq!(
        read_frame(&mut socket).await.header.kind,
        FrameKind::StreamOpened
    );
    assert_eq!(
        read_frame(&mut socket).await.header.kind,
        FrameKind::StreamReset
    );

    for index in 0..64_u32 {
        let stream_id = index * 2 + 3;
        write_frame(&mut socket, event_open_frame(stream_id, 1025)).await;
        let opened = read_frame(&mut socket).await;
        assert_eq!(opened.header.stream_id, stream_id);
    }

    write_frame(&mut socket, state_snapshot_request(1)).await;
    assert_eq!(
        read_frame(&mut socket).await.header.kind,
        FrameKind::Response
    );
}

#[tokio::test]
async fn connection_accepts_64_active_streams_and_rejects_the_65th() {
    let server = StreamServer::start().await;
    let (mut socket, _) = server.raw_connect().await;
    for index in 0..64_u32 {
        let stream_id = index * 2 + 1;
        write_frame(&mut socket, event_open_frame(stream_id, 2)).await;
        let opened = read_frame(&mut socket).await;
        assert_eq!(opened.header.stream_id, stream_id);
    }

    write_frame(&mut socket, event_open_frame(129, 2)).await;

    assert_closed(&mut socket).await;
}

#[tokio::test]
async fn reused_active_stream_id_is_a_connection_protocol_error() {
    let server = StreamServer::start().await;
    let (mut socket, _) = server.raw_connect().await;
    write_frame(&mut socket, event_open_frame(1, 2)).await;
    assert_eq!(
        read_frame(&mut socket).await.header.kind,
        FrameKind::StreamOpened
    );

    write_frame(&mut socket, event_open_frame(1, 2)).await;

    assert_closed(&mut socket).await;
}

#[tokio::test]
async fn event_stream_rejects_input_as_connection_protocol_error() {
    let server = StreamServer::start().await;
    let (mut socket, _) = server.raw_connect().await;
    write_frame(&mut socket, event_open_frame(1, 2)).await;
    assert_eq!(
        read_frame(&mut socket).await.header.kind,
        FrameKind::StreamOpened
    );

    write_frame(
        &mut socket,
        client_stream_frame(FrameKind::Input, 1, b"x".to_vec()),
    )
    .await;

    assert_closed(&mut socket).await;
}

#[tokio::test]
async fn homie_client_terminal_replay_grid_input_and_resize_use_real_uds() {
    let server = StreamServer::start_terminal(b"hi").await;
    let client = server.connect().await;
    let mut terminal = client
        .open_terminal(terminal_open_request())
        .await
        .expect("terminal stream");
    let mut items = Vec::new();
    for _ in 0..5 {
        items.push(
            timeout(IO_TIMEOUT, terminal.recv())
                .await
                .expect("terminal item timeout")
                .expect("terminal receive")
                .expect("terminal item"),
        );
    }

    assert!(matches!(items[0], TerminalItem::ReplayBegin(0)));
    assert!(matches!(
        &items[1],
        TerminalItem::Output { offset: 0, bytes } if bytes == b"hi"
    ));
    assert!(matches!(items[2], TerminalItem::ReplayEnd(2)));
    assert!(matches!(
        &items[3],
        TerminalItem::Grid(update)
            if update.is_full_snapshot && update.cols == 4 && update.rows == 2
    ));
    assert!(matches!(&items[4], TerminalItem::Modes(modes) if modes == &[0x11, 0x22]));

    terminal
        .send_input(vec![0x00, 0xff, 0x80, b'\n'])
        .expect("raw input");
    terminal.resize(0x012c, 0x0101).expect("big-endian resize");
    timeout(IO_TIMEOUT, async {
        loop {
            if server.terminal.inputs.lock().expect("inputs").len() == 1
                && server.terminal.resizes.lock().expect("resizes").len() == 1
            {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("terminal backend calls");

    assert_eq!(
        server.terminal.inputs.lock().expect("inputs").as_slice(),
        &[("session-1".to_string(), vec![0x00, 0xff, 0x80, b'\n'])]
    );
    assert_eq!(
        server.terminal.resizes.lock().expect("resizes").as_slice(),
        &[("session-1".to_string(), 0x012c, 0x0101)]
    );
    client.close().await.expect("client close");
}

#[tokio::test]
async fn homie_client_adopts_server_clamped_replay_offset_over_real_uds() {
    let server = StreamServer::start_terminal(b"hi").await;
    let client = server.connect().await;
    let mut terminal = client
        .open_terminal(TerminalStreamOpen {
            output_offset: 100,
            ..terminal_open_request()
        })
        .await
        .expect("terminal stream");

    let begin = timeout(IO_TIMEOUT, terminal.recv())
        .await
        .expect("replay begin timeout")
        .expect("replay begin receive")
        .expect("replay begin");

    assert_eq!(begin, TerminalItem::ReplayBegin(2));
    assert_eq!(terminal.last_confirmed_offset(), 2);
    client.close().await.expect("client close");
}

#[tokio::test]
async fn two_terminal_streams_share_one_source_and_drop_preserves_control() {
    let server = StreamServer::start_terminal(b"").await;
    let client = server.connect().await;
    let mut first = client
        .open_terminal(terminal_open_request())
        .await
        .expect("first terminal");
    drain_terminal_items(&mut first, 4).await;
    let mut second = client
        .open_terminal(terminal_open_request())
        .await
        .expect("second terminal");
    drain_terminal_items(&mut second, 4).await;

    assert_eq!(server.terminal.describe_calls.load(Ordering::Relaxed), 1);
    assert_eq!(
        server.streams.terminal_stats().await,
        TerminalSourceStats {
            source_count: 1,
            output_log_readers: 1,
            subscriber_count: 2,
        }
    );

    drop(first);
    drop(second);
    wait_for_stats(&server, TerminalSourceStats::default()).await;
    let snapshot: StateSnapshot = client
        .request(Method::STATE_SNAPSHOT, json!({}))
        .await
        .expect("control after stream drop");

    assert_eq!(snapshot.event_cursor, 0);
    client.close().await.expect("client close");
}

#[tokio::test]
async fn stream_close_detaches_terminal_source_and_preserves_control() {
    assert_client_stream_detach(FrameKind::StreamClose).await;
}

#[tokio::test]
async fn valid_client_stream_reset_detaches_terminal_source_and_preserves_control() {
    assert_client_stream_detach(FrameKind::StreamReset).await;
}

#[tokio::test]
async fn disconnect_drops_terminal_source_without_terminating_server() {
    let server = StreamServer::start_terminal(b"").await;
    let (mut socket, _) = server.raw_connect().await;
    write_frame(&mut socket, terminal_open_frame(1)).await;
    drain_raw_terminal(&mut socket, 5).await;

    drop(socket);
    wait_for_stats(&server, TerminalSourceStats::default()).await;
    let (mut control, _) = server.raw_connect().await;
    write_frame(&mut control, state_snapshot_request(1)).await;

    assert_eq!(
        read_frame(&mut control).await.header.kind,
        FrameKind::Response
    );
}

#[tokio::test]
async fn terminal_actor_backpressure_resets_only_stream_as_slow_consumer() {
    assert_terminal_input_error_is_stream_local(
        TerminalStreamError::Backpressure,
        StreamResetReason::SlowConsumer,
    )
    .await;
}

#[tokio::test]
async fn terminal_backend_unavailable_resets_only_stream_as_protocol_error() {
    assert_terminal_input_error_is_stream_local(
        TerminalStreamError::Backend,
        StreamResetReason::ProtocolError,
    )
    .await;
}

#[tokio::test]
async fn invalid_terminal_resize_is_a_connection_protocol_error() {
    let server = StreamServer::start_terminal(b"").await;
    let (mut socket, _) = server.raw_connect().await;
    write_frame(&mut socket, terminal_open_frame(1)).await;
    drain_raw_terminal(&mut socket, 5).await;

    write_frame(
        &mut socket,
        client_stream_frame(FrameKind::Resize, 1, vec![0, 0, 0, 40]),
    )
    .await;

    assert_closed(&mut socket).await;
}

#[tokio::test]
async fn stream_close_with_nonzero_sequence_is_a_connection_protocol_error() {
    let server = StreamServer::start_terminal(b"").await;
    let (mut socket, _) = server.raw_connect().await;
    write_frame(&mut socket, terminal_open_frame(1)).await;
    drain_raw_terminal(&mut socket, 5).await;
    let mut close = client_stream_frame(FrameKind::StreamClose, 1, Vec::new());
    close.header.sequence = 1;

    write_frame(&mut socket, close).await;

    assert_closed(&mut socket).await;
}

#[derive(Default)]
struct FakeTerminalBackend {
    descriptors: Mutex<HashMap<String, TerminalSourceDescriptor>>,
    describe_calls: AtomicUsize,
    inputs: Mutex<Vec<(String, Vec<u8>)>>,
    resizes: Mutex<Vec<(String, u16, u16)>>,
    input_failure: AtomicU8,
}

impl FakeTerminalBackend {
    fn insert(&self, descriptor: TerminalSourceDescriptor) {
        self.descriptors
            .lock()
            .expect("descriptors")
            .insert(descriptor.session_id.clone(), descriptor);
    }

    fn fail_input_with(&self, error: TerminalStreamError) {
        let code = match error {
            TerminalStreamError::Backpressure => 1,
            TerminalStreamError::Backend => 2,
            _ => panic!("unsupported fake input error"),
        };
        self.input_failure.store(code, Ordering::Release);
    }
}

impl TerminalBackend for FakeTerminalBackend {
    async fn describe(
        &self,
        session_id: &str,
    ) -> Result<TerminalSourceDescriptor, TerminalStreamError> {
        self.describe_calls.fetch_add(1, Ordering::Relaxed);
        self.descriptors
            .lock()
            .expect("descriptors")
            .get(session_id)
            .cloned()
            .ok_or(TerminalStreamError::Backend)
    }

    async fn send_input(
        &self,
        session_id: &str,
        input: Vec<u8>,
    ) -> Result<(), TerminalStreamError> {
        match self.input_failure.load(Ordering::Acquire) {
            1 => return Err(TerminalStreamError::Backpressure),
            2 => return Err(TerminalStreamError::Backend),
            _ => {}
        }
        self.inputs
            .lock()
            .expect("inputs")
            .push((session_id.to_string(), input));
        Ok(())
    }

    async fn resize(
        &self,
        session_id: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(), TerminalStreamError> {
        self.resizes
            .lock()
            .expect("resizes")
            .push((session_id.to_string(), cols, rows));
        Ok(())
    }
}

#[derive(Clone)]
struct FakeControlHandler {
    snapshot: StateSnapshot,
}

impl ControlHandler for FakeControlHandler {
    fn handle<'a>(&'a self, method: &'a str, _params: Value) -> ControlFuture<'a> {
        Box::pin(async move {
            match method {
                Method::STATE_SNAPSHOT => {
                    serde_json::to_value(&self.snapshot).map_err(|_| ServiceError::Internal)
                }
                _ => Err(ServiceError::MethodNotFound(method.to_string())),
            }
        })
    }
}

struct StreamServer {
    _temp: TempDir,
    socket_path: PathBuf,
    terminal: Arc<FakeTerminalBackend>,
    streams: Arc<RuntimeStreamHandler<FakeTerminalBackend>>,
    task: JoinHandle<Result<(), homie_runtime::server::ServerError>>,
}

impl StreamServer {
    async fn start() -> Self {
        Self::start_with(
            vec![event(1, "session.created"), event(2, "session.updated")],
            None,
        )
        .await
    }

    async fn start_terminal(output: &[u8]) -> Self {
        Self::start_with(Vec::new(), Some(output)).await
    }

    async fn start_with(events: Vec<RuntimeEvent>, terminal_output: Option<&[u8]>) -> Self {
        let temp = tempfile::tempdir().expect("tempdir");
        let socket_path = temp.path().join("runtime.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind UDS");
        let runtime_dir = temp.path().join("runtime");
        std::fs::create_dir_all(&runtime_dir).expect("runtime dir");
        write_event_log(&runtime_dir.join("events.jsonl"), &events);
        let event_store =
            Arc::new(EventStore::open(temp.path().to_path_buf()).expect("event store"));
        let terminal = Arc::new(FakeTerminalBackend::default());
        if let Some(output) = terminal_output {
            let output_path = runtime_dir.join("terminal.log");
            std::fs::write(&output_path, output).expect("terminal output");
            terminal.insert(TerminalSourceDescriptor {
                session_id: "session-1".to_string(),
                output_path,
                cols: 4,
                rows: 2,
                modes: vec![0x11, 0x22],
            });
        }
        let streams = Arc::new(RuntimeStreamHandler::new(
            event_store.clone(),
            terminal.clone(),
        ));
        let snapshot = StateSnapshot {
            sessions: Vec::new(),
            event_cursor: event_store.bounds().latest_seq,
        };
        let runtime_server = Arc::new(RuntimeServer::new(
            ServerConfig::current_user(),
            ServerIdentity {
                daemon_build: "test-build".to_string(),
                daemon_version: "1.0.0".to_string(),
                daemon_pid: std::process::id(),
                daemon_instance_id: "stream-test".to_string(),
                executable_hash: "sha256:test".to_string(),
            },
            Arc::new(FakeControlHandler { snapshot }),
            streams.clone(),
        ));
        let task = tokio::spawn(runtime_server.serve_listener(listener));
        Self {
            _temp: temp,
            socket_path,
            terminal,
            streams,
            task,
        }
    }

    async fn connect(&self) -> HomieClient {
        HomieClient::connect(ClientOptions {
            endpoint: RuntimeEndpoint::new(self.socket_path.clone()).expect("runtime endpoint"),
            role: ClientRole::Cli,
            connect_timeout: Duration::from_secs(2),
            request_timeout: Duration::from_secs(2),
        })
        .await
        .expect("connect client")
    }

    async fn raw_connect(&self) -> (UnixStream, HelloResponse) {
        let mut socket = UnixStream::connect(&self.socket_path)
            .await
            .expect("connect raw client");
        socket
            .write_all(
                &Preface {
                    major: WIRE_MAJOR,
                    minor: WIRE_MINOR,
                }
                .encode(),
            )
            .await
            .expect("preface");
        write_frame(
            &mut socket,
            Frame {
                header: FrameHeader {
                    version: WIRE_MAJOR,
                    kind: FrameKind::Hello,
                    flags: 0,
                    stream_id: 0,
                    message_id: 0,
                    sequence: 0,
                },
                payload: serde_json::to_vec(&HelloRequest {
                    wire_major: WIRE_MAJOR,
                    wire_minor: WIRE_MINOR,
                    client_name: "server-streams-test".to_string(),
                    client_version: "1.0.0".to_string(),
                    client_role: ClientRole::Cli,
                    process_id: std::process::id(),
                })
                .expect("hello payload"),
            },
        )
        .await;
        let hello = read_frame(&mut socket).await;
        assert_eq!(hello.header.kind, FrameKind::HelloAck);
        (
            socket,
            serde_json::from_slice(&hello.payload).expect("hello response"),
        )
    }
}

impl Drop for StreamServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

fn event(seq: u64, name: &str) -> RuntimeEvent {
    RuntimeEvent {
        seq,
        event: name.to_string(),
        session_id: Some("session-1".to_string()),
        status: None,
    }
}

fn write_event_log(path: &std::path::Path, events: &[RuntimeEvent]) {
    let mut file = std::fs::File::create(path).expect("event log");
    for event in events {
        serde_json::to_writer(&mut file, event).expect("event JSON");
        file.write_all(b"\n").expect("event newline");
    }
}

fn event_open_frame(stream_id: u32, after_seq: u64) -> Frame {
    stream_open_frame(
        stream_id,
        StreamOpenRequest::Events(EventStreamOpen {
            after_seq,
            event_filter: Vec::new(),
        }),
    )
}

fn terminal_open_frame(stream_id: u32) -> Frame {
    stream_open_frame(
        stream_id,
        StreamOpenRequest::Terminal(terminal_open_request()),
    )
}

fn terminal_open_request() -> TerminalStreamOpen {
    TerminalStreamOpen {
        session_id: "session-1".to_string(),
        output_offset: 0,
        client_role: ClientRole::Cli,
        last_grid_sequence: None,
    }
}

fn stream_open_frame(stream_id: u32, request: StreamOpenRequest) -> Frame {
    Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::StreamOpen,
            flags: 0,
            stream_id,
            message_id: 0,
            sequence: 0,
        },
        payload: serde_json::to_vec(&request).expect("stream open payload"),
    }
}

fn client_stream_frame(kind: FrameKind, stream_id: u32, payload: Vec<u8>) -> Frame {
    Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind,
            flags: 0,
            stream_id,
            message_id: 0,
            sequence: 0,
        },
        payload,
    }
}

fn state_snapshot_request(message_id: u64) -> Frame {
    Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::Request,
            flags: 0,
            stream_id: 0,
            message_id,
            sequence: 0,
        },
        payload: serde_json::to_vec(&ControlMessage::request(
            RequestId::from(message_id),
            Method::STATE_SNAPSHOT,
            json!({}),
        ))
        .expect("state snapshot request"),
    }
}

async fn drain_terminal_items(stream: &mut homie_client::TerminalStream, count: usize) {
    for _ in 0..count {
        timeout(IO_TIMEOUT, stream.recv())
            .await
            .expect("terminal item timeout")
            .expect("terminal receive")
            .expect("terminal item");
    }
}

async fn drain_raw_terminal(socket: &mut UnixStream, count: usize) {
    for _ in 0..count {
        read_frame(socket).await;
    }
}

async fn wait_for_stats(server: &StreamServer, expected: TerminalSourceStats) {
    timeout(IO_TIMEOUT, async {
        loop {
            if server.streams.terminal_stats().await == expected {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("terminal source stats timeout");
}

async fn assert_client_stream_detach(kind: FrameKind) {
    let server = StreamServer::start_terminal(b"").await;
    let (mut socket, _) = server.raw_connect().await;
    write_frame(&mut socket, terminal_open_frame(1)).await;
    drain_raw_terminal(&mut socket, 5).await;
    let payload = if kind == FrameKind::StreamReset {
        serde_json::to_vec(&StreamReset {
            reason: StreamResetReason::ProtocolError,
            last_confirmed_offset: None,
            latest_seq: None,
        })
        .expect("stream reset")
    } else {
        Vec::new()
    };

    write_frame(&mut socket, client_stream_frame(kind, 1, payload)).await;
    wait_for_stats(&server, TerminalSourceStats::default()).await;
    write_frame(&mut socket, state_snapshot_request(1)).await;

    assert_eq!(
        read_frame(&mut socket).await.header.kind,
        FrameKind::Response
    );
}

async fn assert_terminal_input_error_is_stream_local(
    backend_error: TerminalStreamError,
    expected_reason: StreamResetReason,
) {
    let server = StreamServer::start_terminal(b"").await;
    server.terminal.fail_input_with(backend_error);
    let (mut socket, _) = server.raw_connect().await;
    write_frame(&mut socket, terminal_open_frame(1)).await;
    drain_raw_terminal(&mut socket, 5).await;

    write_frame(
        &mut socket,
        client_stream_frame(FrameKind::Input, 1, vec![0xff]),
    )
    .await;
    let reset = read_frame(&mut socket).await;
    let payload: StreamReset = serde_json::from_slice(&reset.payload).expect("stream reset");

    assert_eq!(reset.header.kind, FrameKind::StreamReset);
    assert_eq!(payload.reason, expected_reason);
    wait_for_stats(&server, TerminalSourceStats::default()).await;
    write_frame(&mut socket, state_snapshot_request(1)).await;
    assert_eq!(
        read_frame(&mut socket).await.header.kind,
        FrameKind::Response
    );
}

async fn write_frame(socket: &mut UnixStream, frame: Frame) {
    let encoded = frame
        .encode(EndpointRole::Client)
        .expect("encode client frame");
    socket
        .write_all(&encoded)
        .await
        .expect("write client frame");
}

async fn read_frame(socket: &mut UnixStream) -> Frame {
    timeout(IO_TIMEOUT, async {
        let mut length = [0_u8; 4];
        socket.read_exact(&mut length).await.expect("frame length");
        let frame_len = u32::from_be_bytes(length) as usize;
        let mut encoded = vec![0_u8; 4 + frame_len];
        encoded[..4].copy_from_slice(&length);
        socket
            .read_exact(&mut encoded[4..])
            .await
            .expect("frame body");
        Frame::decode(&encoded, EndpointRole::Server)
            .expect("decode server frame")
            .expect("complete server frame")
            .0
    })
    .await
    .expect("server frame timeout")
}

async fn assert_closed(socket: &mut UnixStream) {
    let mut byte = [0_u8; 1];
    let result = timeout(IO_TIMEOUT, socket.read(&mut byte))
        .await
        .expect("connection close timeout");
    match result {
        Ok(0) => {}
        Ok(read) => panic!("expected EOF, read {read} bytes"),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::BrokenPipe
                    | std::io::ErrorKind::NotConnected
            ) => {}
        Err(error) => panic!("unexpected read error: {error}"),
    }
}
