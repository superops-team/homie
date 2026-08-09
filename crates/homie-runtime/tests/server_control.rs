#![cfg(unix)]

use std::future::pending;
use std::sync::Arc;
use std::time::Duration;

use homie_proto::stream::{EventStreamOpen, StreamKind, StreamOpenRequest};
use homie_proto::transport::{
    ClientRole, EndpointRole, FRAME_HEADER_LEN, Frame, FrameHeader, FrameKind, HelloRequest,
    HelloResponse, MAX_CONTROL_PAYLOAD, MAX_FRAME_LEN, MAX_OUTPUT_PAYLOAD, Preface,
    StableErrorCode, WIRE_MAJOR, WIRE_MINOR,
};
use homie_proto::{ControlMessage, Method, RequestId};
use homie_runtime::capabilities::{method_capabilities, stream_capabilities};
use homie_runtime::runtime_actor::{ServiceError, ServiceResult};
use homie_runtime::server::ShutdownHandle;
use homie_runtime::{
    ActiveStream, ActiveStreamFuture, ControlFuture, ControlHandler, EventBounds,
    RuntimeDispatcher, RuntimeServer, ServerConfig, ServerIdentity, StreamError, StreamFuture,
    StreamHandler,
};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};

const IO_TIMEOUT: Duration = Duration::from_secs(2);

#[tokio::test]
async fn real_uds_accepts_peer_and_returns_exact_hello_identity_and_registries() {
    let server = TestServer::start(FakeHandler).await;
    let (_socket, hello) = server.connect_and_handshake().await;
    let expected_methods = method_capabilities()
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();

    assert_eq!(hello.wire_major, WIRE_MAJOR);
    assert_eq!(hello.wire_minor, WIRE_MINOR);
    assert_eq!(hello.daemon_build, server.identity.daemon_build);
    assert_eq!(hello.daemon_version, server.identity.daemon_version);
    assert_eq!(hello.daemon_pid, server.identity.daemon_pid);
    assert_eq!(hello.daemon_instance_id, server.identity.daemon_instance_id);
    assert_eq!(hello.executable_hash, server.identity.executable_hash);
    assert_eq!(hello.method_capabilities, expected_methods);
    assert_eq!(
        hello.stream_capabilities,
        vec![StreamKind::EventsV1, StreamKind::TerminalV1]
    );
    assert_eq!(hello.event_oldest_seq, 41);
    assert_eq!(hello.event_latest_seq, 99);
}

#[test]
fn runtime_dispatcher_implements_async_control_handler_adapter() {
    fn assert_handler<T: ControlHandler>() {}

    assert_handler::<RuntimeDispatcher>();
}

#[tokio::test]
async fn peer_uid_mismatch_is_rejected_before_protocol_read() {
    let config = ServerConfig {
        expected_peer_uid: ServerConfig::current_process_uid().wrapping_add(1),
    };
    let server = TestServer::start_with_config(FakeHandler, config).await;
    let mut socket = UnixStream::connect(&server.socket_path)
        .await
        .expect("connect");

    assert_closed(&mut socket).await;
}

#[tokio::test]
async fn frame_without_preface_is_rejected() {
    let server = TestServer::start(FakeHandler).await;
    let mut socket = UnixStream::connect(&server.socket_path)
        .await
        .expect("connect");

    write_frame(&mut socket, hello_frame(valid_hello()))
        .await
        .expect("write hello without preface");

    assert_closed(&mut socket).await;
}

#[tokio::test]
async fn first_frame_after_preface_must_be_hello() {
    let server = TestServer::start(FakeHandler).await;
    let mut socket = UnixStream::connect(&server.socket_path)
        .await
        .expect("connect");
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

    write_frame(&mut socket, request_frame(1, 1, "echo", json!({})))
        .await
        .expect("request first");

    assert_closed(&mut socket).await;
}

#[tokio::test]
async fn preface_major_mismatch_is_rejected() {
    let server = TestServer::start(FakeHandler).await;
    let mut socket = UnixStream::connect(&server.socket_path)
        .await
        .expect("connect");
    socket
        .write_all(
            &Preface {
                major: WIRE_MAJOR + 1,
                minor: WIRE_MINOR,
            }
            .encode(),
        )
        .await
        .expect("preface");

    assert_closed(&mut socket).await;
}

#[tokio::test]
async fn preface_minor_above_server_minor_is_rejected() {
    let server = TestServer::start(FakeHandler).await;
    let mut socket = UnixStream::connect(&server.socket_path)
        .await
        .expect("connect");
    socket
        .write_all(
            &Preface {
                major: WIRE_MAJOR,
                minor: WIRE_MINOR + 1,
            }
            .encode(),
        )
        .await
        .expect("preface");

    assert_closed(&mut socket).await;
}

#[tokio::test]
async fn hello_wire_fields_must_match_preface_and_server_version() {
    let server = TestServer::start(FakeHandler).await;
    let mut socket = connect_prefaced(&server.socket_path).await;
    let mut hello = valid_hello();
    hello.wire_major += 1;

    write_frame(&mut socket, hello_frame(hello))
        .await
        .expect("hello");

    assert_closed(&mut socket).await;
}

#[tokio::test]
async fn hello_identity_fields_must_be_non_empty_and_pid_non_zero() {
    let server = TestServer::start(FakeHandler).await;
    let mut socket = connect_prefaced(&server.socket_path).await;
    let hello = HelloRequest {
        client_name: String::new(),
        process_id: 0,
        ..valid_hello()
    };

    write_frame(&mut socket, hello_frame(hello))
        .await
        .expect("hello");

    assert_closed(&mut socket).await;
}

#[tokio::test]
async fn concurrent_requests_respond_in_completion_order_with_correlated_ids() {
    let server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;
    write_frame(
        &mut socket,
        request_frame(11, 11, "delayed", json!({"delayMs": 80, "value": "slow"})),
    )
    .await
    .expect("slow request");
    write_frame(
        &mut socket,
        request_frame(22, 22, "delayed", json!({"delayMs": 1, "value": "fast"})),
    )
    .await
    .expect("fast request");

    let first = read_response(&mut socket).await;
    let second = read_response(&mut socket).await;

    assert_eq!(first.message_id, 22);
    assert_eq!(first.request_id, 22);
    assert_eq!(first.result, Some(json!("fast")));
    assert_eq!(second.message_id, 11);
    assert_eq!(second.request_id, 11);
    assert_eq!(second.result, Some(json!("slow")));
}

#[tokio::test]
async fn successful_daemon_shutdown_flushes_response_before_eof_and_server_exit() {
    let mut server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;

    write_frame(
        &mut socket,
        request_frame(31, 31, Method::DAEMON_SHUTDOWN, json!({})),
    )
    .await
    .expect("shutdown request");
    let response = read_response(&mut socket).await;

    assert_eq!(response.result, Some(json!({"shutdown": true})));
    assert_closed(&mut socket).await;
    server.wait_for_exit().await;
}

#[tokio::test]
async fn daemon_shutdown_flushes_ack_then_cancels_pending_long_poll() {
    let mut server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;
    write_frame(
        &mut socket,
        request_frame(32, 32, Method::EVENTS_WAIT, json!({})),
    )
    .await
    .expect("pending long poll");
    write_frame(
        &mut socket,
        request_frame(33, 33, Method::DAEMON_SHUTDOWN, json!({})),
    )
    .await
    .expect("shutdown request");

    let shutdown = read_response(&mut socket).await;

    assert_eq!(shutdown.message_id, 33);
    assert_eq!(shutdown.result, Some(json!({"shutdown": true})));
    assert_closed(&mut socket).await;
    server.wait_for_exit().await;
}

#[tokio::test]
async fn daemon_prepare_shutdown_returns_response_and_keeps_server_running() {
    let mut server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;

    write_frame(
        &mut socket,
        request_frame(32, 32, Method::DAEMON_PREPARE_SHUTDOWN, json!({})),
    )
    .await
    .expect("prepare shutdown request");
    let response = read_response(&mut socket).await;
    write_frame(&mut socket, request_frame(33, 33, "echo", json!("alive")))
        .await
        .expect("request after prepare");
    let after_prepare = read_response(&mut socket).await;

    assert_eq!(response.result, Some(json!({"prepared": true})));
    assert_eq!(after_prepare.result, Some(json!("alive")));
    assert!(!server.is_finished());
    server.request_shutdown();
    assert_closed(&mut socket).await;
    server.wait_for_exit().await;
}

#[tokio::test]
async fn failed_daemon_shutdown_response_does_not_stop_server() {
    let mut server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;

    write_frame(
        &mut socket,
        request_frame(34, 34, Method::DAEMON_SHUTDOWN, json!({"fail": true})),
    )
    .await
    .expect("failing shutdown request");
    let response = read_response(&mut socket).await;
    write_frame(&mut socket, request_frame(35, 35, "echo", json!("alive")))
        .await
        .expect("request after failed shutdown");
    let after_failure = read_response(&mut socket).await;

    assert_eq!(
        response.error.expect("shutdown error").code,
        StableErrorCode::Internal.as_str()
    );
    assert_eq!(after_failure.result, Some(json!("alive")));
    assert!(!server.is_finished());
    server.request_shutdown();
    assert_closed(&mut socket).await;
    server.wait_for_exit().await;
}

#[tokio::test]
async fn external_shutdown_waits_for_accepted_slow_response_before_exit() {
    let handler = GatedHandler::new();
    let mut server = TestServer::start(handler.clone()).await;
    let (mut socket, _) = server.connect_and_handshake().await;

    write_frame(&mut socket, request_frame(36, 36, "gated", json!("done")))
        .await
        .expect("gated request");
    handler.started.notified().await;
    server.request_shutdown();

    assert!(!server.is_finished());
    handler.release.notify_one();
    let response = read_response(&mut socket).await;
    assert_eq!(response.result, Some(json!("done")));
    assert_closed(&mut socket).await;
    server.wait_for_exit().await;
}

#[tokio::test]
async fn connection_started_after_shutdown_is_not_accepted_while_existing_connection_drains() {
    let handler = GatedHandler::new();
    let mut server = TestServer::start(handler.clone()).await;
    let (mut existing, _) = server.connect_and_handshake().await;
    write_frame(
        &mut existing,
        request_frame(37, 37, "gated", json!("existing")),
    )
    .await
    .expect("gated request");
    handler.started.notified().await;

    server.request_shutdown();
    if let Ok(mut socket) = timeout(IO_TIMEOUT, UnixStream::connect(&server.socket_path))
        .await
        .expect("new connection attempt")
    {
        assert_closed(&mut socket).await;
    }

    handler.release.notify_one();
    assert_eq!(
        read_response(&mut existing).await.result,
        Some(json!("existing"))
    );
    assert_closed(&mut existing).await;
    server.wait_for_exit().await;
}

#[tokio::test]
async fn request_payload_id_must_match_frame_message_id() {
    let server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;

    write_frame(&mut socket, request_frame(7, 8, "echo", json!({})))
        .await
        .expect("mismatched request");

    assert_closed(&mut socket).await;
}

#[tokio::test]
async fn unknown_method_returns_safe_method_not_found_error() {
    let server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;

    write_frame(
        &mut socket,
        request_frame(
            9,
            9,
            "future.method",
            json!({"secret": "example-sensitive-value"}),
        ),
    )
    .await
    .expect("unknown request");
    let response = read_response(&mut socket).await;
    let error = response.error.expect("service error");

    assert_eq!(error.code, StableErrorCode::MethodNotFound.as_str());
    assert_eq!(error.message, "method not found");
    assert!(!error.message.contains("future.method"));
    assert!(!error.message.contains("example-sensitive-value"));
}

#[tokio::test]
async fn backend_bad_request_message_is_replaced_with_safe_wire_error() {
    let server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;

    write_frame(
        &mut socket,
        request_frame(10, 10, "unsafe_error", json!({})),
    )
    .await
    .expect("bad request");
    let response = read_response(&mut socket).await;
    let error = response.error.expect("service error");

    assert_eq!(error.code, StableErrorCode::BadRequest.as_str());
    assert_eq!(error.message, "invalid request");
    assert!(!error.message.contains("raw-secret"));
}

#[tokio::test]
async fn backend_backpressure_maps_to_stable_retryable_error() {
    let server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;

    write_frame(
        &mut socket,
        request_frame(12, 12, "backpressure", json!({})),
    )
    .await
    .expect("backpressure request");
    let response = read_response(&mut socket).await;
    let error = response.error.expect("service error");

    assert_eq!(error.code, StableErrorCode::Backpressure.as_str());
    assert!(error.retryable);
}

#[tokio::test]
async fn ping_on_control_stream_returns_pong() {
    let server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;
    let ping = Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::Ping,
            flags: 0,
            stream_id: 0,
            message_id: 0,
            sequence: 41,
        },
        payload: Vec::new(),
    };

    write_frame(&mut socket, ping).await.expect("ping");
    let pong = read_frame(&mut socket).await;

    assert_eq!(pong.header.kind, FrameKind::Pong);
    assert_eq!(pong.header.stream_id, 0);
    assert_eq!(pong.header.message_id, 0);
    assert_eq!(pong.header.sequence, 41);
    assert!(pong.payload.is_empty());
}

#[tokio::test]
async fn advertised_fake_event_stream_opens_and_close_preserves_control_connection() {
    let server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;
    write_frame(
        &mut socket,
        stream_open_frame(
            1,
            StreamOpenRequest::Events(EventStreamOpen {
                after_seq: 0,
                event_filter: Vec::new(),
            }),
        ),
    )
    .await
    .expect("stream open");

    let opened = read_frame(&mut socket).await;
    assert_eq!(opened.header.kind, FrameKind::StreamOpened);
    assert_eq!(opened.header.stream_id, 1);

    write_frame(
        &mut socket,
        Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind: FrameKind::StreamClose,
                flags: 0,
                stream_id: 1,
                message_id: 0,
                sequence: 0,
            },
            payload: Vec::new(),
        },
    )
    .await
    .expect("stream close");
    write_frame(
        &mut socket,
        Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind: FrameKind::Ping,
                flags: 0,
                stream_id: 0,
                message_id: 0,
                sequence: 0,
            },
            payload: Vec::new(),
        },
    )
    .await
    .expect("ping after close");

    assert_eq!(read_frame(&mut socket).await.header.kind, FrameKind::Pong);
}

#[tokio::test]
async fn frame_length_smaller_than_header_is_rejected_before_body_read() {
    assert_raw_header_rejected(RawHeader {
        frame_len: (FRAME_HEADER_LEN - 1) as u32,
        kind: FrameKind::Request as u8,
        message_id: 1,
        ..RawHeader::default()
    })
    .await;
}

#[tokio::test]
async fn frame_length_over_total_limit_is_rejected_before_body_read() {
    assert_raw_header_rejected(RawHeader {
        frame_len: (MAX_FRAME_LEN + 1) as u32,
        kind: FrameKind::Request as u8,
        message_id: 1,
        ..RawHeader::default()
    })
    .await;
}

#[tokio::test]
async fn control_payload_over_limit_is_rejected_after_header_before_payload_read() {
    assert_raw_header_rejected(RawHeader {
        frame_len: (FRAME_HEADER_LEN + MAX_CONTROL_PAYLOAD + 1) as u32,
        kind: FrameKind::Request as u8,
        message_id: 1,
        ..RawHeader::default()
    })
    .await;
}

#[tokio::test]
async fn output_payload_over_limit_is_rejected_after_header_before_payload_read() {
    assert_raw_header_rejected(RawHeader {
        frame_len: (FRAME_HEADER_LEN + MAX_OUTPUT_PAYLOAD + 1) as u32,
        kind: FrameKind::Output as u8,
        stream_id: 1,
        ..RawHeader::default()
    })
    .await;
}

#[tokio::test]
async fn unknown_frame_kind_is_rejected_after_header_before_payload_read() {
    assert_raw_header_rejected(RawHeader {
        frame_len: (FRAME_HEADER_LEN + 1024) as u32,
        kind: 255,
        message_id: 1,
        ..RawHeader::default()
    })
    .await;
}

#[tokio::test]
async fn non_zero_flags_are_rejected_after_header_before_payload_read() {
    assert_raw_header_rejected(RawHeader {
        frame_len: (FRAME_HEADER_LEN + 1024) as u32,
        kind: FrameKind::Request as u8,
        flags: 1,
        message_id: 1,
        ..RawHeader::default()
    })
    .await;
}

#[tokio::test]
async fn request_on_non_control_stream_is_rejected_before_payload_read() {
    assert_raw_header_rejected(RawHeader {
        frame_len: (FRAME_HEADER_LEN + 1024) as u32,
        kind: FrameKind::Request as u8,
        stream_id: 1,
        message_id: 1,
        ..RawHeader::default()
    })
    .await;
}

#[tokio::test]
async fn malformed_json_payload_fails_closed() {
    let server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;
    write_raw_frame(
        &mut socket,
        RawHeader {
            kind: FrameKind::Request as u8,
            message_id: 1,
            ..RawHeader::default()
        },
        b"{",
    )
    .await;

    assert_closed(&mut socket).await;
}

#[tokio::test]
async fn sixty_fifth_active_connection_is_closed_before_protocol_payload() {
    let server = TestServer::start(FakeHandler).await;
    let mut active = Vec::new();
    for _ in 0..64 {
        active.push(server.connect_and_handshake().await.0);
    }
    let mut rejected = UnixStream::connect(&server.socket_path)
        .await
        .expect("65th connect");

    assert_closed(&mut rejected).await;
    assert_eq!(active.len(), 64);
}

#[tokio::test]
async fn connection_limits_handler_in_flight_requests_to_1024() {
    let server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;
    for message_id in 1..=1025 {
        write_frame(
            &mut socket,
            request_frame(message_id, message_id, Method::EVENTS_WAIT, json!({})),
        )
        .await
        .expect("hold request");
    }

    let response = read_response(&mut socket).await;
    let error = response.error.expect("backpressure error");

    assert_eq!(response.message_id, 1025);
    assert_eq!(error.code, StableErrorCode::Backpressure.as_str());
}

#[tokio::test]
async fn duplicate_active_message_id_closes_connection() {
    let server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;
    write_frame(
        &mut socket,
        request_frame(77, 77, Method::EVENTS_WAIT, json!({})),
    )
    .await
    .expect("first active request");
    write_frame(
        &mut socket,
        request_frame(77, 77, Method::EVENTS_WAIT, json!({})),
    )
    .await
    .expect("duplicate active request");

    assert_closed(&mut socket).await;
}

#[tokio::test]
async fn handler_panic_closes_connection() {
    let server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;

    write_frame(&mut socket, request_frame(88, 88, "panic", json!({})))
        .await
        .expect("panic request");

    assert_closed(&mut socket).await;
}

#[tokio::test]
async fn many_completed_unique_message_ids_are_reaped_and_connection_stays_healthy() {
    let server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;

    for message_id in 1..=2048_u64 {
        write_frame(
            &mut socket,
            request_frame(
                message_id,
                message_id,
                "echo",
                json!({"messageId": message_id}),
            ),
        )
        .await
        .expect("sequential request");
        let response = read_response(&mut socket).await;
        assert_eq!(response.message_id, message_id);
        assert_eq!(response.result, Some(json!({"messageId": message_id})));
    }
    write_frame(
        &mut socket,
        request_frame(2049, 2049, "echo", json!("healthy")),
    )
    .await
    .expect("health request");
    let response = read_response(&mut socket).await;

    assert_eq!(response.result, Some(json!("healthy")));
}

#[derive(Clone, Copy)]
struct FakeHandler;

impl ControlHandler for FakeHandler {
    fn handle<'a>(&'a self, method: &'a str, params: Value) -> ControlFuture<'a> {
        Box::pin(async move {
            match method {
                "echo" => Ok(params),
                "delayed" => {
                    let delay = params
                        .get("delayMs")
                        .and_then(Value::as_u64)
                        .ok_or_else(|| ServiceError::BadRequest("delay missing".to_string()))?;
                    let value = params.get("value").cloned().unwrap_or(Value::Null);
                    sleep(Duration::from_millis(delay)).await;
                    Ok(value)
                }
                "unsafe_error" => Err(ServiceError::BadRequest(
                    "raw-secret must never cross the socket".to_string(),
                )),
                "backpressure" => Err(ServiceError::Backpressure),
                Method::EVENTS_WAIT => pending::<ServiceResult<Value>>().await,
                "panic" => panic!("intentional handler panic"),
                Method::DAEMON_PREPARE_SHUTDOWN => Ok(json!({"prepared": true})),
                Method::DAEMON_SHUTDOWN if params.get("fail") == Some(&Value::Bool(true)) => {
                    Err(ServiceError::Internal)
                }
                Method::DAEMON_SHUTDOWN => Ok(json!({"shutdown": true})),
                _ => Err(ServiceError::MethodNotFound(method.to_string())),
            }
        })
    }
}

#[derive(Clone)]
struct GatedHandler {
    started: Arc<Notify>,
    release: Arc<Notify>,
}

impl GatedHandler {
    fn new() -> Self {
        Self {
            started: Arc::new(Notify::new()),
            release: Arc::new(Notify::new()),
        }
    }
}

impl ControlHandler for GatedHandler {
    fn handle<'a>(&'a self, method: &'a str, params: Value) -> ControlFuture<'a> {
        Box::pin(async move {
            if method != "gated" {
                return Err(ServiceError::MethodNotFound(method.to_string()));
            }
            self.started.notify_one();
            self.release.notified().await;
            Ok(params)
        })
    }
}

impl StreamHandler for GatedHandler {
    fn capabilities(&self) -> Vec<StreamKind> {
        FakeHandler.capabilities()
    }

    fn event_bounds(&self) -> EventBounds {
        FakeHandler.event_bounds()
    }

    fn open<'a>(
        &'a self,
        stream_id: u32,
        request: StreamOpenRequest,
        writer: homie_runtime::writer::WriterHandle,
    ) -> StreamFuture<'a> {
        FakeHandler.open(stream_id, request, writer)
    }
}

impl StreamHandler for FakeHandler {
    fn capabilities(&self) -> Vec<StreamKind> {
        vec![StreamKind::EventsV1, StreamKind::TerminalV1]
    }

    fn event_bounds(&self) -> EventBounds {
        EventBounds {
            oldest_seq: 41,
            latest_seq: 99,
        }
    }

    fn open<'a>(
        &'a self,
        stream_id: u32,
        request: StreamOpenRequest,
        writer: homie_runtime::writer::WriterHandle,
    ) -> StreamFuture<'a> {
        Box::pin(async move {
            writer
                .try_send_high(Frame {
                    header: FrameHeader {
                        version: WIRE_MAJOR,
                        kind: FrameKind::StreamOpened,
                        flags: 0,
                        stream_id,
                        message_id: 0,
                        sequence: 0,
                    },
                    payload: b"{}".to_vec(),
                })
                .map_err(StreamError::Writer)?;
            Ok(Box::new(FakeActiveStream {
                terminal: matches!(request, StreamOpenRequest::Terminal(_)),
            }) as Box<dyn ActiveStream>)
        })
    }
}

struct FakeActiveStream {
    terminal: bool,
}

impl ActiveStream for FakeActiveStream {
    fn is_finished(&self) -> bool {
        false
    }

    fn handle<'a>(&'a mut self, frame: Frame) -> ActiveStreamFuture<'a> {
        Box::pin(async move {
            if self.terminal && matches!(frame.header.kind, FrameKind::Input | FrameKind::Resize) {
                Ok(())
            } else {
                Err(StreamError::Protocol)
            }
        })
    }

    fn close(self: Box<Self>) -> ActiveStreamFuture<'static> {
        Box::pin(async { Ok(()) })
    }
}

struct TestServer {
    _temp: TempDir,
    socket_path: std::path::PathBuf,
    identity: ServerIdentity,
    shutdown: ShutdownHandle,
    task: Option<JoinHandle<Result<(), homie_runtime::server::ServerError>>>,
}

impl TestServer {
    async fn start(handler: impl ControlHandler + StreamHandler) -> Self {
        Self::start_with_config(handler, ServerConfig::current_user()).await
    }

    async fn start_with_config(
        handler: impl ControlHandler + StreamHandler,
        config: ServerConfig,
    ) -> Self {
        let temp = tempfile::tempdir().expect("tempdir");
        let socket_path = temp.path().join("runtime.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind UDS");
        let identity = ServerIdentity {
            daemon_build: "test-build".to_string(),
            daemon_version: "1.2.3".to_string(),
            daemon_pid: std::process::id(),
            daemon_instance_id: "instance-test".to_string(),
            executable_hash: "sha256:test".to_string(),
        };
        let handler = Arc::new(handler);
        let runtime_server = Arc::new(RuntimeServer::new(
            config,
            identity.clone(),
            handler.clone(),
            handler,
        ));
        let shutdown = runtime_server.shutdown_handle();
        let task = tokio::spawn(runtime_server.serve_listener(listener));
        Self {
            _temp: temp,
            socket_path,
            identity,
            shutdown,
            task: Some(task),
        }
    }

    async fn connect_and_handshake(&self) -> (UnixStream, HelloResponse) {
        let mut socket = connect_prefaced(&self.socket_path).await;
        write_frame(&mut socket, hello_frame(valid_hello()))
            .await
            .expect("hello");
        let frame = read_frame(&mut socket).await;
        assert_eq!(frame.header.kind, FrameKind::HelloAck);
        let hello = serde_json::from_slice(&frame.payload).expect("hello ack");
        (socket, hello)
    }

    fn request_shutdown(&self) {
        self.shutdown.request_shutdown();
    }

    fn is_finished(&self) -> bool {
        self.task.as_ref().is_some_and(JoinHandle::is_finished)
    }

    async fn wait_for_exit(&mut self) {
        let task = self.task.take().expect("server task");
        timeout(IO_TIMEOUT, task)
            .await
            .expect("server exit timeout")
            .expect("server task")
            .expect("server result");
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}

struct Response {
    message_id: u64,
    request_id: u64,
    result: Option<Value>,
    error: Option<homie_proto::ErrorEnvelope>,
}

async fn connect_prefaced(socket_path: &std::path::Path) -> UnixStream {
    let mut socket = UnixStream::connect(socket_path).await.expect("connect");
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
    socket
}

fn valid_hello() -> HelloRequest {
    HelloRequest {
        wire_major: WIRE_MAJOR,
        wire_minor: WIRE_MINOR,
        client_name: "homie-runtime-test".to_string(),
        client_version: "1.0.0".to_string(),
        client_role: ClientRole::Cli,
        process_id: std::process::id(),
    }
}

fn hello_frame(hello: HelloRequest) -> Frame {
    Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::Hello,
            flags: 0,
            stream_id: 0,
            message_id: 0,
            sequence: 0,
        },
        payload: serde_json::to_vec(&hello).expect("serialize hello"),
    }
}

fn request_frame(message_id: u64, request_id: u64, method: &str, params: Value) -> Frame {
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
            RequestId::from(request_id),
            method,
            params,
        ))
        .expect("serialize request"),
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
        payload: serde_json::to_vec(&request).expect("serialize stream open"),
    }
}

async fn write_frame(socket: &mut UnixStream, frame: Frame) -> std::io::Result<()> {
    let encoded = frame
        .encode(EndpointRole::Client)
        .expect("encode client frame");
    socket.write_all(&encoded).await
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
    .expect("frame timeout")
}

async fn read_response(socket: &mut UnixStream) -> Response {
    let frame = read_frame(socket).await;
    assert_eq!(frame.header.kind, FrameKind::Response);
    let message: ControlMessage = serde_json::from_slice(&frame.payload).expect("control response");
    let ControlMessage::Response {
        request_id,
        result,
        error,
        ..
    } = message
    else {
        panic!("expected response");
    };
    Response {
        message_id: frame.header.message_id,
        request_id: request_id.as_u64(),
        result,
        error,
    }
}

async fn assert_raw_header_rejected(header: RawHeader) {
    let server = TestServer::start(FakeHandler).await;
    let (mut socket, _) = server.connect_and_handshake().await;
    write_raw_header(&mut socket, header).await;

    assert_closed(&mut socket).await;
}

#[derive(Clone, Copy)]
struct RawHeader {
    frame_len: u32,
    version: u16,
    kind: u8,
    flags: u8,
    stream_id: u32,
    message_id: u64,
    sequence: u64,
}

impl Default for RawHeader {
    fn default() -> Self {
        Self {
            frame_len: FRAME_HEADER_LEN as u32,
            version: WIRE_MAJOR,
            kind: FrameKind::Ping as u8,
            flags: 0,
            stream_id: 0,
            message_id: 0,
            sequence: 0,
        }
    }
}

async fn write_raw_header(socket: &mut UnixStream, header: RawHeader) {
    let mut encoded = Vec::with_capacity(4 + FRAME_HEADER_LEN);
    encoded.extend_from_slice(&header.frame_len.to_be_bytes());
    encoded.extend_from_slice(&header.version.to_be_bytes());
    encoded.push(header.kind);
    encoded.push(header.flags);
    encoded.extend_from_slice(&header.stream_id.to_be_bytes());
    encoded.extend_from_slice(&header.message_id.to_be_bytes());
    encoded.extend_from_slice(&header.sequence.to_be_bytes());
    socket.write_all(&encoded).await.expect("raw header");
}

async fn write_raw_frame(socket: &mut UnixStream, mut header: RawHeader, payload: &[u8]) {
    header.frame_len = (FRAME_HEADER_LEN + payload.len()) as u32;
    write_raw_header(socket, header).await;
    socket.write_all(payload).await.expect("raw payload");
}

async fn assert_closed(socket: &mut UnixStream) {
    let mut byte = [0_u8; 1];
    let result = timeout(IO_TIMEOUT, socket.read(&mut byte))
        .await
        .expect("connection was not closed before payload");
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

#[test]
fn expected_stream_registry_is_exact() {
    assert_eq!(
        stream_capabilities(),
        [StreamKind::EventsV1, StreamKind::TerminalV1]
    );
    assert_eq!(Method::HELLO, "hello");
}
