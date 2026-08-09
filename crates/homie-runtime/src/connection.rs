use std::collections::HashMap;
use std::future::Future;
use std::os::fd::AsRawFd;
use std::pin::Pin;
use std::sync::Arc;

use homie_proto::stream::{StreamKind, StreamOpenRequest, StreamReset, StreamResetReason};
use homie_proto::transport::{
    FRAME_HEADER_LEN, Frame, FrameHeader, FrameKind, HelloRequest, HelloResponse,
    MAX_CONTROL_PAYLOAD, MAX_FRAME_LEN, MAX_OUTPUT_PAYLOAD, PREFACE_LEN, Preface, StableErrorCode,
    TransportError, WIRE_MAJOR, WIRE_MINOR,
};
use homie_proto::{ControlMessage, ErrorEnvelope, Method, RequestId};
use serde::Deserialize;
use serde::de::IgnoredAny;
use serde_json::Value;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::net::UnixStream;
use tokio::sync::{Semaphore, watch};
use tokio::task::{AbortHandle, JoinSet};

use crate::capabilities::method_capabilities;
use crate::dispatcher::{HandlerClass, RuntimeDispatcher, request_handlers};
use crate::event_stream::EventBounds;
use crate::runtime_actor::{ServiceError, ServiceResult};
use crate::server::{ServerIdentity, ShutdownHandle};
use crate::writer::{WriterError, WriterHandle, writer_channel};

const MAX_IN_FLIGHT_HANDLERS: usize = 1024;
const MAX_ACTIVE_STREAMS: usize = 64;

pub type ControlFuture<'a> = Pin<Box<dyn Future<Output = ServiceResult<Value>> + Send + 'a>>;

pub trait ControlHandler: Send + Sync + 'static {
    fn handle<'a>(&'a self, method: &'a str, params: Value) -> ControlFuture<'a>;
}

impl ControlHandler for RuntimeDispatcher {
    fn handle<'a>(&'a self, method: &'a str, params: Value) -> ControlFuture<'a> {
        Box::pin(self.dispatch(method, params))
    }
}

pub type ActiveStreamFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), StreamError>> + Send + 'a>>;

pub trait ActiveStream: Send + 'static {
    fn is_finished(&self) -> bool;

    fn handle<'a>(&'a mut self, frame: Frame) -> ActiveStreamFuture<'a>;

    fn close(self: Box<Self>) -> ActiveStreamFuture<'static>;
}

pub type StreamFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Box<dyn ActiveStream>, StreamError>> + Send + 'a>>;

pub trait StreamHandler: Send + Sync + 'static {
    fn capabilities(&self) -> Vec<StreamKind>;

    fn event_bounds(&self) -> EventBounds;

    fn open<'a>(
        &'a self,
        stream_id: u32,
        request: StreamOpenRequest,
        writer: WriterHandle,
    ) -> StreamFuture<'a>;
}

struct ConnectionServices {
    control: Arc<dyn ControlHandler>,
    streams: Arc<dyn StreamHandler>,
    shutdown: ShutdownHandle,
}

struct ConnectionState {
    handlers: JoinSet<u64>,
    in_flight: Arc<Semaphore>,
    active_handlers: HashMap<u64, ActiveHandler>,
    streams: ActiveStreamRegistry,
}

struct ActiveHandler {
    abort: AbortHandle,
    drain_on_shutdown: bool,
}

impl ConnectionState {
    fn new() -> Self {
        Self {
            handlers: JoinSet::new(),
            in_flight: Arc::new(Semaphore::new(MAX_IN_FLIGHT_HANDLERS)),
            active_handlers: HashMap::new(),
            streams: ActiveStreamRegistry::default(),
        }
    }
}

pub(crate) async fn serve_connection(
    mut socket: UnixStream,
    expected_peer_uid: u32,
    identity: ServerIdentity,
    control: Arc<dyn ControlHandler>,
    stream_handler: Arc<dyn StreamHandler>,
    shutdown: ShutdownHandle,
) -> Result<(), ConnectionError> {
    validate_peer_uid(&socket, expected_peer_uid)?;
    let mut shutdown_signal = shutdown.subscribe();
    tokio::select! {
        biased;
        () = wait_for_shutdown(&mut shutdown_signal) => return Ok(()),
        result = read_and_validate_hello(&mut socket) => result?,
    }

    let (read_half, write_half) = socket.into_split();
    let (writer, driver) = writer_channel();
    let writer_task = tokio::spawn(driver.run(write_half));
    let services = ConnectionServices {
        control,
        streams: stream_handler,
        shutdown,
    };
    writer.try_send_high(hello_ack_frame(&identity, services.streams.as_ref())?)?;

    let mut reader = read_half;
    let mut state = ConnectionState::new();
    let result = run_connection(
        &mut reader,
        &writer,
        &services,
        &mut state,
        &mut shutdown_signal,
    )
    .await;

    let mut result = result.map(|ConnectionExit::Shutdown| ());
    let graceful_shutdown = result.is_ok();
    if graceful_shutdown {
        if let Err(error) = drain_handlers(&mut state.handlers, &mut state.active_handlers).await {
            result = Err(error);
        }
    } else {
        state.handlers.abort_all();
        while state.handlers.join_next().await.is_some() {}
        state.active_handlers.clear();
    }
    if graceful_shutdown
        && let Err(error) = writer.flush().await
        && result.is_ok()
    {
        result = Err(error.into());
    }
    state.streams.close_all().await;
    writer.close();
    drop(writer);
    let _ = writer_task.await;
    result
}

enum ConnectionExit {
    Shutdown,
}

async fn run_connection(
    reader: &mut (impl AsyncRead + Unpin),
    writer: &WriterHandle,
    services: &ConnectionServices,
    state: &mut ConnectionState,
    shutdown_signal: &mut watch::Receiver<bool>,
) -> Result<ConnectionExit, ConnectionError> {
    loop {
        reap_completed_handlers(&mut state.handlers, &mut state.active_handlers)?;
        state.streams.prune_finished().await;

        enum Next {
            Frame(Result<Frame, ConnectionError>),
            HandlerCompleted(Option<Result<u64, tokio::task::JoinError>>),
            WriterClosed,
            Shutdown,
        }

        let next = tokio::select! {
            biased;
            () = wait_for_shutdown(shutdown_signal) => Next::Shutdown,
            frame = read_frame(reader) => Next::Frame(frame),
            _ = writer.closed() => Next::WriterClosed,
            result = state.handlers.join_next(), if !state.handlers.is_empty() => Next::HandlerCompleted(result),
        };
        let frame = match next {
            Next::Frame(frame) => frame?,
            Next::HandlerCompleted(Some(result)) => {
                reap_handler(result, &mut state.active_handlers)?;
                continue;
            }
            Next::HandlerCompleted(None) => return Err(ConnectionError::HandlerTask),
            Next::WriterClosed => {
                return Err(ConnectionError::Writer(WriterError::ConnectionClosed));
            }
            Next::Shutdown => return Ok(ConnectionExit::Shutdown),
        };

        match frame.header.kind {
            FrameKind::Request => {
                handle_request(
                    frame,
                    writer,
                    services.control.clone(),
                    &state.in_flight,
                    &mut state.handlers,
                    &mut state.active_handlers,
                    services.shutdown.clone(),
                )?;
            }
            FrameKind::Ping => {
                validate_ping(&frame)?;
                writer.try_send_high(Frame {
                    header: FrameHeader {
                        version: WIRE_MAJOR,
                        kind: FrameKind::Pong,
                        flags: 0,
                        stream_id: 0,
                        message_id: 0,
                        sequence: frame.header.sequence,
                    },
                    payload: Vec::new(),
                })?;
            }
            FrameKind::StreamOpen => {
                validate_client_stream_frame(&frame)?;
                let request: StreamOpenRequest = serde_json::from_slice(&frame.payload)
                    .map_err(|_| ConnectionError::Protocol)?;
                let stream_id = frame.header.stream_id;
                state.streams.reserve(stream_id).await?;
                match services
                    .streams
                    .open(stream_id, request, writer.clone())
                    .await
                {
                    Ok(stream) => state.streams.insert(stream_id, stream)?,
                    Err(StreamError::Protocol) => return Err(ConnectionError::Protocol),
                    Err(StreamError::Reset(reason)) => {
                        writer.try_send_high(stream_reset_frame(stream_id, reason)?)?;
                    }
                    Err(StreamError::ResetSent) => {}
                    Err(StreamError::Writer(error)) => return Err(error.into()),
                }
            }
            FrameKind::StreamClose | FrameKind::StreamReset => {
                validate_client_stream_frame(&frame)?;
                if frame.header.kind == FrameKind::StreamReset {
                    let _: StreamReset = serde_json::from_slice(&frame.payload)
                        .map_err(|_| ConnectionError::Protocol)?;
                }
                state.streams.close(frame.header.stream_id).await?;
            }
            FrameKind::Input | FrameKind::Resize => {
                validate_client_stream_frame(&frame)?;
                let stream_id = frame.header.stream_id;
                let result = state.streams.handle(stream_id, frame).await?;
                match result {
                    Ok(()) => {}
                    Err(StreamError::Protocol) => return Err(ConnectionError::Protocol),
                    Err(StreamError::Reset(reason)) => {
                        state.streams.close(stream_id).await?;
                        writer.try_send_high(stream_reset_frame(stream_id, reason)?)?;
                    }
                    Err(StreamError::ResetSent) => {
                        state.streams.close(stream_id).await?;
                    }
                    Err(StreamError::Writer(error)) => return Err(error.into()),
                }
            }
            _ => return Err(ConnectionError::Protocol),
        }
    }
}

fn handle_request(
    frame: Frame,
    writer: &WriterHandle,
    control: Arc<dyn ControlHandler>,
    in_flight: &Arc<Semaphore>,
    handlers: &mut JoinSet<u64>,
    active_handlers: &mut HashMap<u64, ActiveHandler>,
    shutdown: ShutdownHandle,
) -> Result<(), ConnectionError> {
    if frame.header.sequence != 0 {
        return Err(ConnectionError::Protocol);
    }
    let message: ControlMessage =
        serde_json::from_slice(&frame.payload).map_err(|_| ConnectionError::Protocol)?;
    let ControlMessage::Request {
        request_id,
        method,
        params,
    } = message
    else {
        return Err(ConnectionError::Protocol);
    };
    if request_id.as_u64() != frame.header.message_id || method.is_empty() {
        return Err(ConnectionError::Protocol);
    }
    if active_handlers.contains_key(&frame.header.message_id) {
        return Err(ConnectionError::Protocol);
    }

    let Ok(permit) = in_flight.clone().try_acquire_owned() else {
        writer.try_send_high(response_frame(
            frame.header.message_id,
            Err(ServiceError::Backpressure),
        )?)?;
        return Ok(());
    };
    let response_writer = writer.clone();
    let message_id = frame.header.message_id;
    let request_shutdown = method == Method::DAEMON_SHUTDOWN;
    let drain_on_shutdown = request_handlers()
        .iter()
        .find(|registration| registration.method == method)
        .is_none_or(|registration| registration.class == HandlerClass::Actor);
    let abort = handlers.spawn(async move {
        let _permit = permit;
        let result = control.handle(&method, params).await;
        let succeeded = result.is_ok();
        let Ok(frame) = response_frame(message_id, result) else {
            return message_id;
        };
        if response_writer.try_send_high(frame).is_ok()
            && request_shutdown
            && succeeded
            && response_writer.flush().await.is_ok()
        {
            shutdown.request_shutdown();
        }
        message_id
    });
    active_handlers.insert(
        message_id,
        ActiveHandler {
            abort,
            drain_on_shutdown,
        },
    );
    Ok(())
}

fn reap_completed_handlers(
    handlers: &mut JoinSet<u64>,
    active_handlers: &mut HashMap<u64, ActiveHandler>,
) -> Result<(), ConnectionError> {
    while let Some(result) = handlers.try_join_next() {
        reap_handler(result, active_handlers)?;
    }
    Ok(())
}

fn reap_handler(
    result: Result<u64, tokio::task::JoinError>,
    active_handlers: &mut HashMap<u64, ActiveHandler>,
) -> Result<(), ConnectionError> {
    let message_id = result.map_err(|_| ConnectionError::HandlerTask)?;
    if active_handlers.remove(&message_id).is_none() {
        return Err(ConnectionError::HandlerTask);
    }
    Ok(())
}

async fn drain_handlers(
    handlers: &mut JoinSet<u64>,
    active_handlers: &mut HashMap<u64, ActiveHandler>,
) -> Result<(), ConnectionError> {
    for handler in active_handlers.values() {
        if !handler.drain_on_shutdown {
            handler.abort.abort();
        }
    }

    while let Some(result) = handlers.join_next().await {
        match result {
            Ok(message_id) => {
                active_handlers.remove(&message_id);
            }
            Err(error) if error.is_cancelled() => {}
            Err(_) => return Err(ConnectionError::HandlerTask),
        }
    }
    active_handlers.clear();
    Ok(())
}

fn response_frame(message_id: u64, result: ServiceResult<Value>) -> Result<Frame, ConnectionError> {
    let request_id = RequestId::from(message_id);
    let message = match result {
        Ok(value) => ControlMessage::success(request_id, value),
        Err(error) => ControlMessage::failure(request_id, safe_error(error)),
    };
    Ok(Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::Response,
            flags: 0,
            stream_id: 0,
            message_id,
            sequence: 0,
        },
        payload: serde_json::to_vec(&message).map_err(|_| ConnectionError::Protocol)?,
    })
}

fn safe_error(error: ServiceError) -> ErrorEnvelope {
    let (code, message, retryable) = match error {
        ServiceError::Backpressure => (StableErrorCode::Backpressure, "runtime is busy", true),
        ServiceError::Unavailable | ServiceError::Cancelled => {
            (StableErrorCode::Unavailable, "runtime is unavailable", true)
        }
        ServiceError::Timeout => (
            StableErrorCode::Timeout,
            "runtime operation timed out",
            true,
        ),
        ServiceError::BadRequest(_) => (StableErrorCode::BadRequest, "invalid request", false),
        ServiceError::MethodNotFound(_) => {
            (StableErrorCode::MethodNotFound, "method not found", false)
        }
        ServiceError::Internal => (StableErrorCode::Internal, "runtime operation failed", false),
    };
    ErrorEnvelope::new(code.as_str(), message, retryable)
}

fn hello_ack_frame(
    identity: &ServerIdentity,
    stream_handler: &dyn StreamHandler,
) -> Result<Frame, ConnectionError> {
    let event_bounds = stream_handler.event_bounds();
    let response = HelloResponse {
        wire_major: WIRE_MAJOR,
        wire_minor: WIRE_MINOR,
        daemon_build: identity.daemon_build.clone(),
        daemon_version: identity.daemon_version.clone(),
        daemon_pid: identity.daemon_pid,
        daemon_instance_id: identity.daemon_instance_id.clone(),
        executable_hash: identity.executable_hash.clone(),
        method_capabilities: method_capabilities()
            .into_iter()
            .map(str::to_string)
            .collect(),
        stream_capabilities: stream_handler.capabilities(),
        event_oldest_seq: event_bounds.oldest_seq,
        event_latest_seq: event_bounds.latest_seq,
    };
    Ok(Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::HelloAck,
            flags: 0,
            stream_id: 0,
            message_id: 0,
            sequence: 0,
        },
        payload: serde_json::to_vec(&response).map_err(|_| ConnectionError::Protocol)?,
    })
}

fn stream_reset_frame(stream_id: u32, reason: StreamResetReason) -> Result<Frame, ConnectionError> {
    Ok(Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::StreamReset,
            flags: 0,
            stream_id,
            message_id: 0,
            sequence: 0,
        },
        payload: serde_json::to_vec(&StreamReset {
            reason,
            last_confirmed_offset: None,
            latest_seq: None,
        })
        .map_err(|_| ConnectionError::Protocol)?,
    })
}

async fn read_and_validate_hello(socket: &mut UnixStream) -> Result<(), ConnectionError> {
    let preface = read_preface(socket).await?;
    let hello_frame = read_frame(socket).await?;
    validate_hello_frame(&hello_frame)?;
    let hello: HelloRequest =
        serde_json::from_slice(&hello_frame.payload).map_err(|_| ConnectionError::Protocol)?;
    validate_hello(&hello, preface)
}

async fn wait_for_shutdown(shutdown: &mut watch::Receiver<bool>) {
    if *shutdown.borrow_and_update() {
        return;
    }
    while shutdown.changed().await.is_ok() {
        if *shutdown.borrow_and_update() {
            return;
        }
    }
}

async fn read_preface(reader: &mut (impl AsyncRead + Unpin)) -> Result<Preface, ConnectionError> {
    let mut encoded = [0_u8; PREFACE_LEN];
    reader.read_exact(&mut encoded).await?;
    let preface = Preface::decode(&encoded).map_err(|_| ConnectionError::Protocol)?;
    if preface.minor > WIRE_MINOR {
        return Err(ConnectionError::Protocol);
    }
    Ok(preface)
}

async fn read_frame(reader: &mut (impl AsyncRead + Unpin)) -> Result<Frame, ConnectionError> {
    let mut length = [0_u8; 4];
    reader.read_exact(&mut length).await?;
    let frame_len = u32::from_be_bytes(length) as usize;
    if !(FRAME_HEADER_LEN..=MAX_FRAME_LEN).contains(&frame_len) {
        return Err(ConnectionError::Protocol);
    }

    let mut encoded_header = [0_u8; FRAME_HEADER_LEN];
    reader.read_exact(&mut encoded_header).await?;
    let payload_len = frame_len - FRAME_HEADER_LEN;
    let header = decode_and_validate_header(&encoded_header, payload_len)?;

    let mut payload = vec![0_u8; payload_len];
    reader.read_exact(&mut payload).await?;
    validate_payload(header.kind, &payload)?;
    Ok(Frame { header, payload })
}

fn decode_and_validate_header(
    encoded: &[u8; FRAME_HEADER_LEN],
    payload_len: usize,
) -> Result<FrameHeader, ConnectionError> {
    let kind = FrameKind::try_from(encoded[2]).map_err(|_| ConnectionError::Protocol)?;
    let header = FrameHeader {
        version: u16::from_be_bytes([encoded[0], encoded[1]]),
        kind,
        flags: encoded[3],
        stream_id: u32::from_be_bytes([encoded[4], encoded[5], encoded[6], encoded[7]]),
        message_id: u64::from_be_bytes(
            encoded[8..16]
                .try_into()
                .map_err(|_| ConnectionError::Protocol)?,
        ),
        sequence: u64::from_be_bytes(
            encoded[16..24]
                .try_into()
                .map_err(|_| ConnectionError::Protocol)?,
        ),
    };
    if header.version != WIRE_MAJOR || header.flags != 0 {
        return Err(ConnectionError::Protocol);
    }

    let control = matches!(
        kind,
        FrameKind::Hello | FrameKind::HelloAck | FrameKind::Request | FrameKind::Response
    );
    let data = !control && !matches!(kind, FrameKind::Ping | FrameKind::Pong);
    if (control && header.stream_id != 0) || (data && header.stream_id == 0) {
        return Err(ConnectionError::Protocol);
    }
    if kind == FrameKind::StreamOpen && header.stream_id.is_multiple_of(2) {
        return Err(ConnectionError::Protocol);
    }
    if matches!(kind, FrameKind::Request | FrameKind::Response) && header.message_id == 0 {
        return Err(ConnectionError::Protocol);
    }
    if is_json(kind) && payload_len > MAX_CONTROL_PAYLOAD {
        return Err(ConnectionError::Protocol);
    }
    if kind == FrameKind::Output && payload_len > MAX_OUTPUT_PAYLOAD {
        return Err(ConnectionError::Protocol);
    }
    Ok(header)
}

fn validate_payload(kind: FrameKind, payload: &[u8]) -> Result<(), ConnectionError> {
    if is_json(kind) {
        if kind == FrameKind::StreamClose && payload.is_empty() {
            return Ok(());
        }
        let mut deserializer = serde_json::Deserializer::from_slice(payload);
        IgnoredAny::deserialize(&mut deserializer).map_err(|_| ConnectionError::Protocol)?;
        deserializer.end().map_err(|_| ConnectionError::Protocol)?;
        return Ok(());
    }

    let valid = match kind {
        FrameKind::Output => payload.len() >= 8,
        FrameKind::Resize => payload.len() == 4,
        FrameKind::ReplayBegin | FrameKind::ReplayEnd => payload.len() == 8,
        FrameKind::Ping | FrameKind::Pong => payload.is_empty(),
        _ => true,
    };
    if valid {
        Ok(())
    } else {
        Err(ConnectionError::Protocol)
    }
}

fn is_json(kind: FrameKind) -> bool {
    matches!(
        kind,
        FrameKind::Hello
            | FrameKind::HelloAck
            | FrameKind::Request
            | FrameKind::Response
            | FrameKind::Event
            | FrameKind::StreamOpen
            | FrameKind::StreamOpened
            | FrameKind::StreamReset
            | FrameKind::StreamClose
    )
}

fn validate_hello_frame(frame: &Frame) -> Result<(), ConnectionError> {
    if frame.header.kind != FrameKind::Hello
        || frame.header.message_id != 0
        || frame.header.sequence != 0
    {
        return Err(ConnectionError::Protocol);
    }
    Ok(())
}

fn validate_hello(hello: &HelloRequest, preface: Preface) -> Result<(), ConnectionError> {
    if hello.wire_major != WIRE_MAJOR
        || hello.wire_minor != WIRE_MINOR
        || hello.wire_major != preface.major
        || hello.wire_minor != preface.minor
        || hello.client_name.trim().is_empty()
        || hello.client_version.trim().is_empty()
        || hello.process_id == 0
    {
        return Err(ConnectionError::Protocol);
    }
    Ok(())
}

fn validate_ping(frame: &Frame) -> Result<(), ConnectionError> {
    if frame.header.stream_id != 0 || frame.header.message_id != 0 || !frame.payload.is_empty() {
        return Err(ConnectionError::Protocol);
    }
    Ok(())
}

fn validate_client_stream_frame(frame: &Frame) -> Result<(), ConnectionError> {
    if frame.header.message_id != 0 || frame.header.sequence != 0 {
        return Err(ConnectionError::Protocol);
    }
    Ok(())
}

#[derive(Default)]
struct ActiveStreamRegistry {
    streams: HashMap<u32, Box<dyn ActiveStream>>,
}

impl ActiveStreamRegistry {
    async fn reserve(&mut self, stream_id: u32) -> Result<(), ConnectionError> {
        self.prune_finished().await;
        if stream_id == 0
            || stream_id.is_multiple_of(2)
            || self.streams.len() >= MAX_ACTIVE_STREAMS
            || self.streams.contains_key(&stream_id)
        {
            return Err(ConnectionError::Protocol);
        }
        Ok(())
    }

    fn insert(
        &mut self,
        stream_id: u32,
        stream: Box<dyn ActiveStream>,
    ) -> Result<(), ConnectionError> {
        if self.streams.insert(stream_id, stream).is_some() {
            return Err(ConnectionError::Protocol);
        }
        Ok(())
    }

    async fn handle(
        &mut self,
        stream_id: u32,
        frame: Frame,
    ) -> Result<Result<(), StreamError>, ConnectionError> {
        let stream = self
            .streams
            .get_mut(&stream_id)
            .ok_or(ConnectionError::Protocol)?;
        Ok(stream.handle(frame).await)
    }

    async fn close(&mut self, stream_id: u32) -> Result<(), ConnectionError> {
        let stream = self
            .streams
            .remove(&stream_id)
            .ok_or(ConnectionError::Protocol)?;
        let _ = stream.close().await;
        Ok(())
    }

    async fn prune(&mut self, stream_id: u32) {
        if self
            .streams
            .get(&stream_id)
            .is_some_and(|stream| stream.is_finished())
            && let Some(stream) = self.streams.remove(&stream_id)
        {
            let _ = stream.close().await;
        }
    }

    async fn prune_finished(&mut self) {
        let finished = self
            .streams
            .iter()
            .filter_map(|(stream_id, stream)| stream.is_finished().then_some(*stream_id))
            .collect::<Vec<_>>();
        for stream_id in finished {
            self.prune(stream_id).await;
        }
    }

    async fn close_all(&mut self) {
        for (_, stream) in self.streams.drain() {
            let _ = stream.close().await;
        }
    }
}

fn validate_peer_uid(socket: &UnixStream, expected_peer_uid: u32) -> Result<(), ConnectionError> {
    let actual = peer_uid(socket)?;
    if actual == expected_peer_uid {
        Ok(())
    } else {
        Err(ConnectionError::Unauthorized)
    }
}

#[cfg(any(
    target_os = "macos",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
fn peer_uid(socket: &UnixStream) -> Result<u32, ConnectionError> {
    let mut uid: libc::uid_t = 0;
    let mut gid: libc::gid_t = 0;
    // SAFETY: fd is owned by socket and uid/gid point to initialized writable values.
    let result = unsafe { libc::getpeereid(socket.as_raw_fd(), &mut uid, &mut gid) };
    if result == 0 {
        Ok(uid)
    } else {
        Err(ConnectionError::PeerCredentials)
    }
}

#[cfg(target_os = "linux")]
fn peer_uid(socket: &UnixStream) -> Result<u32, ConnectionError> {
    let mut credentials = std::mem::MaybeUninit::<libc::ucred>::uninit();
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: fd is valid for the call, credentials has enough writable space, and length
    // describes that space. A successful call initializes credentials.
    let result = unsafe {
        libc::getsockopt(
            socket.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            credentials.as_mut_ptr().cast(),
            &mut length,
        )
    };
    if result != 0 || length as usize != std::mem::size_of::<libc::ucred>() {
        return Err(ConnectionError::PeerCredentials);
    }
    // SAFETY: getsockopt succeeded and returned the full ucred structure.
    Ok(unsafe { credentials.assume_init() }.uid)
}

#[derive(Debug, Error)]
pub(crate) enum ConnectionError {
    #[error("connection I/O failed")]
    Io(#[from] std::io::Error),
    #[error("peer credentials unavailable")]
    PeerCredentials,
    #[error("peer is unauthorized")]
    Unauthorized,
    #[error("protocol violation")]
    Protocol,
    #[error("control handler task failed")]
    HandlerTask,
    #[error(transparent)]
    Writer(#[from] WriterError),
}

#[derive(Clone, Copy, Debug, Error, PartialEq)]
pub enum StreamError {
    #[error("stream protocol violation")]
    Protocol,
    #[error("stream reset required")]
    Reset(StreamResetReason),
    #[error("stream reset already sent")]
    ResetSent,
    #[error(transparent)]
    Writer(#[from] WriterError),
}

impl From<TransportError> for ConnectionError {
    fn from(_: TransportError) -> Self {
        Self::Protocol
    }
}
