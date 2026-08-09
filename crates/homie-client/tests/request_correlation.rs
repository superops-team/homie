mod support;

use std::sync::Arc;
use std::time::Duration;

use homie_client::{ClientOptions, ConnectionState, HomieClient};
use homie_proto::paths::RuntimeEndpoint;
use homie_proto::transport::ClientRole;
use serde_json::{Value, json};
use tokio::sync::{mpsc, oneshot};

use support::MockSocket;

#[tokio::test]
async fn connect_completes_hello_and_publishes_connected_state() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let _peer = socket.accept(&[], &[], "daemon-a").await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    });

    let client = HomieClient::connect(options(endpoint, Duration::from_secs(1)))
        .await
        .expect("connect");
    let state = client.connection_state().borrow().clone();

    assert!(matches!(
        state,
        ConnectionState::Connected {
            daemon_instance_id,
            ..
        } if daemon_instance_id == "daemon-a"
    ));
    client.close().await.expect("close");
    server.abort();
}

#[tokio::test]
async fn concurrent_requests_resolve_by_message_id_when_responses_are_out_of_order() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket.accept(&["first", "second"], &[], "daemon-a").await;
        let first = peer.read_request().await;
        let second = peer.read_request().await;
        peer.respond_ok(
            second.message_id,
            json!({"method": second.method, "params": second.params}),
        )
        .await;
        peer.respond_ok(
            first.message_id,
            json!({"method": first.method, "params": first.params}),
        )
        .await;
    });
    let client = HomieClient::connect(options(endpoint, Duration::from_secs(1)))
        .await
        .expect("connect");

    let first = client.request::<_, Value>("first", json!({"value": 1}));
    let second = client.request::<_, Value>("second", json!({"value": 2}));
    let (first, second) = tokio::join!(first, second);

    assert_eq!(first.expect("first response")["method"], "first");
    assert_eq!(second.expect("second response")["method"], "second");
    client.close().await.expect("close");
    server.await.expect("server");
}

#[tokio::test]
async fn request_timeout_removes_only_its_waiter_and_late_response_is_ignored() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket.accept(&["slow", "fast"], &[], "daemon-a").await;
        let slow = peer.read_request().await;
        tokio::time::sleep(Duration::from_millis(150)).await;
        peer.respond_ok(slow.message_id, json!({"late": true}))
            .await;
        let fast = peer.read_request().await;
        peer.respond_ok(fast.message_id, json!({"late": false}))
            .await;
    });
    let client = HomieClient::connect(options(endpoint, Duration::from_millis(100)))
        .await
        .expect("connect");

    let error = client
        .request::<_, Value>("slow", json!({}))
        .await
        .expect_err("slow request should timeout");
    let fast = client
        .request::<_, Value>("fast", json!({}))
        .await
        .expect("healthy connection accepts next request");

    assert_eq!(error.code(), "timeout");
    assert_eq!(fast, json!({"late": false}));
    client.close().await.expect("close");
    server.await.expect("server");
}

#[tokio::test]
async fn dropped_request_future_removes_waiter_and_late_response_keeps_connection_healthy() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let (request_seen_tx, request_seen_rx) = oneshot::channel();
    let (send_late_tx, send_late_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut peer = socket
            .accept(&["cancelled", "healthy"], &[], "daemon-a")
            .await;
        let cancelled = peer.read_request().await;
        request_seen_tx.send(()).expect("request seen");
        send_late_rx.await.expect("send late response");
        peer.respond_ok(cancelled.message_id, json!({"late": true}))
            .await;
        let healthy = peer.read_request().await;
        peer.respond_ok(healthy.message_id, json!({"ok": true}))
            .await;
    });
    let client = Arc::new(
        HomieClient::connect(options(endpoint, Duration::from_secs(1)))
            .await
            .expect("connect"),
    );
    let cancelled_client = client.clone();
    let cancelled = tokio::spawn(async move {
        cancelled_client
            .request::<_, Value>("cancelled", json!({}))
            .await
    });
    request_seen_rx.await.expect("request reached server");

    cancelled.abort();
    send_late_tx.send(()).expect("release late response");
    let response = client
        .request::<_, Value>("healthy", json!({}))
        .await
        .expect("healthy request");

    assert_eq!(response, json!({"ok": true}));
    client.close().await.expect("close");
    server.await.expect("server");
}

#[tokio::test]
async fn disconnect_fails_all_pending_requests_once_without_replay() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket.accept(&["one", "two"], &[], "daemon-a").await;
        let _one = peer.read_request().await;
        let _two = peer.read_request().await;
    });
    let client = HomieClient::connect(options(endpoint, Duration::from_secs(2)))
        .await
        .expect("connect");

    let one = client.request::<_, Value>("one", json!({}));
    let two = client.request::<_, Value>("two", json!({}));
    let (one, two) = tokio::join!(one, two);

    assert_eq!(one.expect_err("one unavailable").code(), "unavailable");
    assert_eq!(two.expect_err("two unavailable").code(), "unavailable");
    client.close().await.expect("close");
    server.await.expect("server");
}

#[tokio::test]
async fn pending_request_limit_rejects_request_1025_with_backpressure() {
    const PENDING_LIMIT: usize = 1024;

    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let (seen_tx, mut seen_rx) = mpsc::unbounded_channel();
    let (release_tx, release_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut peer = socket.accept(&["hold"], &[], "daemon-a").await;
        for _ in 0..PENDING_LIMIT {
            let _request = peer.read_request().await;
            seen_tx.send(()).expect("request seen");
        }
        release_rx.await.expect("release server");
    });
    let client = Arc::new(
        HomieClient::connect(options(endpoint, Duration::from_secs(10)))
            .await
            .expect("connect"),
    );
    let mut pending = Vec::with_capacity(PENDING_LIMIT);
    for batch_start in (0..PENDING_LIMIT).step_by(64) {
        for _ in batch_start..batch_start + 64 {
            let client = client.clone();
            pending.push(tokio::spawn(async move {
                client.request::<_, Value>("hold", json!({})).await
            }));
        }
        for _ in 0..64 {
            seen_rx.recv().await.expect("server observed request");
        }
    }

    let error = client
        .request::<_, Value>("hold", json!({}))
        .await
        .expect_err("pending limit");

    assert_eq!(error.code(), "backpressure");
    release_tx.send(()).expect("release server");
    server.await.expect("server");
    for request in pending {
        assert_eq!(
            request
                .await
                .expect("request task")
                .expect_err("disconnect")
                .code(),
            "unavailable"
        );
    }
    client.close().await.expect("close");
}

#[tokio::test]
async fn close_enters_shutdown_and_stops_connection_work() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let _peer = socket.accept(&[], &[], "daemon-a").await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    });
    let client = HomieClient::connect(options(endpoint, Duration::from_secs(1)))
        .await
        .expect("connect");

    client.close().await.expect("close");

    assert_eq!(
        *client.connection_state().borrow(),
        ConnectionState::Shutdown
    );
    server.abort();
}

fn options(endpoint: std::path::PathBuf, request_timeout: Duration) -> ClientOptions {
    ClientOptions {
        endpoint: RuntimeEndpoint::new(endpoint).expect("absolute endpoint"),
        role: ClientRole::Cli,
        connect_timeout: Duration::from_secs(1),
        request_timeout,
    }
}
