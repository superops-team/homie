mod support;

use std::time::Duration;

use homie_client::{
    ClientError, ClientOptions, EventStreamItem, HomieClient, StreamState, TerminalItem,
};
use homie_proto::model::{RuntimeEvent, StateSnapshot};
use homie_proto::paths::RuntimeEndpoint;
use homie_proto::stream::{
    EventStreamOpen, StreamKind, StreamOpenRequest, StreamReset, StreamResetReason,
    TerminalStreamOpen,
};
use homie_proto::transport::{ClientRole, Frame, FrameHeader, FrameKind, WIRE_MAJOR};
use homie_proto::{ErrorEnvelope, Method};
use serde_json::json;
use tokio::sync::oneshot;

use support::{MockPeer, MockSocket};

#[tokio::test]
async fn event_gap_requests_snapshot_and_reopens_from_authoritative_cursor() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(
                &[Method::STATE_SNAPSHOT],
                &[StreamKind::EventsV1],
                "daemon-a",
            )
            .await;
        let open = peer.read_frame().await;
        let stream_id = open.header.stream_id;
        assert_event_open(&open, 7);
        send_stream_opened(&mut peer, stream_id).await;
        send_json_frame(
            &mut peer,
            FrameKind::StreamReset,
            stream_id,
            0,
            &StreamReset {
                reason: StreamResetReason::EventGap,
                last_confirmed_offset: None,
                latest_seq: Some(100),
            },
        )
        .await;

        let snapshot_request = peer.read_request().await;
        assert_eq!(snapshot_request.method, Method::STATE_SNAPSHOT);
        peer.respond_ok(
            snapshot_request.message_id,
            serde_json::to_value(StateSnapshot {
                sessions: Vec::new(),
                event_cursor: 100,
            })
            .expect("snapshot"),
        )
        .await;
        let reopened = peer.read_frame().await;
        assert_eq!(reopened.header.stream_id, stream_id);
        assert_event_open(&reopened, 100);
        send_stream_opened(&mut peer, stream_id).await;
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");

    let mut events = client
        .subscribe_events(EventStreamOpen {
            after_seq: 7,
            event_filter: Vec::new(),
        })
        .await
        .expect("open events");
    let item = events
        .recv()
        .await
        .expect("event recv")
        .expect("snapshot item");

    assert_eq!(
        item,
        EventStreamItem::Snapshot(StateSnapshot {
            sessions: Vec::new(),
            event_cursor: 100,
        })
    );
    client.close().await.expect("close");
    server.await.expect("server");
}

#[tokio::test]
async fn event_recovery_retries_a_retryable_snapshot_failure() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(
                &[Method::STATE_SNAPSHOT],
                &[StreamKind::EventsV1],
                "daemon-a",
            )
            .await;
        let open = peer.read_frame().await;
        let stream_id = open.header.stream_id;
        send_stream_opened(&mut peer, stream_id).await;
        send_json_frame(
            &mut peer,
            FrameKind::StreamReset,
            stream_id,
            0,
            &StreamReset {
                reason: StreamResetReason::EventGap,
                last_confirmed_offset: None,
                latest_seq: Some(100),
            },
        )
        .await;

        let first = peer.read_request().await;
        assert_eq!(first.method, Method::STATE_SNAPSHOT);
        peer.respond_error(
            first.message_id,
            ErrorEnvelope::new("backpressure", "retry snapshot", true),
        )
        .await;
        let second = tokio::time::timeout(Duration::from_secs(1), peer.read_request())
            .await
            .expect("snapshot retry");
        assert_eq!(second.method, Method::STATE_SNAPSHOT);
        peer.respond_ok(
            second.message_id,
            serde_json::to_value(StateSnapshot {
                sessions: Vec::new(),
                event_cursor: 100,
            })
            .expect("snapshot"),
        )
        .await;
        let reopened = peer.read_frame().await;
        assert_event_open(&reopened, 100);
        send_stream_opened(&mut peer, stream_id).await;
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let mut events = client
        .subscribe_events(EventStreamOpen {
            after_seq: 7,
            event_filter: Vec::new(),
        })
        .await
        .expect("open events");

    let item = tokio::time::timeout(Duration::from_secs(1), events.recv())
        .await
        .expect("recovered snapshot")
        .expect("event recv")
        .expect("snapshot item");
    assert!(matches!(
        item,
        EventStreamItem::Snapshot(StateSnapshot {
            event_cursor: 100,
            ..
        })
    ));

    client.close().await.expect("close");
    server.await.expect("server");
}

#[tokio::test]
async fn event_stream_delivers_replayable_events_in_sequence() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(&[], &[StreamKind::EventsV1], "daemon-a")
            .await;
        let open = peer.read_frame().await;
        let stream_id = open.header.stream_id;
        send_stream_opened(&mut peer, stream_id).await;
        send_json_frame(
            &mut peer,
            FrameKind::Event,
            stream_id,
            8,
            &RuntimeEvent {
                seq: 8,
                event: "session.updated".to_string(),
                session_id: Some("session-1".to_string()),
                status: Some("running".to_string()),
            },
        )
        .await;
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");

    let mut events = client
        .subscribe_events(EventStreamOpen {
            after_seq: 7,
            event_filter: Vec::new(),
        })
        .await
        .expect("open events");
    let item = events
        .recv()
        .await
        .expect("event recv")
        .expect("event item");

    assert!(matches!(
        item,
        EventStreamItem::Event(RuntimeEvent { seq: 8, .. })
    ));
    client.close().await.expect("close");
    server.await.expect("server");
}

#[tokio::test]
async fn event_stream_accepts_strictly_increasing_filtered_sequence_gaps() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(&[], &[StreamKind::EventsV1], "daemon-a")
            .await;
        let open = peer.read_frame().await;
        let stream_id = open.header.stream_id;
        assert_event_open(&open, 7);
        send_stream_opened(&mut peer, stream_id).await;
        send_json_frame(
            &mut peer,
            FrameKind::Event,
            stream_id,
            9,
            &RuntimeEvent {
                seq: 9,
                event: "session.updated".to_string(),
                session_id: Some("session-1".to_string()),
                status: Some("running".to_string()),
            },
        )
        .await;
        send_json_frame(
            &mut peer,
            FrameKind::Event,
            stream_id,
            12,
            &RuntimeEvent {
                seq: 12,
                event: "session.updated".to_string(),
                session_id: Some("session-1".to_string()),
                status: Some("idle".to_string()),
            },
        )
        .await;
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let mut events = client
        .subscribe_events(EventStreamOpen {
            after_seq: 7,
            event_filter: Vec::new(),
        })
        .await
        .expect("open events");

    let first = tokio::time::timeout(Duration::from_secs(1), events.recv())
        .await
        .expect("first event timeout")
        .expect("first event recv")
        .expect("first event");
    let second = tokio::time::timeout(Duration::from_secs(1), events.recv())
        .await
        .expect("second event timeout")
        .expect("second event recv")
        .expect("second event");

    assert!(matches!(
        (first, second),
        (
            EventStreamItem::Event(RuntimeEvent { seq: 9, .. }),
            EventStreamItem::Event(RuntimeEvent { seq: 12, .. })
        )
    ));
    client.close().await.expect("close");
    server.await.expect("server");
}

#[tokio::test]
async fn event_slow_consumer_reset_recovers_without_closing_removed_remote_stream() {
    assert_event_reset_recovers(StreamResetReason::SlowConsumer).await;
}

#[tokio::test]
async fn event_resync_required_reset_recovers_without_closing_removed_remote_stream() {
    assert_event_reset_recovers(StreamResetReason::ResyncRequired).await;
}

#[tokio::test]
async fn event_protocol_error_reset_recovers_without_closing_removed_remote_stream() {
    assert_event_reset_recovers(StreamResetReason::ProtocolError).await;
}

#[tokio::test]
async fn event_local_overflow_closes_remote_once_before_single_snapshot_recovery() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let (recovery_started_tx, recovery_started_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(
                &[Method::STATE_SNAPSHOT],
                &[StreamKind::EventsV1],
                "daemon-a",
            )
            .await;
        let open = peer.read_frame().await;
        let stream_id = open.header.stream_id;
        send_stream_opened(&mut peer, stream_id).await;
        for sequence in 1..=258 {
            send_json_frame(
                &mut peer,
                FrameKind::Event,
                stream_id,
                sequence,
                &RuntimeEvent {
                    seq: sequence,
                    event: "session.updated".to_string(),
                    session_id: Some("session-1".to_string()),
                    status: None,
                },
            )
            .await;
        }

        let close = peer.read_frame().await;
        assert_eq!(close.header.kind, FrameKind::StreamClose);
        assert_eq!(close.header.stream_id, stream_id);
        let snapshot_request = peer.read_request().await;
        assert_eq!(snapshot_request.method, Method::STATE_SNAPSHOT);
        assert!(
            tokio::time::timeout(Duration::from_millis(50), peer.read_frame())
                .await
                .is_err(),
            "overflow started duplicate recovery"
        );
        recovery_started_tx
            .send(())
            .expect("recovery started signal");
        peer.respond_ok(
            snapshot_request.message_id,
            serde_json::to_value(StateSnapshot {
                sessions: Vec::new(),
                event_cursor: 300,
            })
            .expect("snapshot"),
        )
        .await;
        let reopened = peer.read_frame().await;
        assert_event_open(&reopened, 300);
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let mut events = client
        .subscribe_events(EventStreamOpen {
            after_seq: 0,
            event_filter: Vec::new(),
        })
        .await
        .expect("open events");

    recovery_started_rx.await.expect("recovery started");
    let first = events
        .recv()
        .await
        .expect("event recv")
        .expect("event item");

    assert!(matches!(
        first,
        EventStreamItem::Event(RuntimeEvent { seq: 1, .. })
    ));
    server.await.expect("server");
    client.close().await.expect("close");
}

#[tokio::test]
async fn terminal_reconnect_uses_confirmed_offset_and_discards_diff_until_full_grid() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let (disconnect_tx, disconnect_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut first = socket
            .accept(&[], &[StreamKind::TerminalV1], "daemon-a")
            .await;
        let open = first.read_frame().await;
        let stream_id = open.header.stream_id;
        assert_terminal_open(&open, 0);
        send_stream_opened(&mut first, stream_id).await;
        send_output(&mut first, stream_id, 1, 0, b"abc").await;
        send_grid(&mut first, stream_id, 2, true).await;
        disconnect_rx.await.expect("disconnect first daemon");
        drop(first);

        let mut second = socket
            .accept(&[], &[StreamKind::TerminalV1], "daemon-b")
            .await;
        let reopened = second.read_frame().await;
        assert_eq!(reopened.header.stream_id, stream_id);
        assert_terminal_open(&reopened, 3);
        send_stream_opened(&mut second, stream_id).await;
        send_grid(&mut second, stream_id, 1, false).await;
        send_grid(&mut second, stream_id, 2, true).await;
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let mut terminal = client
        .open_terminal(TerminalStreamOpen {
            session_id: "session-1".to_string(),
            output_offset: 0,
            client_role: ClientRole::Cli,
            last_grid_sequence: None,
        })
        .await
        .expect("open terminal");

    let output = terminal.recv().await.expect("output recv").expect("output");
    let initial_grid = terminal.recv().await.expect("grid recv").expect("grid");
    assert!(matches!(
        output,
        TerminalItem::Output { offset: 0, ref bytes } if bytes == b"abc"
    ));
    assert!(matches!(
        initial_grid,
        TerminalItem::Grid(ref update) if update.is_full_snapshot
    ));
    assert_eq!(terminal.last_confirmed_offset(), 3);

    disconnect_tx.send(()).expect("disconnect");
    let reopened_grid = tokio::time::timeout(Duration::from_secs(3), terminal.recv())
        .await
        .expect("terminal reconnect timeout")
        .expect("terminal recv")
        .expect("reopened grid");

    assert!(matches!(
        reopened_grid,
        TerminalItem::Grid(ref update) if update.is_full_snapshot
    ));
    client.close().await.expect("close");
    server.await.expect("server");
}

#[tokio::test]
async fn terminal_forward_sequence_gap_reopens_before_delivering_output() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(&[], &[StreamKind::TerminalV1], "daemon-a")
            .await;
        let open = peer.read_frame().await;
        let stream_id = open.header.stream_id;
        assert_terminal_open(&open, 0);
        send_stream_opened(&mut peer, stream_id).await;
        send_output(&mut peer, stream_id, 2, 0, b"jumped").await;

        let close = peer.read_frame().await;
        assert_eq!(close.header.kind, FrameKind::StreamClose);
        let reopened = peer.read_frame().await;
        assert_terminal_open(&reopened, 0);
        send_stream_opened(&mut peer, stream_id).await;
        send_output(&mut peer, stream_id, 1, 0, b"recovered").await;
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let mut terminal = client
        .open_terminal(TerminalStreamOpen {
            session_id: "session-1".to_string(),
            output_offset: 0,
            client_role: ClientRole::Cli,
            last_grid_sequence: None,
        })
        .await
        .expect("open terminal");

    let item = terminal
        .recv()
        .await
        .expect("terminal recv")
        .expect("terminal output");

    assert!(matches!(
        item,
        TerminalItem::Output { ref bytes, .. } if bytes == b"recovered"
    ));
    client.close().await.expect("close");
    server.await.expect("server");
}

#[tokio::test]
async fn terminal_decoded_queue_overflow_exposes_resync_with_last_confirmed_offset() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let (closed_tx, closed_rx) = oneshot::channel();
    let (dropped_tx, dropped_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(&[], &[StreamKind::TerminalV1], "daemon-a")
            .await;
        let open = peer.read_frame().await;
        let stream_id = open.header.stream_id;
        send_stream_opened(&mut peer, stream_id).await;
        for offset in 0..257_u64 {
            send_output(&mut peer, stream_id, offset + 1, offset, b"x").await;
        }
        let close = peer.read_frame().await;
        assert_eq!(close.header.kind, FrameKind::StreamClose);
        assert_eq!(close.header.stream_id, stream_id);
        closed_tx.send(()).expect("overflow close signal");
        dropped_rx.await.expect("terminal dropped");
        assert!(
            tokio::time::timeout(Duration::from_millis(50), peer.read_frame())
                .await
                .is_err(),
            "terminal drop sent a second close"
        );
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let mut terminal = client
        .open_terminal(TerminalStreamOpen {
            session_id: "session-1".to_string(),
            output_offset: 0,
            client_role: ClientRole::Cli,
            last_grid_sequence: None,
        })
        .await
        .expect("open terminal");
    let mut state = terminal.state();

    wait_for_resync(&mut state).await;
    closed_rx.await.expect("overflow close");

    assert_eq!(
        *state.borrow(),
        StreamState::ResyncRequired {
            last_confirmed_offset: Some(256)
        }
    );
    for _ in 0..256 {
        terminal
            .recv()
            .await
            .expect("terminal receive")
            .expect("queued output");
    }
    assert_eq!(terminal.recv().await.expect("closed receiver"), None);
    drop(terminal);
    dropped_tx.send(()).expect("terminal dropped signal");
    server.await.expect("server");
    client.close().await.expect("close");
}

#[tokio::test]
async fn replay_begin_adopts_server_authoritative_offset_before_output() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(&[], &[StreamKind::TerminalV1], "daemon-a")
            .await;
        let open = peer.read_frame().await;
        let stream_id = open.header.stream_id;
        assert_terminal_open(&open, 100);
        send_stream_opened(&mut peer, stream_id).await;
        send_offset_frame(&mut peer, FrameKind::ReplayBegin, stream_id, 1, 5).await;
        send_output(&mut peer, stream_id, 2, 5, b"x").await;
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let mut terminal = client
        .open_terminal(TerminalStreamOpen {
            session_id: "session-1".to_string(),
            output_offset: 100,
            client_role: ClientRole::Cli,
            last_grid_sequence: None,
        })
        .await
        .expect("open terminal");

    let begin = tokio::time::timeout(Duration::from_secs(1), terminal.recv())
        .await
        .expect("begin timeout")
        .expect("begin recv")
        .expect("begin");
    let output = tokio::time::timeout(Duration::from_secs(1), terminal.recv())
        .await
        .expect("output timeout")
        .expect("output recv")
        .expect("output");

    assert_eq!(begin, TerminalItem::ReplayBegin(5));
    assert!(matches!(
        output,
        TerminalItem::Output { offset: 5, ref bytes } if bytes == b"x"
    ));
    assert_eq!(terminal.last_confirmed_offset(), 6);
    client.close().await.expect("close");
    server.await.expect("server");
}

#[tokio::test]
async fn terminal_send_input_preserves_raw_wire_bytes() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let expected = vec![0, 0xff, 0x80, b'\n'];
    let server_expected = expected.clone();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(&[], &[StreamKind::TerminalV1], "daemon-a")
            .await;
        let open = peer.read_frame().await;
        let stream_id = open.header.stream_id;
        send_stream_opened(&mut peer, stream_id).await;

        let input = peer.read_frame().await;

        assert_terminal_control_header(&input, FrameKind::Input, stream_id);
        assert_eq!(input.payload, server_expected);
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let terminal = open_terminal(&client).await;

    terminal.send_input(expected).expect("send raw input");

    server.await.expect("server");
    client.close().await.expect("close");
}

#[tokio::test]
async fn terminal_send_input_accepts_64_kib_and_rejects_larger_payload() {
    const MAX_INPUT_BYTES: usize = 64 * 1024;

    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(&[], &[StreamKind::TerminalV1], "daemon-a")
            .await;
        let open = peer.read_frame().await;
        let stream_id = open.header.stream_id;
        send_stream_opened(&mut peer, stream_id).await;

        let input = peer.read_frame().await;

        assert_terminal_control_header(&input, FrameKind::Input, stream_id);
        assert_eq!(input.payload, vec![0xa5; MAX_INPUT_BYTES]);
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let terminal = open_terminal(&client).await;

    let error = terminal
        .send_input(vec![0; MAX_INPUT_BYTES + 1])
        .expect_err("oversized input must be rejected");
    assert!(matches!(error, ClientError::BadRequest(_)));
    terminal
        .send_input(vec![0xa5; MAX_INPUT_BYTES])
        .expect("64 KiB input");

    server.await.expect("server");
    client.close().await.expect("close");
}

#[tokio::test]
async fn terminal_resize_sends_big_endian_dimensions() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(&[], &[StreamKind::TerminalV1], "daemon-a")
            .await;
        let open = peer.read_frame().await;
        let stream_id = open.header.stream_id;
        send_stream_opened(&mut peer, stream_id).await;

        let resize = peer.read_frame().await;

        assert_terminal_control_header(&resize, FrameKind::Resize, stream_id);
        assert_eq!(resize.payload, [0x01, 0x2c, 0x00, 0x50]);
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let terminal = open_terminal(&client).await;

    terminal.resize(300, 80).expect("resize terminal");

    server.await.expect("server");
    client.close().await.expect("close");
}

#[tokio::test]
async fn terminal_resize_rejects_zero_dimensions() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(&[], &[StreamKind::TerminalV1], "daemon-a")
            .await;
        let open = peer.read_frame().await;
        let stream_id = open.header.stream_id;
        send_stream_opened(&mut peer, stream_id).await;

        let resize = peer.read_frame().await;

        assert_terminal_control_header(&resize, FrameKind::Resize, stream_id);
        assert_eq!(resize.payload, [0x00, 0x50, 0x00, 0x18]);
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let terminal = open_terminal(&client).await;

    let zero_cols = terminal
        .resize(0, 24)
        .expect_err("zero columns must be rejected");
    assert!(matches!(zero_cols, ClientError::BadRequest(_)));
    let zero_rows = terminal
        .resize(80, 0)
        .expect_err("zero rows must be rejected");
    assert!(matches!(zero_rows, ClientError::BadRequest(_)));
    terminal.resize(80, 24).expect("valid resize");

    server.await.expect("server");
    client.close().await.expect("close");
}

#[tokio::test]
async fn terminal_resize_rejects_geometry_over_shared_cell_limit() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(&[], &[StreamKind::TerminalV1], "daemon-a")
            .await;
        let open = peer.read_frame().await;
        send_stream_opened(&mut peer, open.header.stream_id).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let terminal = open_terminal(&client).await;

    let error = terminal
        .resize(4_096, 257)
        .expect_err("terminal cell limit must be enforced");

    assert!(matches!(error, ClientError::BadRequest(_)));
    client.close().await.expect("close");
    server.await.expect("server");
}

#[tokio::test]
async fn terminal_control_requires_open_stream_state() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let (reopened_tx, reopened_rx) = oneshot::channel();
    let (resume_tx, resume_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(&[], &[StreamKind::TerminalV1], "daemon-a")
            .await;
        let open = peer.read_frame().await;
        let stream_id = open.header.stream_id;
        send_stream_opened(&mut peer, stream_id).await;
        send_json_frame(
            &mut peer,
            FrameKind::StreamReset,
            stream_id,
            0,
            &StreamReset {
                reason: StreamResetReason::ResyncRequired,
                last_confirmed_offset: None,
                latest_seq: None,
            },
        )
        .await;

        let reopened = peer.read_frame().await;
        assert_terminal_open(&reopened, 0);
        reopened_tx.send(()).expect("reopened observed");
        resume_rx.await.expect("resume server");

        send_stream_opened(&mut peer, stream_id).await;
        let input = peer.read_frame().await;
        assert_terminal_control_header(&input, FrameKind::Input, stream_id);
        assert_eq!(input.payload, b"open");
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let terminal = open_terminal(&client).await;
    let mut state = terminal.state();
    reopened_rx.await.expect("terminal reopened");
    assert_eq!(*state.borrow(), StreamState::Reconnecting);

    let reconnecting_result = terminal.send_input(b"blocked".as_slice());
    resume_tx.send(()).expect("resume");
    assert!(matches!(reconnecting_result, Err(ClientError::Unavailable)));

    wait_for_open(&mut state).await;
    terminal
        .send_input(b"open".as_slice())
        .expect("open stream input");

    server.await.expect("server");
    client.close().await.expect("close");
}

#[tokio::test]
async fn close_closes_all_streams_and_never_marks_them_reconnecting() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(
                &[],
                &[StreamKind::EventsV1, StreamKind::TerminalV1],
                "daemon-a",
            )
            .await;
        let event_open = peer.read_frame().await;
        send_stream_opened(&mut peer, event_open.header.stream_id).await;
        let terminal_open = peer.read_frame().await;
        send_stream_opened(&mut peer, terminal_open.header.stream_id).await;

        let first_close = peer.read_frame().await;
        let second_close = peer.read_frame().await;
        let mut closed_ids = [first_close.header.stream_id, second_close.header.stream_id];
        closed_ids.sort_unstable();

        assert_eq!(first_close.header.kind, FrameKind::StreamClose);
        assert_eq!(second_close.header.kind, FrameKind::StreamClose);
        assert_eq!(
            closed_ids,
            [event_open.header.stream_id, terminal_open.header.stream_id]
        );
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let event = client
        .subscribe_events(EventStreamOpen {
            after_seq: 0,
            event_filter: Vec::new(),
        })
        .await
        .expect("event stream");
    let terminal = open_terminal(&client).await;
    let event_state = event.state();
    let terminal_state = terminal.state();

    client.close().await.expect("close");

    assert_eq!(*event_state.borrow(), StreamState::Closed);
    assert_eq!(*terminal_state.borrow(), StreamState::Closed);
    server.await.expect("server");
    drop(event);
    drop(terminal);
}

#[tokio::test]
async fn terminal_control_rejects_closed_stream() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let (release_tx, release_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(&[], &[StreamKind::TerminalV1], "daemon-a")
            .await;
        let open = peer.read_frame().await;
        let stream_id = open.header.stream_id;
        send_stream_opened(&mut peer, stream_id).await;
        send_json_frame(&mut peer, FrameKind::StreamClose, stream_id, 1, &json!({})).await;
        release_rx.await.expect("release server");
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let terminal = open_terminal(&client).await;
    let mut state = terminal.state();
    wait_for_closed(&mut state).await;

    assert!(matches!(
        terminal.send_input(b"input".as_slice()),
        Err(ClientError::Unavailable)
    ));
    assert!(matches!(
        terminal.resize(80, 24),
        Err(ClientError::Unavailable)
    ));

    release_tx.send(()).expect("release");
    client.close().await.expect("close");
    server.await.expect("server");
}

#[tokio::test]
async fn terminal_control_rejects_unavailable_client() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(&[], &[StreamKind::TerminalV1], "daemon-a")
            .await;
        let open = peer.read_frame().await;
        send_stream_opened(&mut peer, open.header.stream_id).await;
        tokio::time::sleep(Duration::from_secs(5)).await;
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let terminal = open_terminal(&client).await;
    client.close().await.expect("close");

    assert!(matches!(
        terminal.send_input(Vec::new()),
        Err(ClientError::Unavailable)
    ));
    assert!(matches!(
        terminal.resize(80, 24),
        Err(ClientError::Unavailable)
    ));

    server.abort();
}

fn options(endpoint: std::path::PathBuf) -> ClientOptions {
    ClientOptions {
        endpoint: RuntimeEndpoint::new(endpoint).expect("absolute endpoint"),
        role: ClientRole::Cli,
        connect_timeout: Duration::from_secs(1),
        request_timeout: Duration::from_secs(1),
    }
}

async fn open_terminal(client: &HomieClient) -> homie_client::TerminalStream {
    client
        .open_terminal(TerminalStreamOpen {
            session_id: "session-1".to_string(),
            output_offset: 0,
            client_role: ClientRole::Cli,
            last_grid_sequence: None,
        })
        .await
        .expect("open terminal")
}

async fn send_stream_opened(peer: &mut MockPeer, stream_id: u32) {
    send_json_frame(peer, FrameKind::StreamOpened, stream_id, 0, &json!({})).await;
}

async fn send_json_frame(
    peer: &mut MockPeer,
    kind: FrameKind,
    stream_id: u32,
    sequence: u64,
    payload: &impl serde::Serialize,
) {
    peer.write_frame(Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind,
            flags: 0,
            stream_id,
            message_id: 0,
            sequence,
        },
        payload: serde_json::to_vec(payload).expect("json frame"),
    })
    .await;
}

async fn send_output(
    peer: &mut MockPeer,
    stream_id: u32,
    sequence: u64,
    offset: u64,
    bytes: &[u8],
) {
    let mut payload = offset.to_be_bytes().to_vec();
    payload.extend_from_slice(bytes);
    peer.write_frame(Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::Output,
            flags: 0,
            stream_id,
            message_id: 0,
            sequence,
        },
        payload,
    })
    .await;
}

async fn send_offset_frame(
    peer: &mut MockPeer,
    kind: FrameKind,
    stream_id: u32,
    sequence: u64,
    offset: u64,
) {
    peer.write_frame(Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind,
            flags: 0,
            stream_id,
            message_id: 0,
            sequence,
        },
        payload: offset.to_be_bytes().to_vec(),
    })
    .await;
}

async fn send_grid(peer: &mut MockPeer, stream_id: u32, sequence: u64, full: bool) {
    let mut payload = Vec::new();
    payload.extend_from_slice(&80_u16.to_be_bytes());
    payload.extend_from_slice(&24_u16.to_be_bytes());
    payload.extend_from_slice(&0_u16.to_be_bytes());
    payload.extend_from_slice(&0_u16.to_be_bytes());
    payload.push(u8::from(full) << 1);
    payload.extend_from_slice(&0_u16.to_be_bytes());
    peer.write_frame(Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::Grid,
            flags: 0,
            stream_id,
            message_id: 0,
            sequence,
        },
        payload,
    })
    .await;
}

fn assert_event_open(frame: &Frame, expected_after_seq: u64) {
    assert_eq!(frame.header.kind, FrameKind::StreamOpen);
    let request: StreamOpenRequest =
        serde_json::from_slice(&frame.payload).expect("event stream open");
    assert!(matches!(
        request,
        StreamOpenRequest::Events(EventStreamOpen { after_seq, .. })
            if after_seq == expected_after_seq
    ));
}

fn assert_terminal_open(frame: &Frame, expected_offset: u64) {
    assert_eq!(frame.header.kind, FrameKind::StreamOpen);
    let request: StreamOpenRequest =
        serde_json::from_slice(&frame.payload).expect("terminal stream open");
    assert!(matches!(
        request,
        StreamOpenRequest::Terminal(TerminalStreamOpen { output_offset, .. })
            if output_offset == expected_offset
    ));
}

fn assert_terminal_control_header(frame: &Frame, kind: FrameKind, stream_id: u32) {
    assert_eq!(frame.header.kind, kind);
    assert_eq!(frame.header.stream_id, stream_id);
    assert_eq!(frame.header.message_id, 0);
    assert_eq!(frame.header.sequence, 0);
}

async fn assert_event_reset_recovers(reason: StreamResetReason) {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(
                &[Method::STATE_SNAPSHOT],
                &[StreamKind::EventsV1],
                "daemon-a",
            )
            .await;
        let open = peer.read_frame().await;
        let stream_id = open.header.stream_id;
        send_stream_opened(&mut peer, stream_id).await;
        send_json_frame(
            &mut peer,
            FrameKind::StreamReset,
            stream_id,
            0,
            &StreamReset {
                reason,
                last_confirmed_offset: None,
                latest_seq: Some(100),
            },
        )
        .await;

        let snapshot_request = peer.read_request().await;
        assert_eq!(snapshot_request.method, Method::STATE_SNAPSHOT);
        peer.respond_ok(
            snapshot_request.message_id,
            serde_json::to_value(StateSnapshot {
                sessions: Vec::new(),
                event_cursor: 100,
            })
            .expect("snapshot"),
        )
        .await;
        let reopened = peer.read_frame().await;
        assert_event_open(&reopened, 100);
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    let mut events = client
        .subscribe_events(EventStreamOpen {
            after_seq: 7,
            event_filter: Vec::new(),
        })
        .await
        .expect("open events");

    let item = tokio::time::timeout(Duration::from_secs(1), events.recv())
        .await
        .expect("snapshot timeout")
        .expect("snapshot recv")
        .expect("snapshot");

    assert!(matches!(
        item,
        EventStreamItem::Snapshot(StateSnapshot {
            event_cursor: 100,
            ..
        })
    ));
    server.await.expect("server");
    client.close().await.expect("close");
}

async fn wait_for_resync(state: &mut tokio::sync::watch::Receiver<StreamState>) {
    loop {
        if matches!(*state.borrow(), StreamState::ResyncRequired { .. }) {
            return;
        }
        state.changed().await.expect("stream state remains open");
    }
}

async fn wait_for_open(state: &mut tokio::sync::watch::Receiver<StreamState>) {
    loop {
        if *state.borrow() == StreamState::Open {
            return;
        }
        state
            .changed()
            .await
            .expect("stream state remains available");
    }
}

async fn wait_for_closed(state: &mut tokio::sync::watch::Receiver<StreamState>) {
    loop {
        if *state.borrow() == StreamState::Closed {
            return;
        }
        state
            .changed()
            .await
            .expect("stream state remains available");
    }
}
