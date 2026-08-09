use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use homie_proto::grid::{GridCell, TermColor, TermStyle, encode_row, terminal_cell_count};
use homie_proto::stream::TerminalStreamOpen;
use homie_proto::transport::{Frame, FrameHeader, FrameKind, MAX_OUTPUT_PAYLOAD, WIRE_MAJOR};
use thiserror::Error;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::{Mutex, mpsc, oneshot};

use crate::dispatcher::RuntimeResponse;
use crate::runtime_actor::{RuntimeActorHandle, RuntimeCall, RuntimeReply, ServiceError};
use crate::screen::HeadlessScreen;
use crate::writer::{LowEnqueue, StreamPosition, WriterError, WriterHandle};

const OUTPUT_BYTES_PER_CHUNK: usize = MAX_OUTPUT_PAYLOAD - size_of::<u64>();
const ACTIVE_TAIL_DELAY: Duration = Duration::from_millis(50);
const IDLE_TAIL_DELAY: Duration = Duration::from_millis(250);
const SOURCE_COMMAND_CAPACITY: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalSourceDescriptor {
    pub session_id: String,
    pub output_path: PathBuf,
    pub cols: u16,
    pub rows: u16,
    pub modes: Vec<u8>,
}

pub trait TerminalBackend: Send + Sync + 'static {
    fn describe(
        &self,
        session_id: &str,
    ) -> impl Future<Output = Result<TerminalSourceDescriptor, TerminalStreamError>> + Send;

    fn send_input(
        &self,
        session_id: &str,
        input: Vec<u8>,
    ) -> impl Future<Output = Result<(), TerminalStreamError>> + Send;

    fn resize(
        &self,
        session_id: &str,
        cols: u16,
        rows: u16,
    ) -> impl Future<Output = Result<(), TerminalStreamError>> + Send;
}

#[derive(Clone)]
pub struct RuntimeTerminalBackend {
    actor: RuntimeActorHandle,
}

impl RuntimeTerminalBackend {
    #[must_use]
    pub fn new(actor: RuntimeActorHandle) -> Self {
        Self { actor }
    }

    async fn call(&self, request: RuntimeCall) -> Result<RuntimeReply, TerminalStreamError> {
        let reply = self.actor.try_call(request).map_err(map_actor_error)?;
        reply
            .await
            .map_err(|_| TerminalStreamError::SourceUnavailable)?
            .map_err(map_actor_error)
    }

    async fn call_ack(&self, request: RuntimeCall) -> Result<(), TerminalStreamError> {
        match self.call(request).await? {
            RuntimeReply::Response(RuntimeResponse::Ack(response)) if response.ok => Ok(()),
            _ => Err(TerminalStreamError::Backend),
        }
    }
}

impl TerminalBackend for RuntimeTerminalBackend {
    async fn describe(
        &self,
        session_id: &str,
    ) -> Result<TerminalSourceDescriptor, TerminalStreamError> {
        self.call(RuntimeCall::TerminalDescribe {
            session_id: session_id.to_string(),
        })
        .await?
        .into_terminal_descriptor()
        .map_err(map_actor_error)
    }

    async fn send_input(
        &self,
        session_id: &str,
        input: Vec<u8>,
    ) -> Result<(), TerminalStreamError> {
        self.call_ack(RuntimeCall::TerminalInput {
            session_id: session_id.to_string(),
            bytes: input,
        })
        .await
    }

    async fn resize(
        &self,
        session_id: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(), TerminalStreamError> {
        self.call_ack(RuntimeCall::TerminalResize {
            session_id: session_id.to_string(),
            cols,
            rows,
        })
        .await
    }
}

fn map_actor_error(error: ServiceError) -> TerminalStreamError {
    match error {
        ServiceError::Backpressure => TerminalStreamError::Backpressure,
        ServiceError::Unavailable => TerminalStreamError::SourceUnavailable,
        ServiceError::Timeout
        | ServiceError::Cancelled
        | ServiceError::BadRequest(_)
        | ServiceError::MethodNotFound(_)
        | ServiceError::Internal => TerminalStreamError::Backend,
    }
}

#[derive(Debug, Error)]
pub enum TerminalStreamError {
    #[error("terminal backend failed")]
    Backend,
    #[error("terminal backend queue is full")]
    Backpressure,
    #[error("terminal source descriptor is invalid")]
    InvalidDescriptor,
    #[error("terminal client frame is invalid")]
    InvalidFrame,
    #[error("terminal source is unavailable")]
    SourceUnavailable,
    #[error("terminal writer failed")]
    Writer(#[from] WriterError),
    #[error("terminal subscriber is too slow")]
    SlowConsumer,
    #[error("terminal output log failed")]
    Io(#[from] std::io::Error),
    #[error("terminal stream sequence overflowed")]
    SequenceOverflow,
}

pub struct TerminalSourceManager<B> {
    inner: Arc<ManagerInner<B>>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TerminalSourceStats {
    pub source_count: usize,
    pub output_log_readers: usize,
    pub subscriber_count: usize,
}

impl<B> Clone for TerminalSourceManager<B> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<B: TerminalBackend> TerminalSourceManager<B> {
    #[must_use]
    pub fn new(backend: Arc<B>) -> Self {
        Self {
            inner: Arc::new(ManagerInner {
                backend,
                sources: Mutex::new(HashMap::new()),
                next_subscriber_token: AtomicU64::new(1),
            }),
        }
    }

    pub async fn open(
        &self,
        stream_id: u32,
        request: TerminalStreamOpen,
        writer: WriterHandle,
    ) -> Result<TerminalSubscription, TerminalStreamError> {
        if stream_id == 0 {
            return Err(TerminalStreamError::InvalidDescriptor);
        }
        let source = self.source(&request.session_id).await?;
        let subscriber_token = self.next_subscriber_token();
        let detached = Arc::new(AtomicU64::new(0));
        let (reply, received) = oneshot::channel();
        source
            .send(SourceCommand::Subscribe {
                subscriber_token,
                stream_id,
                output_offset: request.output_offset,
                writer,
                detached: detached.clone(),
                reply,
            })
            .await
            .map_err(|_| TerminalStreamError::SourceUnavailable)?;
        received
            .await
            .map_err(|_| TerminalStreamError::SourceUnavailable)??;
        Ok(TerminalSubscription {
            subscriber_token,
            stream_id,
            source,
            detached,
        })
    }

    pub async fn stats(&self) -> TerminalSourceStats {
        let sources = {
            let mut sources = self.inner.sources.lock().await;
            let mut live_sources = Vec::with_capacity(sources.len());
            sources.retain(|_, source| {
                let Some(source) = source.upgrade() else {
                    return false;
                };
                if source.is_closed() {
                    return false;
                }
                live_sources.push(source);
                true
            });
            live_sources
        };
        let mut subscriber_count = 0;
        for source in &sources {
            let (reply, received) = oneshot::channel();
            if source.send(SourceCommand::Stats { reply }).await.is_ok()
                && let Ok(count) = received.await
            {
                subscriber_count += count;
            }
        }
        let source_count = sources.len();
        TerminalSourceStats {
            source_count,
            output_log_readers: source_count,
            subscriber_count,
        }
    }

    async fn source(
        &self,
        session_id: &str,
    ) -> Result<mpsc::Sender<SourceCommand>, TerminalStreamError> {
        let mut sources = self.inner.sources.lock().await;
        if let Some(source) = sources.get(session_id).and_then(mpsc::WeakSender::upgrade)
            && !source.is_closed()
        {
            return Ok(source);
        }
        sources.remove(session_id);

        let descriptor = self.inner.backend.describe(session_id).await?;
        if descriptor.session_id != session_id
            || terminal_cell_count(descriptor.cols, descriptor.rows).is_none()
        {
            return Err(TerminalStreamError::InvalidDescriptor);
        }
        let (source, commands) = mpsc::channel(SOURCE_COMMAND_CAPACITY);
        let (ready, initialized) = oneshot::channel();
        let source_descriptor = descriptor.clone();
        let backend = self.inner.backend.clone();
        let _source_task = tokio::spawn(async move {
            let result = SourceActor::open(source_descriptor, backend).await;
            match result {
                Ok(actor) => {
                    let _ = ready.send(Ok(()));
                    let _ = actor.run(commands).await;
                }
                Err(error) => {
                    let _ = ready.send(Err(error));
                }
            }
        });
        initialized
            .await
            .map_err(|_| TerminalStreamError::SourceUnavailable)??;
        sources.insert(descriptor.session_id, source.downgrade());
        Ok(source)
    }

    fn next_subscriber_token(&self) -> u64 {
        loop {
            let token = self
                .inner
                .next_subscriber_token
                .fetch_add(1, Ordering::Relaxed);
            if token != 0 {
                return token;
            }
        }
    }
}

struct ManagerInner<B> {
    backend: Arc<B>,
    sources: Mutex<HashMap<String, mpsc::WeakSender<SourceCommand>>>,
    next_subscriber_token: AtomicU64,
}

pub struct TerminalSubscription {
    subscriber_token: u64,
    stream_id: u32,
    source: mpsc::Sender<SourceCommand>,
    detached: Arc<AtomicU64>,
}

impl TerminalSubscription {
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.detached.load(Ordering::Acquire) != 0 || self.source.is_closed()
    }

    pub async fn handle_frame(&self, frame: Frame) -> Result<(), TerminalStreamError> {
        if frame.header.stream_id != self.stream_id
            || frame.header.message_id != 0
            || frame.header.sequence != 0
        {
            return Err(TerminalStreamError::InvalidFrame);
        }
        if self.detached.load(Ordering::Acquire) != 0 {
            return Err(TerminalStreamError::SourceUnavailable);
        }
        match frame.header.kind {
            FrameKind::Input => {
                let (reply, received) = oneshot::channel();
                self.source
                    .send(SourceCommand::Input {
                        subscriber_token: self.subscriber_token,
                        input: frame.payload,
                        reply,
                    })
                    .await
                    .map_err(|_| TerminalStreamError::SourceUnavailable)?;
                received
                    .await
                    .map_err(|_| TerminalStreamError::SourceUnavailable)?
            }
            FrameKind::Resize => {
                let [cols_high, cols_low, rows_high, rows_low] = frame.payload.as_slice() else {
                    return Err(TerminalStreamError::InvalidFrame);
                };
                let cols = u16::from_be_bytes([*cols_high, *cols_low]);
                let rows = u16::from_be_bytes([*rows_high, *rows_low]);
                if terminal_cell_count(cols, rows).is_none() {
                    return Err(TerminalStreamError::InvalidFrame);
                }
                let (reply, received) = oneshot::channel();
                self.source
                    .send(SourceCommand::Resize {
                        subscriber_token: self.subscriber_token,
                        cols,
                        rows,
                        reply,
                    })
                    .await
                    .map_err(|_| TerminalStreamError::SourceUnavailable)?;
                received
                    .await
                    .map_err(|_| TerminalStreamError::SourceUnavailable)?
            }
            FrameKind::StreamClose if frame.payload.is_empty() => self.detach().await,
            _ => Err(TerminalStreamError::InvalidFrame),
        }
    }

    pub async fn close(&self) -> Result<(), TerminalStreamError> {
        self.detach().await
    }

    async fn detach(&self) -> Result<(), TerminalStreamError> {
        if self.detached.swap(1, Ordering::AcqRel) != 0 {
            return Ok(());
        }
        let (reply, received) = oneshot::channel();
        self.source
            .send(SourceCommand::Detach {
                subscriber_token: self.subscriber_token,
                reply: Some(reply),
            })
            .await
            .map_err(|_| TerminalStreamError::SourceUnavailable)?;
        received
            .await
            .map_err(|_| TerminalStreamError::SourceUnavailable)
    }
}

impl Drop for TerminalSubscription {
    fn drop(&mut self) {
        if self.detached.swap(1, Ordering::AcqRel) == 0 {
            let _ = self.source.try_send(SourceCommand::Detach {
                subscriber_token: self.subscriber_token,
                reply: None,
            });
        }
    }
}

enum SourceCommand {
    Subscribe {
        subscriber_token: u64,
        stream_id: u32,
        output_offset: u64,
        writer: WriterHandle,
        detached: Arc<AtomicU64>,
        reply: oneshot::Sender<Result<(), TerminalStreamError>>,
    },
    Input {
        subscriber_token: u64,
        input: Vec<u8>,
        reply: oneshot::Sender<Result<(), TerminalStreamError>>,
    },
    Resize {
        subscriber_token: u64,
        cols: u16,
        rows: u16,
        reply: oneshot::Sender<Result<(), TerminalStreamError>>,
    },
    Detach {
        subscriber_token: u64,
        reply: Option<oneshot::Sender<()>>,
    },
    Stats {
        reply: oneshot::Sender<usize>,
    },
}

struct SourceActor<B> {
    descriptor: TerminalSourceDescriptor,
    backend: Arc<B>,
    file: File,
    screen: HeadlessScreen,
    live_offset: u64,
    subscribers: HashMap<u64, Subscriber>,
}

impl<B: TerminalBackend> SourceActor<B> {
    async fn open(
        descriptor: TerminalSourceDescriptor,
        backend: Arc<B>,
    ) -> Result<Self, TerminalStreamError> {
        terminal_cell_count(descriptor.cols, descriptor.rows)
            .ok_or(TerminalStreamError::InvalidDescriptor)?;
        let mut file = File::open(&descriptor.output_path).await?;
        let mut screen =
            HeadlessScreen::new(usize::from(descriptor.cols), usize::from(descriptor.rows));
        let mut live_offset = 0_u64;
        let mut buffer = vec![0_u8; OUTPUT_BYTES_PER_CHUNK];
        loop {
            let read = file.read(&mut buffer).await?;
            if read == 0 {
                break;
            }
            screen.feed(&buffer[..read]);
            live_offset = live_offset
                .checked_add(read as u64)
                .ok_or(TerminalStreamError::SequenceOverflow)?;
        }
        Ok(Self {
            descriptor,
            backend,
            file,
            screen,
            live_offset,
            subscribers: HashMap::new(),
        })
    }

    async fn run(
        mut self,
        mut commands: mpsc::Receiver<SourceCommand>,
    ) -> Result<(), TerminalStreamError> {
        let tail_sleep = tokio::time::sleep(IDLE_TAIL_DELAY);
        tokio::pin!(tail_sleep);
        let mut had_subscribers = false;
        loop {
            tokio::select! {
                command = commands.recv() => {
                    let Some(command) = command else {
                        return Ok(());
                    };
                    self.handle_command(command).await?;
                    had_subscribers |= !self.subscribers.is_empty();
                }
                () = &mut tail_sleep => {
                    self.remove_detached_subscribers();
                    if had_subscribers && self.subscribers.is_empty() {
                        return Ok(());
                    }
                    let delay = if self.tail_once().await? {
                        ACTIVE_TAIL_DELAY
                    } else {
                        IDLE_TAIL_DELAY
                    };
                    tail_sleep
                        .as_mut()
                        .reset(tokio::time::Instant::now() + delay);
                }
            }
        }
    }

    async fn handle_command(&mut self, command: SourceCommand) -> Result<(), TerminalStreamError> {
        match command {
            SourceCommand::Subscribe {
                subscriber_token,
                stream_id,
                output_offset,
                writer,
                detached,
                reply,
            } => {
                let detached_on_error = detached.clone();
                let result = self
                    .subscribe(subscriber_token, stream_id, output_offset, writer, detached)
                    .await;
                let failed = result.is_err();
                if failed {
                    detached_on_error.store(1, Ordering::Release);
                }
                let _ = reply.send(result);
                if failed {
                    self.subscribers.remove(&subscriber_token);
                }
            }
            SourceCommand::Input {
                subscriber_token,
                input,
                reply,
            } => {
                let result = if self.subscribers.contains_key(&subscriber_token) {
                    self.backend
                        .send_input(&self.descriptor.session_id, input)
                        .await
                } else {
                    Err(TerminalStreamError::SourceUnavailable)
                };
                let _ = reply.send(result);
            }
            SourceCommand::Resize {
                subscriber_token,
                cols,
                rows,
                reply,
            } => {
                let result = if self.subscribers.contains_key(&subscriber_token) {
                    self.backend
                        .resize(&self.descriptor.session_id, cols, rows)
                        .await
                } else {
                    Err(TerminalStreamError::SourceUnavailable)
                };
                if result.is_ok() {
                    self.descriptor.cols = cols;
                    self.descriptor.rows = rows;
                    self.screen.resize(usize::from(cols), usize::from(rows));
                    self.broadcast_grid()?;
                }
                let _ = reply.send(result);
            }
            SourceCommand::Detach {
                subscriber_token,
                reply,
            } => {
                if let Some(subscriber) = self.subscribers.get(&subscriber_token) {
                    subscriber.detached.store(1, Ordering::Release);
                }
                if let Some(subscriber) = self.subscribers.remove(&subscriber_token) {
                    subscriber.writer.reset_stream(subscriber.stream_id);
                }
                if let Some(reply) = reply {
                    let _ = reply.send(());
                }
            }
            SourceCommand::Stats { reply } => {
                let _ = reply.send(self.subscribers.len());
            }
        }
        Ok(())
    }

    async fn subscribe(
        &mut self,
        subscriber_token: u64,
        stream_id: u32,
        requested_offset: u64,
        writer: WriterHandle,
        detached: Arc<AtomicU64>,
    ) -> Result<(), TerminalStreamError> {
        writer.try_send_high(frame(FrameKind::StreamOpened, stream_id, 0, b"{}".to_vec()))?;
        let replay_begin = requested_offset.min(self.live_offset);
        let mut subscriber = Subscriber {
            stream_id,
            next_sequence: 1,
            confirmed_offset: replay_begin,
            writer,
            detached,
        };
        subscriber.send_low(
            FrameKind::ReplayBegin,
            replay_begin.to_be_bytes().to_vec(),
            replay_begin,
        )?;

        self.file
            .seek(std::io::SeekFrom::Start(replay_begin))
            .await?;
        let mut offset = replay_begin;
        let mut buffer = vec![0_u8; OUTPUT_BYTES_PER_CHUNK];
        while offset < self.live_offset {
            let remaining = usize::try_from(self.live_offset - offset)
                .unwrap_or(usize::MAX)
                .min(buffer.len());
            let read = self.file.read(&mut buffer[..remaining]).await?;
            if read == 0 {
                break;
            }
            subscriber.send_output(offset, &buffer[..read])?;
            offset += read as u64;
        }
        self.file
            .seek(std::io::SeekFrom::Start(self.live_offset))
            .await?;
        subscriber.send_low(
            FrameKind::ReplayEnd,
            self.live_offset.to_be_bytes().to_vec(),
            self.live_offset,
        )?;
        subscriber.send_low(
            FrameKind::Grid,
            encode_full_grid(&self.screen, self.descriptor.cols, self.descriptor.rows)?,
            self.live_offset,
        )?;
        subscriber.send_low(
            FrameKind::Modes,
            self.descriptor.modes.clone(),
            self.live_offset,
        )?;
        self.subscribers.insert(subscriber_token, subscriber);
        Ok(())
    }

    fn remove_detached_subscribers(&mut self) {
        self.subscribers.retain(|_, subscriber| {
            if subscriber.detached.load(Ordering::Acquire) == 0 {
                return true;
            }
            subscriber.writer.reset_stream(subscriber.stream_id);
            false
        });
    }

    async fn tail_once(&mut self) -> Result<bool, TerminalStreamError> {
        self.file
            .seek(std::io::SeekFrom::Start(self.live_offset))
            .await?;
        let mut bytes = vec![0_u8; OUTPUT_BYTES_PER_CHUNK];
        let read = self.file.read(&mut bytes).await?;
        if read == 0 {
            return Ok(false);
        }
        bytes.truncate(read);
        let offset = self.live_offset;
        self.live_offset = self
            .live_offset
            .checked_add(read as u64)
            .ok_or(TerminalStreamError::SequenceOverflow)?;
        self.screen.feed(&bytes);
        let grid = encode_full_grid(&self.screen, self.descriptor.cols, self.descriptor.rows)?;
        self.broadcast_output(offset, &bytes, &grid);
        Ok(true)
    }

    fn broadcast_output(&mut self, offset: u64, bytes: &[u8], grid: &[u8]) {
        self.subscribers.retain(|_, subscriber| {
            let sent = subscriber.send_output(offset, bytes).is_ok()
                && subscriber
                    .send_low(
                        FrameKind::Grid,
                        grid.to_vec(),
                        offset.saturating_add(bytes.len() as u64),
                    )
                    .is_ok();
            if !sent {
                subscriber.detached.store(1, Ordering::Release);
                subscriber.writer.reset_stream(subscriber.stream_id);
            }
            sent
        });
    }

    fn broadcast_grid(&mut self) -> Result<(), TerminalStreamError> {
        let grid = encode_full_grid(&self.screen, self.descriptor.cols, self.descriptor.rows)?;
        self.subscribers.retain(|_, subscriber| {
            let sent = subscriber
                .send_low(FrameKind::Grid, grid.clone(), subscriber.confirmed_offset)
                .is_ok();
            if !sent {
                subscriber.detached.store(1, Ordering::Release);
                subscriber.writer.reset_stream(subscriber.stream_id);
            }
            sent
        });
        Ok(())
    }
}

struct Subscriber {
    stream_id: u32,
    next_sequence: u64,
    confirmed_offset: u64,
    writer: WriterHandle,
    detached: Arc<AtomicU64>,
}

impl Subscriber {
    fn send_output(&mut self, offset: u64, bytes: &[u8]) -> Result<(), TerminalStreamError> {
        let mut payload = Vec::with_capacity(size_of::<u64>() + bytes.len());
        payload.extend_from_slice(&offset.to_be_bytes());
        payload.extend_from_slice(bytes);
        let Some(confirmed_offset) = offset.checked_add(bytes.len() as u64) else {
            self.detached.store(1, Ordering::Release);
            return Err(TerminalStreamError::SequenceOverflow);
        };
        self.send_low(FrameKind::Output, payload, confirmed_offset)
    }

    fn send_low(
        &mut self,
        kind: FrameKind,
        payload: Vec<u8>,
        confirmed_offset: u64,
    ) -> Result<(), TerminalStreamError> {
        let sequence = self.next_sequence;
        let Some(next_sequence) = sequence.checked_add(1) else {
            self.detached.store(1, Ordering::Release);
            return Err(TerminalStreamError::SequenceOverflow);
        };
        let result = self.writer.try_send_low(
            frame(kind, self.stream_id, sequence, payload),
            StreamPosition::terminal(self.confirmed_offset),
        );
        let enqueue = match result {
            Ok(enqueue) => enqueue,
            Err(error) => {
                self.detached.store(1, Ordering::Release);
                return Err(error.into());
            }
        };
        match enqueue {
            LowEnqueue::Queued => {
                self.next_sequence = next_sequence;
                self.confirmed_offset = confirmed_offset;
                Ok(())
            }
            LowEnqueue::StreamReset => {
                self.detached.store(1, Ordering::Release);
                Err(TerminalStreamError::SlowConsumer)
            }
        }
    }
}

fn frame(kind: FrameKind, stream_id: u32, sequence: u64, payload: Vec<u8>) -> Frame {
    Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind,
            flags: 0,
            stream_id,
            message_id: 0,
            sequence,
        },
        payload,
    }
}

fn encode_full_grid(
    screen: &HeadlessScreen,
    cols: u16,
    rows: u16,
) -> Result<Vec<u8>, TerminalStreamError> {
    let cell_count =
        terminal_cell_count(cols, rows).ok_or(TerminalStreamError::InvalidDescriptor)?;
    let capacity = cell_count
        .checked_mul(GridCell::WIRE_BYTES + 1)
        .and_then(|bytes| bytes.checked_add(11))
        .and_then(|bytes| {
            usize::from(rows)
                .checked_mul(4)
                .and_then(|row_bytes| bytes.checked_add(row_bytes))
        })
        .ok_or(TerminalStreamError::InvalidDescriptor)?;
    let visible_lines = screen.lines();
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(capacity)
        .map_err(|_| TerminalStreamError::Backpressure)?;
    payload.extend_from_slice(&cols.to_be_bytes());
    payload.extend_from_slice(&rows.to_be_bytes());
    payload.extend_from_slice(&0_u16.to_be_bytes());
    payload.extend_from_slice(&0_u16.to_be_bytes());
    payload.push(2);
    payload.extend_from_slice(&rows.to_be_bytes());

    for y in 0..rows {
        let mut cells = Vec::new();
        cells
            .try_reserve_exact(usize::from(cols))
            .map_err(|_| TerminalStreamError::Backpressure)?;
        if let Some(line) = visible_lines.get(usize::from(y)) {
            cells.extend(line.chars().take(usize::from(cols)).map(|ch| {
                GridCell::new(
                    ch as u32,
                    TermColor::Default,
                    TermColor::DefaultInverted,
                    TermStyle::empty(),
                )
            }));
        }
        cells.resize(usize::from(cols), GridCell::BLANK);
        let encoded = encode_row(&cells);
        payload.extend_from_slice(&y.to_be_bytes());
        payload.extend_from_slice(&cols.to_be_bytes());
        payload.extend_from_slice(&encoded.data[size_of::<u16>()..]);
    }
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;
    use std::time::Duration;

    use homie_proto::grid::{
        GridCell, MAX_TERMINAL_CELLS, MAX_TERMINAL_COLS, TermColor, TermStyle,
    };
    use homie_proto::stream::{StreamReset, StreamResetReason, TerminalStreamOpen};
    use homie_proto::transport::{
        ClientRole, EndpointRole, Frame, FrameHeader, FrameKind, MAX_OUTPUT_PAYLOAD, WIRE_MAJOR,
    };
    use tempfile::TempDir;
    use tokio::io::{AsyncRead, AsyncReadExt};
    use tokio::task::JoinHandle;

    use super::{
        RuntimeTerminalBackend, TerminalBackend, TerminalSourceDescriptor, TerminalSourceManager,
        TerminalSourceStats, TerminalStreamError,
    };
    use crate::dispatcher::RuntimeResponse;
    use crate::runtime_actor::{
        RuntimeActor, RuntimeBackend, RuntimeCall, RuntimeReply, ServiceError, ServiceResult,
    };
    use crate::writer::{LowEnqueue, StreamPosition, WriterDriver, WriterHandle, writer_channel};

    struct RecordingRuntimeBackend {
        calls: Arc<Mutex<Vec<RuntimeCall>>>,
        descriptor: TerminalSourceDescriptor,
    }

    impl RuntimeBackend for RecordingRuntimeBackend {
        fn call(&mut self, request: RuntimeCall) -> ServiceResult<RuntimeReply> {
            self.calls.lock().expect("calls").push(request.clone());
            match request {
                RuntimeCall::TerminalDescribe { .. } => {
                    Ok(RuntimeReply::TerminalDescriptor(self.descriptor.clone()))
                }
                RuntimeCall::TerminalInput { .. } | RuntimeCall::TerminalResize { .. } => {
                    Ok(RuntimeReply::Response(RuntimeResponse::Ack(
                        homie_proto::transport::AckResult { ok: true },
                    )))
                }
                _ => panic!("unexpected runtime call"),
            }
        }
    }

    struct ErrorRuntimeBackend(ServiceError);

    impl RuntimeBackend for ErrorRuntimeBackend {
        fn call(&mut self, _request: RuntimeCall) -> ServiceResult<RuntimeReply> {
            Err(self.0.clone())
        }
    }

    struct BlockingRuntimeBackend {
        gate: Arc<(Mutex<bool>, Condvar)>,
    }

    impl RuntimeBackend for BlockingRuntimeBackend {
        fn call(&mut self, _request: RuntimeCall) -> ServiceResult<RuntimeReply> {
            let (open, changed) = &*self.gate;
            let mut open = open.lock().expect("gate");
            while !*open {
                open = changed.wait(open).expect("gate wait");
            }
            Ok(RuntimeReply::Response(RuntimeResponse::Ack(
                homie_proto::transport::AckResult { ok: true },
            )))
        }
    }

    #[tokio::test]
    async fn runtime_terminal_backend_routes_descriptor_raw_input_and_resize_in_order() {
        let output_path = PathBuf::from("/tmp/runtime-terminal-output.log");
        let descriptor = TerminalSourceDescriptor {
            session_id: "session-1".to_string(),
            output_path: output_path.clone(),
            cols: 132,
            rows: 48,
            modes: Vec::new(),
        };
        let calls = Arc::new(Mutex::new(Vec::new()));
        let actor = RuntimeActor::spawn(RecordingRuntimeBackend {
            calls: calls.clone(),
            descriptor: descriptor.clone(),
        })
        .expect("actor");
        let backend = RuntimeTerminalBackend::new(actor.handle());
        let input = vec![0x00, 0xff, 0x80, b'\n'];

        let actual = backend.describe("session-1").await.expect("describe");
        backend
            .send_input("session-1", input.clone())
            .await
            .expect("input");
        backend.resize("session-1", 132, 48).await.expect("resize");

        assert_eq!(actual, descriptor);
        assert_eq!(
            calls.lock().expect("calls").as_slice(),
            &[
                RuntimeCall::TerminalDescribe {
                    session_id: "session-1".to_string(),
                },
                RuntimeCall::TerminalInput {
                    session_id: "session-1".to_string(),
                    bytes: input,
                },
                RuntimeCall::TerminalResize {
                    session_id: "session-1".to_string(),
                    cols: 132,
                    rows: 48,
                },
            ]
        );
        actor.shutdown_async().await.expect("shutdown");
    }

    #[tokio::test]
    async fn runtime_terminal_backend_maps_full_actor_queue_to_backpressure() {
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        let actor = RuntimeActor::spawn(BlockingRuntimeBackend { gate: gate.clone() })
            .expect("spawn actor");
        let handle = actor.handle();
        let _running = handle
            .try_call(RuntimeCall::TerminalInput {
                session_id: "session-1".to_string(),
                bytes: Vec::new(),
            })
            .expect("running");
        thread::sleep(Duration::from_millis(20));
        let _pending = (0..crate::runtime_actor::ACTOR_QUEUE_CAPACITY)
            .map(|_| {
                handle
                    .try_call(RuntimeCall::TerminalInput {
                        session_id: "session-1".to_string(),
                        bytes: Vec::new(),
                    })
                    .expect("pending")
            })
            .collect::<Vec<_>>();
        let backend = RuntimeTerminalBackend::new(handle);

        let error = backend
            .describe("session-1")
            .await
            .expect_err("queue should be full");

        assert!(matches!(error, TerminalStreamError::Backpressure));
        let (open, changed) = &*gate;
        *open.lock().expect("gate") = true;
        changed.notify_all();
        actor.shutdown_async().await.expect("shutdown");
    }

    #[tokio::test]
    async fn runtime_terminal_backend_maps_stopped_actor_to_source_unavailable() {
        let actor =
            RuntimeActor::spawn(ErrorRuntimeBackend(ServiceError::Internal)).expect("spawn actor");
        let backend = RuntimeTerminalBackend::new(actor.handle());
        actor.shutdown_async().await.expect("shutdown");

        let error = backend
            .describe("session-1")
            .await
            .expect_err("actor should be stopped");

        assert!(matches!(error, TerminalStreamError::SourceUnavailable));
    }

    #[tokio::test]
    async fn runtime_terminal_backend_maps_actor_reply_error_to_backend() {
        let actor =
            RuntimeActor::spawn(ErrorRuntimeBackend(ServiceError::Internal)).expect("spawn actor");
        let backend = RuntimeTerminalBackend::new(actor.handle());

        let error = backend
            .send_input("session-1", vec![0xff])
            .await
            .expect_err("backend should fail");

        assert!(matches!(error, TerminalStreamError::Backend));
        actor.shutdown_async().await.expect("shutdown");
    }

    #[derive(Default)]
    struct FakeBackend {
        descriptors: Mutex<HashMap<String, TerminalSourceDescriptor>>,
        describe_calls: AtomicUsize,
        inputs: Mutex<Vec<(String, Vec<u8>)>>,
        resizes: Mutex<Vec<(String, u16, u16)>>,
    }

    impl FakeBackend {
        fn insert(&self, descriptor: TerminalSourceDescriptor) {
            self.descriptors
                .lock()
                .expect("descriptors")
                .insert(descriptor.session_id.clone(), descriptor);
        }
    }

    impl TerminalBackend for FakeBackend {
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

    struct TestConnection {
        writer: Option<WriterHandle>,
        peer: tokio::io::DuplexStream,
        task: JoinHandle<Result<(), crate::writer::WriterError>>,
    }

    impl TestConnection {
        fn new() -> Self {
            let (writer, driver) = writer_channel();
            let (socket, peer) = tokio::io::duplex(4 * 1024 * 1024);
            let task = tokio::spawn(run_writer(driver, socket));
            Self {
                writer: Some(writer),
                peer,
                task,
            }
        }

        fn take_writer(&mut self) -> WriterHandle {
            self.writer.take().expect("writer")
        }

        async fn frame(&mut self) -> Frame {
            tokio::time::timeout(Duration::from_secs(2), read_frame(&mut self.peer))
                .await
                .expect("frame timeout")
        }
    }

    impl Drop for TestConnection {
        fn drop(&mut self) {
            self.task.abort();
        }
    }

    async fn run_writer(
        driver: WriterDriver,
        socket: tokio::io::DuplexStream,
    ) -> Result<(), crate::writer::WriterError> {
        driver.run(socket).await
    }

    struct Fixture {
        _temp_dir: TempDir,
        backend: Arc<FakeBackend>,
        manager: TerminalSourceManager<FakeBackend>,
        output_path: PathBuf,
    }

    impl Fixture {
        fn new(session_id: &str, output: &[u8], cols: u16, rows: u16, modes: Vec<u8>) -> Self {
            let temp_dir = tempfile::tempdir().expect("temp dir");
            let output_path = temp_dir.path().join("output.log");
            std::fs::write(&output_path, output).expect("output fixture");
            let backend = Arc::new(FakeBackend::default());
            backend.insert(TerminalSourceDescriptor {
                session_id: session_id.to_string(),
                output_path: output_path.clone(),
                cols,
                rows,
                modes,
            });
            let manager = TerminalSourceManager::new(backend.clone());
            Self {
                _temp_dir: temp_dir,
                backend,
                manager,
                output_path,
            }
        }

        async fn open(
            &self,
            stream_id: u32,
            offset: u64,
            connection: &mut TestConnection,
        ) -> super::TerminalSubscription {
            self.open_session("session-1", stream_id, offset, connection)
                .await
        }

        async fn open_session(
            &self,
            session_id: &str,
            stream_id: u32,
            offset: u64,
            connection: &mut TestConnection,
        ) -> super::TerminalSubscription {
            self.manager
                .open(
                    stream_id,
                    TerminalStreamOpen {
                        session_id: session_id.to_string(),
                        output_offset: offset,
                        client_role: ClientRole::App,
                        last_grid_sequence: None,
                    },
                    connection.take_writer(),
                )
                .await
                .expect("open terminal stream")
        }
    }

    #[tokio::test]
    async fn stream_orders_open_replay_grid_modes_then_live_with_contiguous_sequences() {
        let fixture = Fixture::new("session-1", b"hi", 4, 2, vec![0x11, 0x22]);
        let mut connection = TestConnection::new();
        let _subscription = fixture.open(7, 0, &mut connection).await;

        let mut initial = Vec::new();
        for _ in 0..6 {
            initial.push(connection.frame().await);
        }
        assert_eq!(
            initial
                .iter()
                .map(|frame| frame.header.kind)
                .collect::<Vec<_>>(),
            vec![
                FrameKind::StreamOpened,
                FrameKind::ReplayBegin,
                FrameKind::Output,
                FrameKind::ReplayEnd,
                FrameKind::Grid,
                FrameKind::Modes,
            ]
        );
        assert_eq!(
            initial
                .iter()
                .map(|frame| frame.header.sequence)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 5]
        );
        assert_eq!(read_offset(&initial[1].payload), 0);
        assert_eq!(read_offset(&initial[2].payload), 0);
        assert_eq!(&initial[2].payload[8..], b"hi");
        assert_eq!(read_offset(&initial[3].payload), 2);
        assert_eq!(decode_grid(&initial[4].payload), expected_grid());
        assert_eq!(initial[5].payload, vec![0x11, 0x22]);

        append(&fixture.output_path, b"!");
        let output = connection.frame().await;
        let grid = connection.frame().await;

        assert_eq!(output.header.kind, FrameKind::Output);
        assert_eq!(output.header.sequence, 6);
        assert_eq!(read_offset(&output.payload), 2);
        assert_eq!(&output.payload[8..], b"!");
        assert_eq!(grid.header.kind, FrameKind::Grid);
        assert_eq!(grid.header.sequence, 7);
    }

    #[tokio::test]
    async fn output_chunks_include_absolute_offsets_and_never_exceed_64_kib_payload() {
        let data_len = (MAX_OUTPUT_PAYLOAD - 8) * 2 + 3;
        let fixture = Fixture::new("session-1", &vec![b'x'; data_len], 80, 24, Vec::new());
        let mut connection = TestConnection::new();
        let _subscription = fixture.open(1, 0, &mut connection).await;

        let opened = connection.frame().await;
        let begin = connection.frame().await;
        let first = connection.frame().await;
        let second = connection.frame().await;
        let third = connection.frame().await;

        assert_eq!(opened.header.kind, FrameKind::StreamOpened);
        assert_eq!(begin.header.kind, FrameKind::ReplayBegin);
        assert_eq!(first.payload.len(), MAX_OUTPUT_PAYLOAD);
        assert_eq!(second.payload.len(), MAX_OUTPUT_PAYLOAD);
        assert_eq!(third.payload.len(), 11);
        assert_eq!(read_offset(&first.payload), 0);
        assert_eq!(
            read_offset(&second.payload),
            (MAX_OUTPUT_PAYLOAD - 8) as u64
        );
        assert_eq!(
            read_offset(&third.payload),
            ((MAX_OUTPUT_PAYLOAD - 8) * 2) as u64
        );
    }

    #[tokio::test]
    async fn same_session_and_stream_id_use_one_source_reader_with_distinct_subscribers() {
        let fixture = Fixture::new("session-1", b"a", 4, 2, Vec::new());
        let mut first = TestConnection::new();
        let mut second = TestConnection::new();
        let _first_subscription = fixture.open(1, 0, &mut first).await;
        let _second_subscription = fixture.open(1, 0, &mut second).await;

        for _ in 0..6 {
            first.frame().await;
            second.frame().await;
        }
        append(&fixture.output_path, b"b");
        let first_live = first.frame().await;
        let second_live = second.frame().await;
        let stats = fixture.manager.stats().await;

        assert_eq!(fixture.backend.describe_calls.load(Ordering::Relaxed), 1);
        assert_eq!(stats.source_count, 1);
        assert_eq!(stats.output_log_readers, 1);
        assert_eq!(first_live.header.kind, FrameKind::Output);
        assert_eq!(second_live.header.kind, FrameKind::Output);
        assert_eq!(first_live.header.stream_id, 1);
        assert_eq!(second_live.header.stream_id, 1);
        assert_eq!(read_offset(&first_live.payload), 1);
        assert_eq!(read_offset(&second_live.payload), 1);
    }

    #[tokio::test]
    async fn different_session_sources_never_cross_deliver_output() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let first_path = temp_dir.path().join("first.log");
        let second_path = temp_dir.path().join("second.log");
        std::fs::write(&first_path, []).expect("first output");
        std::fs::write(&second_path, []).expect("second output");
        let backend = Arc::new(FakeBackend::default());
        backend.insert(TerminalSourceDescriptor {
            session_id: "session-1".to_string(),
            output_path: first_path.clone(),
            cols: 4,
            rows: 2,
            modes: Vec::new(),
        });
        backend.insert(TerminalSourceDescriptor {
            session_id: "session-2".to_string(),
            output_path: second_path.clone(),
            cols: 4,
            rows: 2,
            modes: Vec::new(),
        });
        let manager = TerminalSourceManager::new(backend);
        let mut first = TestConnection::new();
        let mut second = TestConnection::new();
        let _first_subscription = manager
            .open(1, terminal_request("session-1"), first.take_writer())
            .await
            .expect("first stream");
        let _second_subscription = manager
            .open(1, terminal_request("session-2"), second.take_writer())
            .await
            .expect("second stream");
        for _ in 0..5 {
            first.frame().await;
            second.frame().await;
        }

        append(&first_path, b"first");
        let first_output = first.frame().await;
        let first_grid = first.frame().await;
        let second_result =
            tokio::time::timeout(Duration::from_millis(400), read_frame(&mut second.peer)).await;

        assert_eq!(first_output.header.kind, FrameKind::Output);
        assert_eq!(&first_output.payload[8..], b"first");
        assert_eq!(first_grid.header.kind, FrameKind::Grid);
        assert!(
            second_result.is_err(),
            "second session received first output"
        );

        append(&second_path, b"second");
        let second_output = second.frame().await;
        assert_eq!(second_output.header.kind, FrameKind::Output);
        assert_eq!(&second_output.payload[8..], b"second");
    }

    #[tokio::test]
    async fn input_is_forwarded_raw_and_resize_is_decoded_big_endian() {
        let fixture = Fixture::new("session-1", &[], 4, 2, Vec::new());
        let mut connection = TestConnection::new();
        let subscription = fixture.open(1, 0, &mut connection).await;
        drain_initial(&mut connection, 5).await;

        subscription
            .handle_frame(client_frame(FrameKind::Input, 1, vec![0, 0xff, b'\n']))
            .await
            .expect("input");
        subscription
            .handle_frame(client_frame(
                FrameKind::Resize,
                1,
                [120_u16.to_be_bytes(), 40_u16.to_be_bytes()].concat(),
            ))
            .await
            .expect("resize");

        assert_eq!(
            fixture.backend.inputs.lock().expect("inputs").as_slice(),
            &[("session-1".to_string(), vec![0, 0xff, b'\n'])]
        );
        assert_eq!(
            fixture.backend.resizes.lock().expect("resizes").as_slice(),
            &[("session-1".to_string(), 120, 40)]
        );
        let resized_grid = connection.frame().await;
        assert_eq!(resized_grid.header.kind, FrameKind::Grid);
        assert_eq!(resized_grid.header.sequence, 5);
        assert_eq!(&resized_grid.payload[..4], &[0, 120, 0, 40]);
    }

    #[tokio::test]
    async fn source_rejects_descriptor_geometry_over_shared_axis_limit() {
        let fixture = Fixture::new("session-1", &[], MAX_TERMINAL_COLS + 1, 1, Vec::new());
        let mut connection = TestConnection::new();

        let result =
            fixture
                .manager
                .open(1, terminal_request("session-1"), connection.take_writer());
        let error = match result.await {
            Ok(_) => panic!("oversized descriptor unexpectedly opened"),
            Err(error) => error,
        };

        assert!(matches!(error, TerminalStreamError::InvalidDescriptor));
    }

    #[tokio::test]
    async fn resize_rejects_geometry_over_shared_cell_limit_before_backend_call() {
        let fixture = Fixture::new("session-1", &[], 4, 2, Vec::new());
        let mut connection = TestConnection::new();
        let subscription = fixture.open(1, 0, &mut connection).await;
        drain_initial(&mut connection, 5).await;
        let rows = u16::try_from(MAX_TERMINAL_CELLS / usize::from(MAX_TERMINAL_COLS) + 1)
            .expect("test rows fit u16");

        let error = subscription
            .handle_frame(client_frame(
                FrameKind::Resize,
                1,
                [MAX_TERMINAL_COLS.to_be_bytes(), rows.to_be_bytes()].concat(),
            ))
            .await
            .expect_err("oversized resize must fail");

        assert!(matches!(error, TerminalStreamError::InvalidFrame));
        assert!(fixture.backend.resizes.lock().expect("resizes").is_empty());
    }

    #[tokio::test]
    async fn input_commands_do_not_postpone_idle_tail_reads() {
        let fixture = Fixture::new("session-1", &[], 4, 2, Vec::new());
        let mut connection = TestConnection::new();
        let subscription = fixture.open(1, 0, &mut connection).await;
        drain_initial(&mut connection, 5).await;
        append(&fixture.output_path, b"ready");

        for _ in 0..4 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            subscription
                .handle_frame(client_frame(FrameKind::Input, 1, b"x".to_vec()))
                .await
                .expect("input");
        }
        let output =
            tokio::time::timeout(Duration::from_millis(100), read_frame(&mut connection.peer))
                .await
                .expect("idle tail was postponed by commands");

        assert_eq!(output.header.kind, FrameKind::Output);
        assert_eq!(&output.payload[8..], b"ready");
    }

    #[tokio::test]
    async fn stream_close_releases_source_without_terminating_session_and_reopen_creates_source() {
        let fixture = Fixture::new("session-1", &[], 4, 2, Vec::new());
        let mut first = TestConnection::new();
        let first_subscription = fixture.open(1, 0, &mut first).await;
        drain_initial(&mut first, 5).await;

        first_subscription
            .handle_frame(client_frame(FrameKind::StreamClose, 1, Vec::new()))
            .await
            .expect("stream close");
        let after_close = wait_for_stats(&fixture.manager, TerminalSourceStats::default()).await;

        assert_eq!(after_close, TerminalSourceStats::default());

        let mut second = TestConnection::new();
        let _second_subscription = fixture.open(1, 0, &mut second).await;
        drain_initial(&mut second, 5).await;
        let reopened = fixture.manager.stats().await;

        assert_eq!(fixture.backend.describe_calls.load(Ordering::Relaxed), 2);
        assert_eq!(reopened.source_count, 1);
        assert_eq!(reopened.output_log_readers, 1);
        assert_eq!(reopened.subscriber_count, 1);
    }

    #[tokio::test]
    async fn dropping_last_subscription_eventually_releases_source_reader() {
        let fixture = Fixture::new("session-1", &[], 4, 2, Vec::new());
        let mut connection = TestConnection::new();
        let subscription = fixture.open(1, 0, &mut connection).await;
        drain_initial(&mut connection, 5).await;

        drop(subscription);
        let stats = wait_for_stats(&fixture.manager, TerminalSourceStats::default()).await;

        assert_eq!(stats, TerminalSourceStats::default());
    }

    #[tokio::test]
    async fn writer_failure_marks_subscription_finished_before_source_removes_it() {
        let fixture = Fixture::new("session-1", &[], 4, 2, Vec::new());
        let mut connection = TestConnection::new();
        let writer = connection.writer.as_ref().expect("writer").clone();
        let subscription = fixture.open(1, 0, &mut connection).await;
        drain_initial(&mut connection, 5).await;

        writer.close();
        append(&fixture.output_path, b"closed");
        tokio::time::timeout(Duration::from_secs(2), async {
            while !subscription.is_finished() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("subscription finish timeout");
        let stats = wait_for_stats(&fixture.manager, TerminalSourceStats::default()).await;

        assert!(subscription.is_finished());
        assert_eq!(stats, TerminalSourceStats::default());
    }

    #[tokio::test]
    async fn detached_flag_cleans_lost_drop_notification_while_other_subscriber_remains() {
        let fixture = Fixture::new("session-1", &[], 4, 2, Vec::new());
        let mut first = TestConnection::new();
        let mut second = TestConnection::new();
        let first_subscription = fixture.open(1, 0, &mut first).await;
        let _second_subscription = fixture.open(2, 0, &mut second).await;
        drain_initial(&mut first, 5).await;
        drain_initial(&mut second, 5).await;

        first_subscription.detached.store(1, Ordering::Release);
        drop(first_subscription);
        let stats = wait_for_stats(
            &fixture.manager,
            TerminalSourceStats {
                source_count: 1,
                output_log_readers: 1,
                subscriber_count: 1,
            },
        )
        .await;

        assert_eq!(stats.subscriber_count, 1);
    }

    #[tokio::test]
    async fn slow_writer_resets_only_its_subscriber_and_healthy_stream_keeps_live_output() {
        let fixture = Fixture::new("session-1", &[], 4, 2, Vec::new());
        let (slow_writer, slow_driver) = writer_channel();
        for sequence in 1..=crate::writer::LOW_QUEUE_CAPACITY {
            assert_eq!(
                slow_writer
                    .try_send_low(
                        super::frame(FrameKind::Grid, 1, sequence as u64, Vec::new()),
                        StreamPosition::terminal(0),
                    )
                    .expect("prefill slow queue"),
                LowEnqueue::Queued
            );
        }

        let error = match fixture
            .manager
            .open(1, terminal_request("session-1"), slow_writer)
            .await
        {
            Ok(_) => panic!("slow subscriber unexpectedly opened"),
            Err(error) => error,
        };
        assert!(matches!(error, TerminalStreamError::SlowConsumer));

        let (slow_socket, mut slow_peer) = tokio::io::duplex(64 * 1024);
        let slow_task = tokio::spawn(run_writer(slow_driver, slow_socket));
        let opened = read_frame(&mut slow_peer).await;
        let reset = read_frame(&mut slow_peer).await;
        let reset_payload: StreamReset =
            serde_json::from_slice(&reset.payload).expect("reset payload");
        assert_eq!(opened.header.kind, FrameKind::StreamOpened);
        assert_eq!(reset.header.kind, FrameKind::StreamReset);
        assert_eq!(reset_payload.reason, StreamResetReason::SlowConsumer);

        let mut healthy = TestConnection::new();
        let _healthy_subscription = fixture.open(1, 0, &mut healthy).await;
        drain_initial(&mut healthy, 5).await;
        append(&fixture.output_path, b"healthy");
        let output = healthy.frame().await;

        assert_eq!(output.header.kind, FrameKind::Output);
        assert_eq!(&output.payload[8..], b"healthy");
        assert_eq!(fixture.manager.stats().await.subscriber_count, 1);
        slow_task.abort();
    }

    fn terminal_request(session_id: &str) -> TerminalStreamOpen {
        TerminalStreamOpen {
            session_id: session_id.to_string(),
            output_offset: 0,
            client_role: ClientRole::App,
            last_grid_sequence: None,
        }
    }

    async fn drain_initial(connection: &mut TestConnection, frames: usize) {
        for _ in 0..frames {
            connection.frame().await;
        }
    }

    async fn wait_for_stats(
        manager: &TerminalSourceManager<FakeBackend>,
        expected: TerminalSourceStats,
    ) -> TerminalSourceStats {
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                let stats = manager.stats().await;
                if stats == expected {
                    return stats;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("terminal source stats timeout")
    }

    fn client_frame(kind: FrameKind, stream_id: u32, payload: Vec<u8>) -> Frame {
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

    fn append(path: &PathBuf, bytes: &[u8]) {
        use std::io::Write;

        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(path)
            .expect("append output");
        file.write_all(bytes).expect("write output");
        file.flush().expect("flush output");
    }

    async fn read_frame(reader: &mut (impl AsyncRead + Unpin)) -> Frame {
        let mut length = [0_u8; 4];
        reader.read_exact(&mut length).await.expect("frame length");
        let frame_len = u32::from_be_bytes(length) as usize;
        let mut encoded = vec![0_u8; 4 + frame_len];
        encoded[..4].copy_from_slice(&length);
        reader
            .read_exact(&mut encoded[4..])
            .await
            .expect("frame body");
        Frame::decode(&encoded, EndpointRole::Server)
            .expect("decode frame")
            .expect("complete frame")
            .0
    }

    fn read_offset(payload: &[u8]) -> u64 {
        u64::from_be_bytes(payload[..8].try_into().expect("offset"))
    }

    #[derive(Debug, Eq, PartialEq)]
    struct DecodedGrid {
        cols: u16,
        rows: u16,
        cursor_col: u16,
        cursor_row: u16,
        cursor_visible: bool,
        full: bool,
        changed_rows: Vec<(u16, Vec<GridCell>)>,
    }

    fn decode_grid(payload: &[u8]) -> DecodedGrid {
        let mut decoder = GridDecoder { payload, offset: 0 };
        let cols = decoder.u16();
        let rows = decoder.u16();
        let cursor_col = decoder.u16();
        let cursor_row = decoder.u16();
        let flags = decoder.u8();
        let row_count = decoder.u16();
        let mut changed_rows = Vec::new();
        for _ in 0..row_count {
            let y = decoder.u16();
            let row_cols = decoder.u16();
            let mut cells = Vec::new();
            while cells.len() < usize::from(row_cols) {
                let run = decoder.u8();
                let cell = GridCell::new(
                    decoder.u32(),
                    TermColor::unpack(decoder.u32()),
                    TermColor::unpack(decoder.u32()),
                    TermStyle::from_bits_retain(decoder.u16()),
                );
                cells.extend(std::iter::repeat_n(cell, usize::from(run)));
            }
            changed_rows.push((y, cells));
        }
        assert_eq!(decoder.offset, payload.len());
        DecodedGrid {
            cols,
            rows,
            cursor_col,
            cursor_row,
            cursor_visible: flags & 1 != 0,
            full: flags & 2 != 0,
            changed_rows,
        }
    }

    fn expected_grid() -> DecodedGrid {
        let text = GridCell::new(
            'h' as u32,
            TermColor::Default,
            TermColor::DefaultInverted,
            TermStyle::empty(),
        );
        let second = GridCell::new(
            'i' as u32,
            TermColor::Default,
            TermColor::DefaultInverted,
            TermStyle::empty(),
        );
        DecodedGrid {
            cols: 4,
            rows: 2,
            cursor_col: 0,
            cursor_row: 0,
            cursor_visible: false,
            full: true,
            changed_rows: vec![
                (0, vec![text, second, GridCell::BLANK, GridCell::BLANK]),
                (1, vec![GridCell::BLANK; 4]),
            ],
        }
    }

    struct GridDecoder<'a> {
        payload: &'a [u8],
        offset: usize,
    }

    impl GridDecoder<'_> {
        fn u8(&mut self) -> u8 {
            let value = self.payload[self.offset];
            self.offset += 1;
            value
        }

        fn u16(&mut self) -> u16 {
            u16::from_be_bytes(self.take())
        }

        fn u32(&mut self) -> u32 {
            u32::from_be_bytes(self.take())
        }

        fn take<const N: usize>(&mut self) -> [u8; N] {
            let end = self.offset + N;
            let value = self.payload[self.offset..end]
                .try_into()
                .expect("grid field");
            self.offset = end;
            value
        }
    }
}
