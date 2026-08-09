use std::sync::Arc;
use std::time::Duration;

use homie_proto::ControlMessage;
use homie_proto::transport::{
    ClientRole, EndpointRole, FRAME_HEADER_LEN, Frame, FrameHeader, FrameKind, HelloRequest,
    HelloResponse, MAX_FRAME_LEN, Preface, WIRE_MAJOR, WIRE_MINOR,
};
use tokio::io::{AsyncRead, AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::Instant;

use crate::client::{ClientError, ClientInner, ConnectionState};
use crate::streams::StreamAction;
use crate::writer::{self, WriterHandle, WriterReceiver};

const HEARTBEAT_IDLE: Duration = Duration::from_secs(25);
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(10);
const INITIAL_RECONNECT_BACKOFF: Duration = Duration::from_millis(500);
const MAX_RECONNECT_BACKOFF: Duration = Duration::from_secs(8);

pub(crate) async fn run(
    inner: Arc<ClientInner>,
    initial: oneshot::Sender<Result<(), ClientError>>,
) {
    let mut initial = Some(initial);
    let mut shutdown = inner.shutdown_tx.subscribe();
    let mut backoff = INITIAL_RECONNECT_BACKOFF;
    let mut reconnect_attempt = 0_u32;

    loop {
        if *shutdown.borrow() {
            break;
        }
        inner.set_state(if reconnect_attempt == 0 {
            ConnectionState::Connecting
        } else {
            ConnectionState::Reconnecting {
                attempt: reconnect_attempt,
                delay: backoff,
            }
        });

        match connect_once(&inner).await {
            Ok((reader, writer_half, hello)) => {
                backoff = INITIAL_RECONNECT_BACKOFF;
                reconnect_attempt = 0;
                let (writer, receiver) = writer::channel();
                inner.set_connected(hello, writer.clone());
                if let Err(error) = inner.streams.reopen_all(&writer) {
                    inner.set_state(ConnectionState::Degraded {
                        code: error.code().to_string(),
                    });
                    inner.clear_writer();
                    continue;
                }
                if let Some(initial) = initial.take() {
                    let _ = initial.send(Ok(()));
                }

                let result = run_connected(
                    inner.clone(),
                    reader,
                    writer_half,
                    writer,
                    receiver,
                    &mut shutdown,
                )
                .await;
                inner.streams.connection_lost();
                inner.clear_writer();
                inner.pending.fail_all_unavailable();
                if *shutdown.borrow() {
                    break;
                }
                if let Err(error) = result {
                    inner.set_state(ConnectionState::Degraded {
                        code: error.code().to_string(),
                    });
                    if is_fatal_connection_error(&error) {
                        break;
                    }
                }
            }
            Err(error) => {
                if is_fatal_connection_error(&error) {
                    inner.set_state(ConnectionState::Disconnected);
                    if let Some(initial) = initial.take() {
                        let _ = initial.send(Err(error));
                    }
                    break;
                }
            }
        }

        reconnect_attempt = reconnect_attempt.saturating_add(1);
        inner.set_state(ConnectionState::Reconnecting {
            attempt: reconnect_attempt,
            delay: backoff,
        });
        tokio::select! {
            () = tokio::time::sleep(backoff) => {}
            result = shutdown.changed() => {
                if result.is_err() || *shutdown.borrow() {
                    break;
                }
            }
        }
        backoff = backoff.saturating_mul(2).min(MAX_RECONNECT_BACKOFF);
    }

    inner.clear_writer();
    inner.pending.fail_all_unavailable();
    if *shutdown.borrow() {
        inner.set_state(ConnectionState::Shutdown);
    } else if !matches!(*inner.state_tx.borrow(), ConnectionState::Disconnected) {
        inner.set_state(ConnectionState::Disconnected);
    }
}

async fn connect_once(
    inner: &ClientInner,
) -> Result<(OwnedReadHalf, OwnedWriteHalf, HelloResponse), ClientError> {
    let stream = tokio::time::timeout(
        inner.options.connect_timeout,
        UnixStream::connect(inner.options.endpoint.as_path()),
    )
    .await
    .map_err(|_| ClientError::Timeout)?
    .map_err(|_| ClientError::Unavailable)?;
    inner.set_state(ConnectionState::Handshaking);
    handshake(stream, inner.options.role, inner.options.connect_timeout).await
}

async fn handshake(
    mut stream: UnixStream,
    role: ClientRole,
    timeout: Duration,
) -> Result<(OwnedReadHalf, OwnedWriteHalf, HelloResponse), ClientError> {
    let hello = HelloRequest {
        wire_major: WIRE_MAJOR,
        wire_minor: WIRE_MINOR,
        client_name: "homie-client".to_string(),
        client_version: env!("CARGO_PKG_VERSION").to_string(),
        client_role: role,
        process_id: std::process::id(),
    };
    let frame = Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::Hello,
            flags: 0,
            stream_id: 0,
            message_id: 0,
            sequence: 0,
        },
        payload: serde_json::to_vec(&hello)?,
    };
    let exchange = async {
        stream
            .write_all(
                &Preface {
                    major: WIRE_MAJOR,
                    minor: WIRE_MINOR,
                }
                .encode(),
            )
            .await
            .map_err(|_| ClientError::Unavailable)?;
        stream
            .write_all(
                &frame
                    .encode(EndpointRole::Client)
                    .map_err(|error| ClientError::Protocol(error.to_string()))?,
            )
            .await
            .map_err(|_| ClientError::Unavailable)?;
        read_frame(&mut stream).await
    };
    let frame = tokio::time::timeout(timeout, exchange)
        .await
        .map_err(|_| ClientError::Timeout)??;
    if frame.header.kind != FrameKind::HelloAck {
        return Err(ClientError::Protocol(
            "first daemon frame was not HelloAck".to_string(),
        ));
    }
    let hello: HelloResponse = serde_json::from_slice(&frame.payload)?;
    if hello.wire_major != WIRE_MAJOR {
        return Err(ClientError::VersionMismatch);
    }
    let (reader, writer) = stream.into_split();
    Ok((reader, writer, hello))
}

async fn run_connected(
    inner: Arc<ClientInner>,
    mut reader: OwnedReadHalf,
    writer_half: OwnedWriteHalf,
    writer: WriterHandle,
    receiver: WriterReceiver,
    shutdown: &mut watch::Receiver<bool>,
) -> Result<(), ClientError> {
    let (activity_tx, mut activity_rx) = watch::channel(Instant::now());
    let (writer_failed_tx, mut writer_failed_rx) = mpsc::channel(1);
    let writer_task = tokio::spawn(write_frames(
        writer_half,
        receiver,
        activity_tx,
        writer_failed_tx,
    ));
    let mut last_activity = Instant::now();
    let mut ping_sequence = 0_u64;
    let mut outstanding_ping = None;

    let result = loop {
        let deadline = outstanding_ping
            .map(|(_, sent): (u64, Instant)| sent + HEARTBEAT_TIMEOUT)
            .unwrap_or(last_activity + HEARTBEAT_IDLE);
        tokio::select! {
            result = shutdown.changed() => {
                if result.is_err() || *shutdown.borrow() {
                    break Ok(());
                }
            }
            failed = writer_failed_rx.recv() => {
                if failed.is_some() {
                    break Err(ClientError::Unavailable);
                }
            }
            activity = activity_rx.changed() => {
                if activity.is_ok() {
                    last_activity = *activity_rx.borrow_and_update();
                }
            }
            frame = read_frame(&mut reader) => {
                let frame = match frame {
                    Ok(frame) => frame,
                    Err(error) => break Err(error),
                };
                last_activity = Instant::now();
                match frame.header.kind {
                    FrameKind::Response => dispatch_response(&inner, frame)?,
                    FrameKind::Ping => {
                        validate_heartbeat_frame(&frame)?;
                        writer.try_send_high(Frame {
                            header: FrameHeader {
                                version: WIRE_MAJOR,
                                kind: FrameKind::Pong,
                                flags: 0,
                                stream_id: frame.header.stream_id,
                                message_id: frame.header.message_id,
                                sequence: frame.header.sequence,
                            },
                            payload: Vec::new(),
                        })?;
                    }
                    FrameKind::Pong => {
                        validate_heartbeat_frame(&frame)?;
                        match outstanding_ping {
                            Some((sequence, _)) if sequence == frame.header.sequence => {
                                outstanding_ping = None;
                            }
                            Some((sequence, _)) => {
                                break Err(ClientError::Protocol(format!(
                                    "pong sequence {} does not match outstanding ping {sequence}",
                                    frame.header.sequence
                                )));
                            }
                            None => {
                                break Err(ClientError::Protocol(
                                    "received pong without an outstanding ping".to_string(),
                                ));
                            }
                        }
                    }
                    _ => dispatch_stream_frame(&inner, &writer, frame)?,
                }
            }
            () = tokio::time::sleep_until(deadline) => {
                if outstanding_ping.is_some() {
                    break Err(ClientError::Timeout);
                }
                ping_sequence = ping_sequence.wrapping_add(1);
                if ping_sequence == 0 {
                    ping_sequence = 1;
                }
                writer.try_send_high(Frame {
                    header: FrameHeader {
                        version: WIRE_MAJOR,
                        kind: FrameKind::Ping,
                        flags: 0,
                        stream_id: 0,
                        message_id: 0,
                        sequence: ping_sequence,
                    },
                    payload: Vec::new(),
                })?;
                outstanding_ping = Some((ping_sequence, Instant::now()));
            }
        }
    };

    writer_task.abort();
    let _ = writer_task.await;
    result
}

fn validate_heartbeat_frame(frame: &Frame) -> Result<(), ClientError> {
    if frame.header.stream_id != 0 || frame.header.message_id != 0 || !frame.payload.is_empty() {
        return Err(ClientError::Protocol("invalid heartbeat frame".to_string()));
    }
    Ok(())
}

fn dispatch_stream_frame(
    inner: &Arc<ClientInner>,
    writer: &WriterHandle,
    frame: Frame,
) -> Result<(), ClientError> {
    match inner.streams.dispatch(frame)? {
        StreamAction::None => {}
        StreamAction::RecoverEvent(stream_id) => {
            tokio::spawn(inner.clone().recover_event(stream_id));
        }
        StreamAction::ReopenTerminal(stream_id) => {
            writer.try_send_high(crate::streams::stream_close_frame(stream_id))?;
            writer.try_send_high(inner.streams.reopen_terminal_frame(stream_id)?)?;
        }
    }
    Ok(())
}

fn dispatch_response(inner: &ClientInner, frame: Frame) -> Result<(), ClientError> {
    let response: ControlMessage = serde_json::from_slice(&frame.payload)?;
    let ControlMessage::Response {
        request_id,
        ok,
        result,
        error,
    } = response
    else {
        return Err(ClientError::Protocol(
            "response frame contained a non-response payload".to_string(),
        ));
    };
    if request_id.as_u64() != frame.header.message_id {
        return Err(ClientError::Protocol(
            "response payload message id did not match frame".to_string(),
        ));
    }
    let result = if ok {
        Ok(result.unwrap_or(serde_json::Value::Null))
    } else {
        Err(ClientError::Remote(Box::new(error.unwrap_or_else(|| {
            homie_proto::ErrorEnvelope::new("internal", "daemon response omitted its error", false)
        }))))
    };
    inner.pending.resolve(frame.header.message_id, result);
    Ok(())
}

async fn write_frames(
    mut writer: OwnedWriteHalf,
    mut receiver: WriterReceiver,
    activity: watch::Sender<Instant>,
    failed: mpsc::Sender<()>,
) {
    while let Some(frame) = receiver.next().await {
        let encoded = match frame.encode(EndpointRole::Client) {
            Ok(encoded) => encoded,
            Err(_) => {
                let _ = failed.send(()).await;
                return;
            }
        };
        if writer.write_all(&encoded).await.is_err() {
            let _ = failed.send(()).await;
            return;
        }
        activity.send_replace(Instant::now());
    }
}

async fn read_frame(reader: &mut (impl AsyncRead + Unpin)) -> Result<Frame, ClientError> {
    let mut length = [0_u8; 4];
    reader
        .read_exact(&mut length)
        .await
        .map_err(|_| ClientError::Unavailable)?;
    let frame_len = u32::from_be_bytes(length) as usize;
    if !(FRAME_HEADER_LEN..=MAX_FRAME_LEN).contains(&frame_len) {
        return Err(ClientError::Protocol(
            "daemon sent an invalid frame length".to_string(),
        ));
    }
    let mut encoded = vec![0_u8; 4 + frame_len];
    encoded[..4].copy_from_slice(&length);
    reader
        .read_exact(&mut encoded[4..])
        .await
        .map_err(|_| ClientError::Unavailable)?;
    Frame::decode(&encoded, EndpointRole::Server)
        .map_err(|error| ClientError::Protocol(error.to_string()))?
        .map(|(frame, _)| frame)
        .ok_or_else(|| ClientError::Protocol("daemon sent an incomplete frame".to_string()))
}

fn is_fatal_connection_error(error: &ClientError) -> bool {
    matches!(
        error,
        ClientError::BadRequest(_)
            | ClientError::VersionMismatch
            | ClientError::Unauthorized
            | ClientError::Protocol(_)
            | ClientError::Internal
            | ClientError::Json(_)
    )
}
