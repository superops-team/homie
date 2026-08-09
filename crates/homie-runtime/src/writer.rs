use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use homie_proto::stream::{StreamReset, StreamResetReason};
use homie_proto::transport::{EndpointRole, Frame, FrameHeader, FrameKind, WIRE_MAJOR};
use thiserror::Error;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::{Notify, mpsc, oneshot};

pub const HIGH_QUEUE_CAPACITY: usize = 256;
pub const LOW_QUEUE_CAPACITY: usize = 256;
pub const HIGH_BURST_QUOTA: usize = 32;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StreamPosition {
    pub last_confirmed_offset: Option<u64>,
    pub latest_seq: Option<u64>,
}

impl StreamPosition {
    #[must_use]
    pub const fn terminal(last_confirmed_offset: u64) -> Self {
        Self {
            last_confirmed_offset: Some(last_confirmed_offset),
            latest_seq: None,
        }
    }

    #[must_use]
    pub const fn event(latest_seq: u64) -> Self {
        Self {
            last_confirmed_offset: None,
            latest_seq: Some(latest_seq),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LowEnqueue {
    Queued,
    StreamReset,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum WriterError {
    #[error("high-priority writer queue is full")]
    HighQueueFull,
    #[error("writer connection is closed")]
    ConnectionClosed,
    #[error("writer received an invalid frame")]
    InvalidFrame,
    #[error("socket write failed")]
    SocketWrite,
}

#[derive(Clone)]
pub struct WriterHandle {
    high: mpsc::Sender<HighQueueItem>,
    shared: Arc<WriterShared>,
}

impl WriterHandle {
    pub fn try_send_high(&self, frame: Frame) -> Result<(), WriterError> {
        self.try_send_high_item(HighQueueItem::Frame(frame))
    }

    pub async fn flush(&self) -> Result<(), WriterError> {
        let (reply, waiter) = oneshot::channel();
        self.try_send_high_item(HighQueueItem::Flush(reply))
            .map_err(|error| {
                if error == WriterError::ConnectionClosed {
                    self.shared.terminal_error()
                } else {
                    error
                }
            })?;
        let closed = self.shared.closed_notify.notified();
        tokio::pin!(closed);
        if self.is_closed() {
            return Err(self.shared.terminal_error());
        }
        tokio::select! {
            biased;
            result = waiter => {
                result.unwrap_or_else(|_| Err(self.shared.terminal_error()))
            }
            _ = &mut closed => Err(self.shared.terminal_error()),
        }
    }

    fn try_send_high_item(&self, item: HighQueueItem) -> Result<(), WriterError> {
        if self.is_closed() {
            return Err(WriterError::ConnectionClosed);
        }
        match self.high.try_send(item) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.shared.close_with(WriterError::HighQueueFull);
                Err(WriterError::HighQueueFull)
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                self.shared.close();
                Err(WriterError::ConnectionClosed)
            }
        }
    }

    pub fn try_send_low(
        &self,
        frame: Frame,
        position: StreamPosition,
    ) -> Result<LowEnqueue, WriterError> {
        if self.is_closed() {
            return Err(WriterError::ConnectionClosed);
        }
        let stream_id = frame.header.stream_id;
        let reset_reason = if frame.header.kind == FrameKind::Event {
            StreamResetReason::EventGap
        } else {
            StreamResetReason::SlowConsumer
        };
        let enqueue = self
            .shared
            .low
            .lock()
            .map_err(|_| {
                self.shared.close();
                WriterError::ConnectionClosed
            })?
            .enqueue(frame, position);
        let LowQueueEnqueue::StreamReset(position) = enqueue else {
            self.shared.low_available.notify_one();
            return Ok(LowEnqueue::Queued);
        };

        let payload = serde_json::to_vec(&StreamReset {
            reason: reset_reason,
            last_confirmed_offset: position.last_confirmed_offset,
            latest_seq: position.latest_seq,
        })
        .map_err(|_| WriterError::InvalidFrame)?;
        self.try_send_high(Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind: FrameKind::StreamReset,
                flags: 0,
                stream_id,
                message_id: 0,
                sequence: 0,
            },
            payload,
        })?;
        Ok(LowEnqueue::StreamReset)
    }

    pub fn reset_stream(&self, stream_id: u32) {
        match self.shared.low.lock() {
            Ok(mut low) => low.remove(stream_id),
            Err(_) => self.shared.close(),
        }
    }

    pub fn close(&self) {
        self.shared.close_with(WriterError::ConnectionClosed);
    }

    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.shared.closed.load(Ordering::Acquire)
    }

    pub async fn closed(&self) {
        loop {
            let notified = self.shared.closed_notify.notified();
            if self.is_closed() {
                return;
            }
            notified.await;
        }
    }
}

enum HighQueueItem {
    Frame(Frame),
    Flush(oneshot::Sender<Result<(), WriterError>>),
}

pub(crate) struct WriterDriver {
    high: mpsc::Receiver<HighQueueItem>,
    shared: Arc<WriterShared>,
}

impl WriterDriver {
    pub(crate) async fn run(
        mut self,
        mut socket: impl AsyncWrite + Unpin,
    ) -> Result<(), WriterError> {
        let result = self.run_inner(&mut socket).await;
        if let Err(error) = result {
            self.shared.close_with(error);
        }
        result
    }

    async fn run_inner(
        &mut self,
        socket: &mut (impl AsyncWrite + Unpin),
    ) -> Result<(), WriterError> {
        let mut high_streak = 0_usize;
        loop {
            if self.shared.is_closed() {
                return Err(WriterError::ConnectionClosed);
            }
            if high_streak >= HIGH_BURST_QUOTA
                && let Some(item) = self.pop_low()?
            {
                self.write_low_frame(socket, item).await?;
                high_streak = 0;
                continue;
            }
            match self.high.try_recv() {
                Ok(item) => {
                    if self.write_high_item(socket, item).await? {
                        high_streak += 1;
                    }
                    continue;
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    if let Some(item) = self.pop_low()? {
                        self.write_low_frame(socket, item).await?;
                        high_streak = 0;
                        continue;
                    }
                    return Ok(());
                }
                Err(mpsc::error::TryRecvError::Empty) => {}
            }
            if let Some(item) = self.pop_low()? {
                self.write_low_frame(socket, item).await?;
                high_streak = 0;
                continue;
            }

            let closed = self.shared.closed_notify.notified();
            tokio::pin!(closed);
            if self.shared.is_closed() {
                return Err(WriterError::ConnectionClosed);
            }
            tokio::select! {
                biased;
                _ = &mut closed => {
                    if self.shared.is_closed() {
                        return Err(WriterError::ConnectionClosed);
                    }
                }
                item = self.high.recv() => {
                    let Some(item) = item else {
                        return Ok(());
                    };
                    if self.write_high_item(socket, item).await? {
                        high_streak += 1;
                    }
                }
                _ = self.shared.low_available.notified() => {}
            }
        }
    }

    async fn write_high_item(
        &self,
        socket: &mut (impl AsyncWrite + Unpin),
        item: HighQueueItem,
    ) -> Result<bool, WriterError> {
        match item {
            HighQueueItem::Frame(frame) => {
                self.write_frame(socket, frame).await?;
                Ok(true)
            }
            HighQueueItem::Flush(reply) => {
                let result = self.flush_socket(socket).await;
                let _ = reply.send(result);
                result.map(|()| false)
            }
        }
    }

    fn pop_low(&self) -> Result<Option<LowQueueItem>, WriterError> {
        self.shared
            .low
            .lock()
            .map_err(|_| WriterError::ConnectionClosed)
            .map(|mut low| low.pop())
    }

    async fn write_low_frame(
        &self,
        socket: &mut (impl AsyncWrite + Unpin),
        item: LowQueueItem,
    ) -> Result<(), WriterError> {
        let stream_id = item.frame.header.stream_id;
        self.write_frame(socket, item.frame).await?;
        self.shared
            .low
            .lock()
            .map_err(|_| WriterError::ConnectionClosed)?
            .mark_delivered(stream_id, item.post_write_position);
        Ok(())
    }

    async fn write_frame(
        &self,
        socket: &mut (impl AsyncWrite + Unpin),
        frame: Frame,
    ) -> Result<(), WriterError> {
        let encoded = match frame.encode(EndpointRole::Server) {
            Ok(encoded) => encoded,
            Err(_) => {
                self.shared.close_with(WriterError::InvalidFrame);
                return Err(WriterError::InvalidFrame);
            }
        };
        let closed = self.shared.closed_notify.notified();
        tokio::pin!(closed);
        if self.shared.is_closed() {
            return Err(WriterError::ConnectionClosed);
        }
        tokio::select! {
            biased;
            _ = &mut closed => {
                Err(WriterError::ConnectionClosed)
            }
            result = socket.write_all(&encoded) => {
                if result.is_err() {
                    self.shared.close_with(WriterError::SocketWrite);
                    return Err(WriterError::SocketWrite);
                }
                Ok(())
            }
        }
    }

    async fn flush_socket(
        &self,
        socket: &mut (impl AsyncWrite + Unpin),
    ) -> Result<(), WriterError> {
        let closed = self.shared.closed_notify.notified();
        tokio::pin!(closed);
        if self.shared.is_closed() {
            return Err(WriterError::ConnectionClosed);
        }
        tokio::select! {
            biased;
            _ = &mut closed => {
                Err(WriterError::ConnectionClosed)
            }
            result = socket.flush() => {
                result.map_err(|_| WriterError::SocketWrite)
            }
        }
    }
}

pub(crate) fn writer_channel() -> (WriterHandle, WriterDriver) {
    let (high_sender, high_receiver) = mpsc::channel(HIGH_QUEUE_CAPACITY);
    let shared = Arc::new(WriterShared::default());
    (
        WriterHandle {
            high: high_sender,
            shared: shared.clone(),
        },
        WriterDriver {
            high: high_receiver,
            shared,
        },
    )
}

#[derive(Default)]
struct WriterShared {
    low: Mutex<LowQueues>,
    low_available: Notify,
    closed_notify: Notify,
    closed: AtomicBool,
    terminal_error: Mutex<Option<WriterError>>,
}

impl WriterShared {
    fn close(&self) {
        self.close_with(WriterError::ConnectionClosed);
    }

    fn close_with(&self, error: WriterError) {
        if let Ok(mut terminal_error) = self.terminal_error.lock()
            && terminal_error.is_none()
        {
            *terminal_error = Some(error);
        }
        if !self.closed.swap(true, Ordering::AcqRel) {
            self.closed_notify.notify_waiters();
            self.low_available.notify_waiters();
        }
    }

    fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }

    fn terminal_error(&self) -> WriterError {
        self.terminal_error
            .lock()
            .ok()
            .and_then(|error| *error)
            .unwrap_or(WriterError::ConnectionClosed)
    }
}

enum LowQueueEnqueue {
    Queued,
    StreamReset(StreamPosition),
}

struct LowQueueItem {
    frame: Frame,
    post_write_position: StreamPosition,
}

impl LowQueueItem {
    fn new(frame: Frame, position: StreamPosition) -> Self {
        let post_write_position = post_write_position(&frame, position);
        Self {
            frame,
            post_write_position,
        }
    }
}

#[derive(Default)]
struct LowQueues {
    queues: HashMap<u32, VecDeque<LowQueueItem>>,
    round_robin: VecDeque<u32>,
    delivered: HashMap<u32, StreamPosition>,
}

impl LowQueues {
    fn enqueue(&mut self, frame: Frame, position: StreamPosition) -> LowQueueEnqueue {
        let stream_id = frame.header.stream_id;
        let initial_position = initial_stream_position(&frame, position);
        if self
            .queues
            .get(&stream_id)
            .is_some_and(|queue| queue.len() == LOW_QUEUE_CAPACITY)
        {
            let last_delivered = self
                .delivered
                .get(&stream_id)
                .copied()
                .unwrap_or(initial_position);
            self.remove(stream_id);
            return LowQueueEnqueue::StreamReset(last_delivered);
        }
        self.delivered.entry(stream_id).or_insert(initial_position);
        let queue = self.queues.entry(stream_id).or_default();
        if queue.is_empty() {
            self.round_robin.push_back(stream_id);
        }
        queue.push_back(LowQueueItem::new(frame, position));
        LowQueueEnqueue::Queued
    }

    fn pop(&mut self) -> Option<LowQueueItem> {
        let stream_id = self.round_robin.pop_front()?;
        let queue = self.queues.get_mut(&stream_id)?;
        let item = queue.pop_front();
        if queue.is_empty() {
            self.queues.remove(&stream_id);
        } else {
            self.round_robin.push_back(stream_id);
        }
        item
    }

    fn mark_delivered(&mut self, stream_id: u32, position: StreamPosition) {
        if let Some(delivered) = self.delivered.get_mut(&stream_id) {
            *delivered = position;
        }
    }

    fn remove(&mut self, stream_id: u32) {
        self.queues.remove(&stream_id);
        self.round_robin
            .retain(|queued_stream_id| *queued_stream_id != stream_id);
        self.delivered.remove(&stream_id);
    }
}

fn initial_stream_position(frame: &Frame, fallback: StreamPosition) -> StreamPosition {
    match frame.header.kind {
        FrameKind::Event => StreamPosition::event(frame.header.sequence.saturating_sub(1)),
        FrameKind::Output | FrameKind::ReplayBegin => frame_offset(frame)
            .map(StreamPosition::terminal)
            .unwrap_or_default(),
        _ => fallback,
    }
}

fn post_write_position(frame: &Frame, fallback: StreamPosition) -> StreamPosition {
    match frame.header.kind {
        FrameKind::Event => StreamPosition::event(frame.header.sequence),
        FrameKind::Output => frame_offset(frame)
            .and_then(|offset| {
                let output_len =
                    u64::try_from(frame.payload.len().checked_sub(size_of::<u64>())?).ok()?;
                offset.checked_add(output_len)
            })
            .map(StreamPosition::terminal)
            .unwrap_or_else(|| initial_stream_position(frame, fallback)),
        FrameKind::ReplayBegin | FrameKind::ReplayEnd => frame_offset(frame)
            .map(StreamPosition::terminal)
            .unwrap_or_else(|| initial_stream_position(frame, fallback)),
        _ => fallback,
    }
}

fn frame_offset(frame: &Frame) -> Option<u64> {
    let offset = frame.payload.get(..size_of::<u64>())?.try_into().ok()?;
    Some(u64::from_be_bytes(offset))
}

#[cfg(test)]
mod tests {
    use homie_proto::stream::{StreamReset, StreamResetReason};
    use homie_proto::transport::{EndpointRole, Frame, FrameHeader, FrameKind, WIRE_MAJOR};
    use tokio::io::{AsyncRead, AsyncReadExt};

    use super::{
        HIGH_BURST_QUOTA, HIGH_QUEUE_CAPACITY, LOW_QUEUE_CAPACITY, LowEnqueue, StreamPosition,
        WriterError, writer_channel,
    };

    #[tokio::test]
    async fn high_queue_accepts_exactly_256_frames_then_closes_connection() {
        let (writer, _driver) = writer_channel();

        for _ in 0..HIGH_QUEUE_CAPACITY {
            writer
                .try_send_high(high_frame())
                .expect("frame within high capacity");
        }
        let error = writer
            .try_send_high(high_frame())
            .expect_err("257th high frame must fail");

        assert_eq!(error, WriterError::HighQueueFull);
        assert!(writer.is_closed());
    }

    #[tokio::test]
    async fn flush_completes_only_after_preceding_high_frame_is_written() {
        let (writer, driver) = writer_channel();
        writer.try_send_high(high_frame()).expect("high frame");
        let (socket, mut peer) = tokio::io::duplex(1);
        let writer_task = tokio::spawn(driver.run(socket));
        let flush_writer = writer.clone();
        let flush_task = tokio::spawn(async move { flush_writer.flush().await });
        tokio::task::yield_now().await;

        assert!(!flush_task.is_finished());
        assert_eq!(read_frame(&mut peer).await.header.kind, FrameKind::Pong);
        flush_task.await.expect("flush task").expect("flush result");

        drop(writer);
        writer_task
            .await
            .expect("writer task")
            .expect("writer result");
    }

    #[tokio::test]
    async fn flush_fails_with_socket_error_when_preceding_frame_cannot_be_written() {
        let (writer, driver) = writer_channel();
        writer.try_send_high(high_frame()).expect("high frame");
        let (socket, peer) = tokio::io::duplex(1024);
        drop(peer);
        let writer_task = tokio::spawn(driver.run(socket));

        let error = writer.flush().await.expect_err("flush must fail");

        assert_eq!(error, WriterError::SocketWrite);
        assert_eq!(
            writer_task
                .await
                .expect("writer task")
                .expect_err("writer must fail"),
            WriterError::SocketWrite
        );
    }

    #[tokio::test]
    async fn explicit_close_fails_pending_flush_barrier() {
        let (writer, driver) = writer_channel();
        writer.try_send_high(high_frame()).expect("high frame");
        let (socket, _peer) = tokio::io::duplex(1);
        let writer_task = tokio::spawn(driver.run(socket));
        let flush_writer = writer.clone();
        let flush_task = tokio::spawn(async move { flush_writer.flush().await });
        tokio::task::yield_now().await;

        writer.close();

        assert_eq!(
            flush_task
                .await
                .expect("flush task")
                .expect_err("flush must fail"),
            WriterError::ConnectionClosed
        );
        writer_task
            .await
            .expect("writer task")
            .expect_err("writer must close");
    }

    #[tokio::test]
    async fn flush_barrier_shares_the_existing_256_slot_high_queue() {
        let (writer, _driver) = writer_channel();
        for _ in 0..HIGH_QUEUE_CAPACITY - 1 {
            writer.try_send_high(high_frame()).expect("high frame");
        }
        let flush_writer = writer.clone();
        let flush_task = tokio::spawn(async move { flush_writer.flush().await });
        tokio::task::yield_now().await;
        assert!(!flush_task.is_finished());

        let error = writer
            .try_send_high(high_frame())
            .expect_err("257th high item must fail");

        assert_eq!(error, WriterError::HighQueueFull);
        assert!(writer.is_closed());
        assert_eq!(
            flush_task
                .await
                .expect("flush task")
                .expect_err("barrier must fail"),
            WriterError::HighQueueFull
        );
    }

    #[tokio::test]
    async fn event_overflow_without_driver_resets_to_sequence_before_first_event() {
        let (writer, driver) = writer_channel();
        let (socket, mut peer) = tokio::io::duplex(64 * 1024);

        for sequence in 1..=LOW_QUEUE_CAPACITY {
            assert_eq!(
                writer
                    .try_send_low(
                        low_frame(1, sequence as u64),
                        StreamPosition::event(sequence as u64 - 1),
                    )
                    .expect("frame within low capacity"),
                LowEnqueue::Queued
            );
        }
        assert_eq!(
            writer
                .try_send_low(low_frame(1, 257), StreamPosition::event(256))
                .expect("slow stream reset"),
            LowEnqueue::StreamReset
        );

        let task = tokio::spawn(driver.run(socket));
        let reset = read_frame(&mut peer).await;
        let payload: StreamReset =
            serde_json::from_slice(&reset.payload).expect("stream reset payload");

        assert_eq!(reset.header.kind, FrameKind::StreamReset);
        assert_eq!(reset.header.stream_id, 1);
        assert_eq!(payload.reason, StreamResetReason::EventGap);
        assert_eq!(payload.latest_seq, Some(0));
        drop(writer);
        task.await.expect("writer task").expect("writer result");
    }

    #[tokio::test]
    async fn terminal_overflow_without_driver_resets_to_first_output_offset() {
        let (writer, driver) = writer_channel();

        for offset in 0..LOW_QUEUE_CAPACITY as u64 {
            writer
                .try_send_low(
                    output_frame(1, offset + 1, offset, b"x"),
                    StreamPosition::terminal(offset),
                )
                .expect("terminal output within low capacity");
        }
        assert_eq!(
            writer
                .try_send_low(
                    output_frame(1, 257, 256, b"x"),
                    StreamPosition::terminal(256),
                )
                .expect("slow terminal reset"),
            LowEnqueue::StreamReset
        );

        let (socket, mut peer) = tokio::io::duplex(1024);
        let task = tokio::spawn(driver.run(socket));
        let reset = read_frame(&mut peer).await;
        let payload: StreamReset =
            serde_json::from_slice(&reset.payload).expect("stream reset payload");

        assert_eq!(payload.last_confirmed_offset, Some(0));
        drop(writer);
        task.await.expect("writer task").expect("writer result");
    }

    #[tokio::test]
    async fn terminal_overflow_resets_to_replay_begin_offset() {
        let (writer, driver) = writer_channel();

        writer
            .try_send_low(replay_begin_frame(1, 1, 42), StreamPosition::terminal(999))
            .expect("replay begin");
        for index in 0..LOW_QUEUE_CAPACITY as u64 - 1 {
            writer
                .try_send_low(
                    output_frame(1, index + 2, 42 + index, b"x"),
                    StreamPosition::terminal(42 + index),
                )
                .expect("replayed output within low capacity");
        }
        assert_eq!(
            writer
                .try_send_low(
                    output_frame(1, 257, 297, b"x"),
                    StreamPosition::terminal(297),
                )
                .expect("slow replay reset"),
            LowEnqueue::StreamReset
        );

        let (socket, mut peer) = tokio::io::duplex(1024);
        let task = tokio::spawn(driver.run(socket));
        let reset = read_frame(&mut peer).await;
        let payload: StreamReset =
            serde_json::from_slice(&reset.payload).expect("stream reset payload");

        assert_eq!(payload.last_confirmed_offset, Some(42));
        drop(writer);
        task.await.expect("writer task").expect("writer result");
    }

    #[tokio::test]
    async fn terminal_overflow_resets_to_last_written_output_offset() {
        let (writer, driver) = writer_channel();

        for offset in 0..LOW_QUEUE_CAPACITY as u64 {
            writer
                .try_send_low(
                    output_frame(1, offset + 1, offset, b"x"),
                    StreamPosition::terminal(offset),
                )
                .expect("terminal output within low capacity");
        }
        let written = write_next_low_frame(&driver).await;
        assert_eq!(written.header.kind, FrameKind::Output);
        writer
            .try_send_low(
                output_frame(1, 257, 256, b"x"),
                StreamPosition::terminal(256),
            )
            .expect("replacement terminal output");
        assert_eq!(
            writer
                .try_send_low(
                    output_frame(1, 258, 257, b"x"),
                    StreamPosition::terminal(257),
                )
                .expect("slow terminal reset"),
            LowEnqueue::StreamReset
        );

        let (socket, mut peer) = tokio::io::duplex(1024);
        let task = tokio::spawn(driver.run(socket));
        let reset = read_frame(&mut peer).await;
        let payload: StreamReset =
            serde_json::from_slice(&reset.payload).expect("stream reset payload");

        assert_eq!(payload.last_confirmed_offset, Some(1));
        drop(writer);
        task.await.expect("writer task").expect("writer result");
    }

    #[tokio::test]
    async fn event_overflow_resets_to_last_written_sequence() {
        let (writer, driver) = writer_channel();

        for sequence in 1..=LOW_QUEUE_CAPACITY as u64 {
            writer
                .try_send_low(low_frame(1, sequence), StreamPosition::event(sequence - 1))
                .expect("event within low capacity");
        }
        let written = write_next_low_frame(&driver).await;
        assert_eq!(written.header.sequence, 1);
        writer
            .try_send_low(low_frame(1, 257), StreamPosition::event(256))
            .expect("replacement event");
        assert_eq!(
            writer
                .try_send_low(low_frame(1, 258), StreamPosition::event(257))
                .expect("slow event reset"),
            LowEnqueue::StreamReset
        );

        let (socket, mut peer) = tokio::io::duplex(1024);
        let task = tokio::spawn(driver.run(socket));
        let reset = read_frame(&mut peer).await;
        let payload: StreamReset =
            serde_json::from_slice(&reset.payload).expect("stream reset payload");

        assert_eq!(payload.reason, StreamResetReason::EventGap);
        assert_eq!(payload.latest_seq, Some(1));
        drop(writer);
        task.await.expect("writer task").expect("writer result");
    }

    #[tokio::test]
    async fn writer_attempts_low_after_32_consecutive_high_frames() {
        let (writer, driver) = writer_channel();
        writer
            .try_send_low(low_frame(1, 1), StreamPosition::event(1))
            .expect("low frame");
        for _ in 0..=HIGH_BURST_QUOTA {
            writer.try_send_high(high_frame()).expect("high frame");
        }
        let (socket, mut peer) = tokio::io::duplex(64 * 1024);
        let task = tokio::spawn(driver.run(socket));

        let mut kinds = Vec::new();
        for _ in 0..HIGH_BURST_QUOTA + 2 {
            kinds.push(read_frame(&mut peer).await.header.kind);
        }

        assert!(
            kinds[..HIGH_BURST_QUOTA]
                .iter()
                .all(|kind| *kind == FrameKind::Pong)
        );
        assert_eq!(kinds[HIGH_BURST_QUOTA], FrameKind::Event);
        assert_eq!(kinds[HIGH_BURST_QUOTA + 1], FrameKind::Pong);
        drop(writer);
        task.await.expect("writer task").expect("writer result");
    }

    #[tokio::test]
    async fn low_streams_are_scheduled_round_robin() {
        let (writer, driver) = writer_channel();
        for sequence in 1..=2 {
            writer
                .try_send_low(low_frame(1, sequence), StreamPosition::event(sequence))
                .expect("stream one frame");
            writer
                .try_send_low(low_frame(3, sequence), StreamPosition::event(sequence))
                .expect("stream three frame");
        }
        let (socket, mut peer) = tokio::io::duplex(64 * 1024);
        let task = tokio::spawn(driver.run(socket));

        let mut stream_ids = Vec::new();
        for _ in 0..4 {
            stream_ids.push(read_frame(&mut peer).await.header.stream_id);
        }

        assert_eq!(stream_ids, vec![1, 3, 1, 3]);
        drop(writer);
        task.await.expect("writer task").expect("writer result");
    }

    #[tokio::test]
    async fn resetting_stream_removes_its_pending_low_frames() {
        let (writer, driver) = writer_channel();
        writer
            .try_send_low(low_frame(1, 1), StreamPosition::event(1))
            .expect("stream one frame");
        writer
            .try_send_low(low_frame(3, 1), StreamPosition::event(1))
            .expect("stream three frame");
        writer.reset_stream(1);
        let (socket, mut peer) = tokio::io::duplex(64 * 1024);
        let task = tokio::spawn(driver.run(socket));

        let frame = read_frame(&mut peer).await;

        assert_eq!(frame.header.stream_id, 3);
        drop(writer);
        task.await.expect("writer task").expect("writer result");
    }

    #[tokio::test]
    async fn reset_stream_clears_delivered_position_before_stream_reuse() {
        let (writer, driver) = writer_channel();
        writer
            .try_send_low(low_frame(1, 1), StreamPosition::event(0))
            .expect("first stream generation");
        let written = write_next_low_frame(&driver).await;
        assert_eq!(written.header.sequence, 1);
        writer.reset_stream(1);

        for sequence in 10..10 + LOW_QUEUE_CAPACITY as u64 {
            writer
                .try_send_low(low_frame(1, sequence), StreamPosition::event(sequence - 1))
                .expect("reused stream event within low capacity");
        }
        assert_eq!(
            writer
                .try_send_low(low_frame(1, 266), StreamPosition::event(265))
                .expect("reused stream reset"),
            LowEnqueue::StreamReset
        );

        let (socket, mut peer) = tokio::io::duplex(1024);
        let task = tokio::spawn(driver.run(socket));
        let reset = read_frame(&mut peer).await;
        let payload: StreamReset =
            serde_json::from_slice(&reset.payload).expect("stream reset payload");

        assert_eq!(payload.latest_seq, Some(9));
        drop(writer);
        task.await.expect("writer task").expect("writer result");
    }

    #[tokio::test]
    async fn high_queue_overflow_drops_socket_without_writing_queued_frames() {
        let (writer, driver) = writer_channel();
        for _ in 0..HIGH_QUEUE_CAPACITY {
            writer.try_send_high(high_frame()).expect("high frame");
        }
        assert_eq!(
            writer.try_send_high(high_frame()),
            Err(WriterError::HighQueueFull)
        );
        let (socket, mut peer) = tokio::io::duplex(1024);
        let task = tokio::spawn(driver.run(socket));
        let mut byte = [0_u8; 1];

        let read = peer.read(&mut byte).await.expect("peer read");

        assert_eq!(read, 0);
        task.await
            .expect("writer task")
            .expect_err("closed writer result");
    }

    #[tokio::test]
    async fn socket_write_failure_marks_connection_closed() {
        let (writer, driver) = writer_channel();
        writer.try_send_high(high_frame()).expect("high frame");
        let (socket, peer) = tokio::io::duplex(1024);
        drop(peer);

        let error = driver.run(socket).await.expect_err("socket write failure");

        assert_eq!(error, WriterError::SocketWrite);
        assert!(writer.is_closed());
    }

    #[tokio::test]
    async fn explicit_close_drops_socket_without_flushing_queued_frames() {
        let (writer, driver) = writer_channel();
        writer.try_send_high(high_frame()).expect("high frame");
        writer.close();
        let (socket, mut peer) = tokio::io::duplex(1024);
        let task = tokio::spawn(driver.run(socket));
        let mut byte = [0_u8; 1];

        let read = peer.read(&mut byte).await.expect("peer read");

        assert_eq!(read, 0);
        task.await
            .expect("writer task")
            .expect_err("closed writer result");
    }

    #[tokio::test]
    async fn slow_stream_reset_does_not_block_another_low_stream() {
        let (writer, driver) = writer_channel();
        for sequence in 1..=LOW_QUEUE_CAPACITY {
            writer
                .try_send_low(
                    low_frame(1, sequence as u64),
                    StreamPosition::event(sequence as u64),
                )
                .expect("slow stream frame");
        }
        writer
            .try_send_low(low_frame(3, 7), StreamPosition::event(7))
            .expect("healthy stream frame");
        assert_eq!(
            writer
                .try_send_low(low_frame(1, 257), StreamPosition::event(256))
                .expect("slow stream reset"),
            LowEnqueue::StreamReset
        );
        let (socket, mut peer) = tokio::io::duplex(64 * 1024);
        let task = tokio::spawn(driver.run(socket));

        let reset = read_frame(&mut peer).await;
        let healthy = read_frame(&mut peer).await;

        assert_eq!(reset.header.kind, FrameKind::StreamReset);
        assert_eq!(reset.header.stream_id, 1);
        assert_eq!(healthy.header.kind, FrameKind::Event);
        assert_eq!(healthy.header.stream_id, 3);
        drop(writer);
        task.await.expect("writer task").expect("writer result");
    }

    fn high_frame() -> Frame {
        Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind: FrameKind::Pong,
                flags: 0,
                stream_id: 0,
                message_id: 0,
                sequence: 0,
            },
            payload: Vec::new(),
        }
    }

    fn low_frame(stream_id: u32, sequence: u64) -> Frame {
        Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind: FrameKind::Event,
                flags: 0,
                stream_id,
                message_id: 0,
                sequence,
            },
            payload: br#"{}"#.to_vec(),
        }
    }

    fn output_frame(stream_id: u32, sequence: u64, offset: u64, bytes: &[u8]) -> Frame {
        let mut payload = Vec::with_capacity(size_of::<u64>() + bytes.len());
        payload.extend_from_slice(&offset.to_be_bytes());
        payload.extend_from_slice(bytes);
        Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind: FrameKind::Output,
                flags: 0,
                stream_id,
                message_id: 0,
                sequence,
            },
            payload,
        }
    }

    fn replay_begin_frame(stream_id: u32, sequence: u64, offset: u64) -> Frame {
        Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind: FrameKind::ReplayBegin,
                flags: 0,
                stream_id,
                message_id: 0,
                sequence,
            },
            payload: offset.to_be_bytes().to_vec(),
        }
    }

    async fn write_next_low_frame(driver: &super::WriterDriver) -> Frame {
        let item = driver
            .pop_low()
            .expect("low queue lock")
            .expect("pending low frame");
        let (mut socket, mut peer) = tokio::io::duplex(1024);
        driver
            .write_low_frame(&mut socket, item)
            .await
            .expect("low frame write");
        read_frame(&mut peer).await
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
}
