use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use homie_proto::transport::Frame;
use tokio::sync::{Notify, mpsc};

const HIGH_QUEUE_CAPACITY: usize = 256;
const LOW_QUEUE_CAPACITY: usize = 256;
const HIGH_BURST_LIMIT: usize = 32;

#[derive(Clone)]
pub(crate) struct WriterHandle {
    high: mpsc::Sender<Frame>,
    low: Arc<Mutex<LowQueues>>,
    low_notify: Arc<Notify>,
}

pub(crate) struct WriterReceiver {
    high: mpsc::Receiver<Frame>,
    low: Arc<Mutex<LowQueues>>,
    low_notify: Arc<Notify>,
    consecutive_high: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QueueError {
    Backpressure,
    Closed,
}

#[derive(Default)]
struct LowQueues {
    queues: HashMap<u32, VecDeque<Frame>>,
    ready: VecDeque<u32>,
}

pub(crate) fn channel() -> (WriterHandle, WriterReceiver) {
    let (high, high_rx) = mpsc::channel(HIGH_QUEUE_CAPACITY);
    let low = Arc::new(Mutex::new(LowQueues::default()));
    let low_notify = Arc::new(Notify::new());
    (
        WriterHandle {
            high,
            low: low.clone(),
            low_notify: low_notify.clone(),
        },
        WriterReceiver {
            high: high_rx,
            low,
            low_notify,
            consecutive_high: 0,
        },
    )
}

impl WriterHandle {
    pub(crate) fn try_send_high(&self, frame: Frame) -> Result<(), QueueError> {
        self.high.try_send(frame).map_err(|error| match error {
            mpsc::error::TrySendError::Full(_) => QueueError::Backpressure,
            mpsc::error::TrySendError::Closed(_) => QueueError::Closed,
        })
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the client wire contract reserves bounded low-priority stream output"
        )
    )]
    pub(crate) fn try_send_low(&self, stream_id: u32, frame: Frame) -> Result<(), QueueError> {
        let mut low = self.low.lock().expect("writer low queue lock poisoned");
        let queue = low.queues.entry(stream_id).or_default();
        if queue.len() >= LOW_QUEUE_CAPACITY {
            return Err(QueueError::Backpressure);
        }
        let was_empty = queue.is_empty();
        queue.push_back(frame);
        if was_empty {
            low.ready.push_back(stream_id);
        }
        drop(low);
        self.low_notify.notify_one();
        Ok(())
    }

    pub(crate) fn close_stream(&self, stream_id: u32) {
        let mut low = self.low.lock().expect("writer low queue lock poisoned");
        low.queues.remove(&stream_id);
        low.ready.retain(|queued| *queued != stream_id);
    }
}

impl WriterReceiver {
    pub(crate) async fn next(&mut self) -> Option<Frame> {
        loop {
            if self.consecutive_high >= HIGH_BURST_LIMIT
                && let Some(frame) = self.pop_low()
            {
                self.consecutive_high = 0;
                return Some(frame);
            }

            match self.high.try_recv() {
                Ok(frame) => {
                    self.consecutive_high += 1;
                    return Some(frame);
                }
                Err(mpsc::error::TryRecvError::Disconnected) if !self.has_low() => return None,
                Err(mpsc::error::TryRecvError::Empty | mpsc::error::TryRecvError::Disconnected) => {
                }
            }

            if let Some(frame) = self.pop_low() {
                self.consecutive_high = 0;
                return Some(frame);
            }

            tokio::select! {
                high = self.high.recv() => {
                    if let Some(frame) = high {
                        self.consecutive_high += 1;
                        return Some(frame);
                    }
                    if !self.has_low() {
                        return None;
                    }
                }
                () = self.low_notify.notified() => {}
            }
        }
    }

    fn pop_low(&self) -> Option<Frame> {
        let mut low = self.low.lock().expect("writer low queue lock poisoned");
        while let Some(stream_id) = low.ready.pop_front() {
            let Some(queue) = low.queues.get_mut(&stream_id) else {
                continue;
            };
            let frame = queue.pop_front();
            if !queue.is_empty() {
                low.ready.push_back(stream_id);
            }
            if frame.is_some() {
                return frame;
            }
        }
        None
    }

    fn has_low(&self) -> bool {
        !self
            .low
            .lock()
            .expect("writer low queue lock poisoned")
            .ready
            .is_empty()
    }
}

#[cfg(test)]
mod tests {
    use homie_proto::transport::{Frame, FrameHeader, FrameKind, WIRE_MAJOR};

    use super::{QueueError, channel};

    #[test]
    fn high_queue_rejects_frame_257() {
        let (writer, _receiver) = channel();
        for message_id in 1..=256 {
            writer
                .try_send_high(frame(FrameKind::Request, 0, message_id))
                .expect("high queue slot");
        }

        let error = writer
            .try_send_high(frame(FrameKind::Request, 0, 257))
            .expect_err("high queue must be bounded");

        assert_eq!(error, QueueError::Backpressure);
    }

    #[test]
    fn low_queue_is_bounded_per_stream() {
        let (writer, _receiver) = channel();
        for sequence in 1..=256 {
            writer
                .try_send_low(1, frame(FrameKind::Event, 1, sequence))
                .expect("low queue slot");
        }

        let error = writer
            .try_send_low(1, frame(FrameKind::Event, 1, 257))
            .expect_err("low queue must be bounded");

        assert_eq!(error, QueueError::Backpressure);
    }

    #[tokio::test]
    async fn writer_attempts_low_frame_after_32_high_frames() {
        let (writer, mut receiver) = channel();
        for message_id in 1..=33 {
            writer
                .try_send_high(frame(FrameKind::Request, 0, message_id))
                .expect("high frame");
        }
        writer
            .try_send_low(1, frame(FrameKind::Event, 1, 1))
            .expect("low frame");

        for expected in 1..=32 {
            assert_eq!(
                receiver.next().await.expect("high frame").header.message_id,
                expected
            );
        }
        assert_eq!(
            receiver.next().await.expect("fair low frame").header.kind,
            FrameKind::Event
        );
        assert_eq!(
            receiver
                .next()
                .await
                .expect("remaining high frame")
                .header
                .message_id,
            33
        );
    }

    #[tokio::test]
    async fn low_streams_are_scheduled_round_robin() {
        let (writer, mut receiver) = channel();
        writer
            .try_send_low(1, frame(FrameKind::Event, 1, 1))
            .expect("stream one first");
        writer
            .try_send_low(1, frame(FrameKind::Event, 1, 2))
            .expect("stream one second");
        writer
            .try_send_low(3, frame(FrameKind::Event, 3, 1))
            .expect("stream three");

        let first = receiver.next().await.expect("first low");
        let second = receiver.next().await.expect("second low");
        let third = receiver.next().await.expect("third low");

        assert_eq!(
            [
                first.header.stream_id,
                second.header.stream_id,
                third.header.stream_id
            ],
            [1, 3, 1]
        );
    }

    fn frame(kind: FrameKind, stream_id: u32, id: u64) -> Frame {
        Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind,
                flags: 0,
                stream_id,
                message_id: id,
                sequence: id,
            },
            payload: Vec::new(),
        }
    }
}
