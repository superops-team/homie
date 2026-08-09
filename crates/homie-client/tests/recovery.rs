mod support;

use std::time::Duration;

use homie_client::{ClientOptions, ConnectionState, HomieClient};
use homie_proto::paths::RuntimeEndpoint;
use homie_proto::transport::{ClientRole, Frame, FrameHeader, FrameKind, WIRE_MAJOR};
use tokio::sync::oneshot;

use support::MockSocket;

#[tokio::test]
async fn idle_connection_sends_ping_at_25_seconds_and_degrades_after_10_more_seconds() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let (ping_tx, mut ping_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut peer = socket.accept(&[], &[], "daemon-a").await;
        let ping = peer.read_frame().await;
        assert_eq!(ping.header.kind, FrameKind::Ping);
        ping_tx.send(()).expect("ping observed");
        tokio::time::sleep(Duration::from_secs(20)).await;
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    tokio::time::pause();
    let mut state = client.connection_state();

    tokio::time::advance(Duration::from_secs(24)).await;
    tokio::task::yield_now().await;
    assert!(matches!(
        ping_rx.try_recv(),
        Err(tokio::sync::oneshot::error::TryRecvError::Empty)
    ));

    tokio::time::advance(Duration::from_secs(1)).await;
    ping_rx.await.expect("ping at 25 seconds");
    tokio::time::advance(Duration::from_secs(10)).await;
    wait_until_not_connected(&mut state).await;

    assert!(!matches!(
        *state.borrow(),
        ConnectionState::Connected { .. }
    ));
    client.close().await.expect("close");
    server.abort();
}

#[tokio::test]
async fn matching_pong_keeps_idle_connection_connected() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let (pong_tx, pong_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut peer = socket.accept(&[], &[], "daemon-a").await;
        let ping = peer.read_frame().await;
        assert_eq!(ping.header.kind, FrameKind::Ping);
        assert_ne!(ping.header.sequence, 0);
        peer.write_frame(Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind: FrameKind::Pong,
                flags: 0,
                stream_id: 0,
                message_id: 0,
                sequence: ping.header.sequence,
            },
            payload: Vec::new(),
        })
        .await;
        pong_tx.send(()).expect("pong sent");
        tokio::time::sleep(Duration::from_secs(20)).await;
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    tokio::time::pause();
    let state = client.connection_state();

    tokio::time::advance(Duration::from_secs(25)).await;
    pong_rx.await.expect("matching pong");
    tokio::task::yield_now().await;

    assert!(matches!(*state.borrow(), ConnectionState::Connected { .. }));
    client.close().await.expect("close");
    server.abort();
}

#[tokio::test]
async fn reconnect_backoff_grows_from_500_milliseconds_to_8_seconds() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let _peer = socket.accept(&[], &[], "daemon-a").await;
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");
    tokio::time::pause();
    server.await.expect("server");
    let mut state = client.connection_state();

    for delay in [
        Duration::from_millis(500),
        Duration::from_secs(1),
        Duration::from_secs(2),
        Duration::from_secs(4),
        Duration::from_secs(8),
    ] {
        wait_for_reconnect_delay(&mut state, delay).await;
        tokio::time::advance(delay).await;
        tokio::task::yield_now().await;
    }
    wait_for_reconnect_delay(&mut state, Duration::from_secs(8)).await;

    client.close().await.expect("close");
}

fn options(endpoint: std::path::PathBuf) -> ClientOptions {
    ClientOptions {
        endpoint: RuntimeEndpoint::new(endpoint).expect("absolute endpoint"),
        role: ClientRole::Cli,
        connect_timeout: Duration::from_secs(1),
        request_timeout: Duration::from_secs(10),
    }
}

async fn wait_until_not_connected(state: &mut tokio::sync::watch::Receiver<ConnectionState>) {
    for _ in 0..32 {
        if !matches!(*state.borrow(), ConnectionState::Connected { .. }) {
            return;
        }
        tokio::task::yield_now().await;
    }
    panic!("connection remained connected after heartbeat timeout");
}

async fn wait_for_reconnect_delay(
    state: &mut tokio::sync::watch::Receiver<ConnectionState>,
    expected: Duration,
) {
    loop {
        if matches!(
            *state.borrow(),
            ConnectionState::Reconnecting { delay, .. } if delay == expected
        ) {
            return;
        }
        state
            .changed()
            .await
            .expect("connection state remains open");
    }
}
