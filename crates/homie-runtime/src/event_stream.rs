use std::collections::VecDeque;
use std::future::Future;
use std::io::Write;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use homie_proto::model::RuntimeEvent;
use homie_proto::stream::{EventStreamOpen, StreamReset, StreamResetReason};
use homie_proto::transport::{Frame, FrameHeader, FrameKind, WIRE_MAJOR};
use homie_proto::{EventCursor, EventsWaitRequest};
use serde::Serialize;
use serde_json::Value;
use tokio::sync::broadcast;
use tokio::task::AbortHandle;

use crate::dispatcher::AsyncWaitHandler;
use crate::runtime_actor::{ServiceError, ServiceResult};
use crate::writer::{LowEnqueue, StreamPosition, WriterError, WriterHandle};

pub const EVENT_REPLAY_CAPACITY: usize = 1024;
pub const MAX_EVENT_WAIT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventBounds {
    pub oldest_seq: u64,
    pub latest_seq: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventReplay {
    pub oldest_seq: u64,
    pub latest_seq: u64,
    pub events: Vec<RuntimeEvent>,
}

pub type EventReplayFuture<'a> = Pin<Box<dyn Future<Output = EventReplay> + Send + 'a>>;

pub struct EventStore {
    data_dir: PathBuf,
    state: Mutex<EventStoreState>,
    sender: broadcast::Sender<RuntimeEvent>,
}

struct EventStoreState {
    events: VecDeque<RuntimeEvent>,
    next_seq: u64,
}

impl EventStore {
    pub fn open(data_dir: PathBuf) -> Result<Self, crate::RuntimeError> {
        let (events, next_seq) = load_events(&data_dir)?;
        let (sender, _) = broadcast::channel(EVENT_REPLAY_CAPACITY);
        Ok(Self {
            data_dir,
            state: Mutex::new(EventStoreState { events, next_seq }),
            sender,
        })
    }

    #[must_use]
    pub fn bounds(&self) -> EventBounds {
        let state = self.state.lock().expect("event store");
        EventBounds {
            oldest_seq: state.events.front().map_or(0, |event| event.seq),
            latest_seq: state.events.back().map_or(0, |event| event.seq),
        }
    }

    #[must_use]
    pub fn replay(&self, after_seq: u64) -> EventReplay {
        let state = self.state.lock().expect("event store");
        EventReplay {
            oldest_seq: state.events.front().map_or(0, |event| event.seq),
            latest_seq: state.events.back().map_or(0, |event| event.seq),
            events: state
                .events
                .iter()
                .filter(|event| event.seq > after_seq)
                .cloned()
                .collect(),
        }
    }

    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<RuntimeEvent> {
        self.sender.subscribe()
    }

    pub fn sync(&self) -> Result<(), crate::RuntimeError> {
        let _state = self.state.lock().expect("event store");
        let path = event_log_path(&self.data_dir);
        let file = match std::fs::OpenOptions::new().read(true).open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.into()),
        };
        file.sync_all()?;
        Ok(())
    }

    pub(super) fn publish(
        &self,
        event: &str,
        session_id: Option<String>,
        status: Option<String>,
    ) -> Result<RuntimeEvent, crate::RuntimeError> {
        let mut state = self.state.lock().expect("event store");
        let next_seq = state
            .next_seq
            .checked_add(1)
            .ok_or_else(sequence_exhausted)?;
        let runtime_event = RuntimeEvent {
            seq: state.next_seq,
            event: event.to_string(),
            session_id,
            status,
        };
        append_event(&self.data_dir, &runtime_event)?;
        state.next_seq = next_seq;
        state.events.push_back(runtime_event.clone());
        if state.events.len() > EVENT_REPLAY_CAPACITY {
            state.events.pop_front();
        }
        drop(state);
        let _ = self.sender.send(runtime_event.clone());
        Ok(runtime_event)
    }
}

pub trait EventBackend: Send + Sync + 'static {
    fn replay(&self, after_seq: u64) -> EventReplayFuture<'_>;

    fn subscribe(&self) -> broadcast::Receiver<RuntimeEvent>;
}

impl EventBackend for EventStore {
    fn replay(&self, after_seq: u64) -> EventReplayFuture<'_> {
        let replay = EventStore::replay(self, after_seq);
        Box::pin(async move { replay })
    }

    fn subscribe(&self) -> broadcast::Receiver<RuntimeEvent> {
        EventStore::subscribe(self)
    }
}

pub struct RuntimeEventWaitHandler {
    store: Arc<EventStore>,
}

impl RuntimeEventWaitHandler {
    #[must_use]
    pub fn new(store: Arc<EventStore>) -> Self {
        Self { store }
    }

    async fn wait_for_events(&self, request: EventsWaitRequest) -> ServiceResult<Value> {
        let mut live = self.store.subscribe();
        let replay = self.store.replay(request.after_seq);
        let existing = replay
            .events
            .into_iter()
            .filter(|event| matches_filter(event, &request.event_filter))
            .collect::<Vec<_>>();
        if !existing.is_empty() {
            return wait_response(false, existing, request.after_seq);
        }

        let mut observed_seq = request.after_seq.max(replay.latest_seq);
        let timeout = bounded_wait_timeout(request.timeout_ms);
        let matched = tokio::time::timeout(timeout, async {
            loop {
                match live.recv().await {
                    Ok(event) => {
                        if event.seq <= observed_seq {
                            continue;
                        }
                        observed_seq = event.seq;
                        if matches_filter(&event, &request.event_filter) {
                            return Some(vec![event]);
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        let replay = self.store.replay(observed_seq);
                        observed_seq = observed_seq.max(replay.latest_seq);
                        let events = replay
                            .events
                            .into_iter()
                            .filter(|event| matches_filter(event, &request.event_filter))
                            .collect::<Vec<_>>();
                        if !events.is_empty() {
                            return Some(events);
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => return None,
                }
            }
        })
        .await;

        match matched {
            Ok(Some(events)) => wait_response(false, events, request.after_seq),
            Ok(None) | Err(_) => wait_response(true, Vec::new(), request.after_seq),
        }
    }
}

impl AsyncWaitHandler for RuntimeEventWaitHandler {
    fn wait(
        &self,
        request: EventsWaitRequest,
    ) -> Pin<Box<dyn Future<Output = ServiceResult<Value>> + Send + '_>> {
        Box::pin(self.wait_for_events(request))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EventsWaitResponse {
    timed_out: bool,
    events: Vec<RuntimeEvent>,
    cursor: EventCursor,
}

fn wait_response(
    timed_out: bool,
    events: Vec<RuntimeEvent>,
    after_seq: u64,
) -> ServiceResult<Value> {
    let next_seq = events.last().map_or(after_seq, |event| event.seq);
    serde_json::to_value(EventsWaitResponse {
        timed_out,
        events,
        cursor: EventCursor { next_seq },
    })
    .map_err(|_| ServiceError::Internal)
}

fn bounded_wait_timeout(timeout_ms: u64) -> Duration {
    Duration::from_millis(timeout_ms).min(MAX_EVENT_WAIT_TIMEOUT)
}

pub struct EventStreamService {
    backend: Arc<dyn EventBackend>,
}

impl EventStreamService {
    #[must_use]
    pub fn new(backend: Arc<dyn EventBackend>) -> Self {
        Self { backend }
    }

    pub fn open(
        &self,
        stream_id: u32,
        request: EventStreamOpen,
        writer: WriterHandle,
    ) -> Result<EventStreamHandle, WriterError> {
        writer.try_send_high(stream_opened_frame(stream_id))?;
        let live = self.backend.subscribe();
        let backend = self.backend.clone();
        let producer_writer = writer.clone();
        let task = tokio::spawn(async move {
            produce_events(
                backend,
                live,
                stream_id,
                request.after_seq,
                request.event_filter,
                producer_writer,
            )
            .await;
        });
        Ok(EventStreamHandle {
            stream_id,
            writer,
            producer: Some(task.abort_handle()),
        })
    }
}

pub struct EventStreamHandle {
    stream_id: u32,
    writer: WriterHandle,
    producer: Option<AbortHandle>,
}

impl EventStreamHandle {
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.producer.as_ref().is_none_or(AbortHandle::is_finished)
    }

    pub fn close(mut self) {
        self.stop();
    }

    fn stop(&mut self) {
        if let Some(producer) = self.producer.take() {
            producer.abort();
            self.writer.reset_stream(self.stream_id);
        }
    }
}

impl Drop for EventStreamHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

async fn produce_events(
    backend: Arc<dyn EventBackend>,
    mut live: broadcast::Receiver<RuntimeEvent>,
    stream_id: u32,
    after_seq: u64,
    event_filter: Vec<String>,
    writer: WriterHandle,
) {
    let replay = backend.replay(after_seq).await;
    if after_seq > replay.latest_seq
        || (replay.oldest_seq != 0 && after_seq.saturating_add(1) < replay.oldest_seq)
    {
        let _ = reset_event_gap(&writer, stream_id, replay.latest_seq);
        return;
    }
    let replay_latest_seq = replay.latest_seq;
    let mut latest_seq = after_seq;
    for event in replay.events {
        if event.seq <= latest_seq {
            continue;
        }
        if event.seq != latest_seq.saturating_add(1) {
            let _ = reset_event_gap(&writer, stream_id, replay_latest_seq);
            return;
        }
        if !matches_filter(&event, &event_filter) {
            latest_seq = event.seq;
            continue;
        }
        if enqueue_event(&writer, stream_id, &event) != Ok(LowEnqueue::Queued) {
            return;
        }
        latest_seq = event.seq;
    }
    latest_seq = latest_seq.max(replay_latest_seq);

    loop {
        let event = match live.recv().await {
            Ok(event) => event,
            Err(broadcast::error::RecvError::Lagged(_)) => {
                let replay = backend.replay(latest_seq).await;
                let _ = reset_event_gap(&writer, stream_id, replay.latest_seq.max(latest_seq));
                return;
            }
            Err(broadcast::error::RecvError::Closed) => return,
        };
        if event.seq <= latest_seq {
            continue;
        }
        if event.seq != latest_seq.saturating_add(1) {
            let _ = reset_event_gap(&writer, stream_id, event.seq);
            return;
        }
        if !matches_filter(&event, &event_filter) {
            latest_seq = event.seq;
            continue;
        }
        if enqueue_event(&writer, stream_id, &event) != Ok(LowEnqueue::Queued) {
            return;
        }
        latest_seq = event.seq;
    }
}

fn matches_filter(event: &RuntimeEvent, event_filter: &[String]) -> bool {
    event_filter.is_empty() || event_filter.iter().any(|wanted| wanted == &event.event)
}

fn enqueue_event(
    writer: &WriterHandle,
    stream_id: u32,
    event: &RuntimeEvent,
) -> Result<LowEnqueue, WriterError> {
    let frame = event_frame(stream_id, event)?;
    writer.try_send_low(frame, StreamPosition::event(event.seq))
}

fn reset_event_gap(
    writer: &WriterHandle,
    stream_id: u32,
    latest_seq: u64,
) -> Result<(), WriterError> {
    writer.reset_stream(stream_id);
    writer.try_send_high(event_gap_frame(stream_id, latest_seq)?)
}

fn event_gap_frame(stream_id: u32, latest_seq: u64) -> Result<Frame, WriterError> {
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
            reason: StreamResetReason::EventGap,
            last_confirmed_offset: None,
            latest_seq: Some(latest_seq),
        })
        .map_err(|_| WriterError::InvalidFrame)?,
    })
}

fn stream_opened_frame(stream_id: u32) -> Frame {
    Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::StreamOpened,
            flags: 0,
            stream_id,
            message_id: 0,
            sequence: 0,
        },
        payload: b"{}".to_vec(),
    }
}

fn event_frame(stream_id: u32, event: &RuntimeEvent) -> Result<Frame, WriterError> {
    Ok(Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::Event,
            flags: 0,
            stream_id,
            message_id: 0,
            sequence: event.seq,
        },
        payload: serde_json::to_vec(event).map_err(|_| WriterError::InvalidFrame)?,
    })
}

fn event_log_path(data_dir: &std::path::Path) -> PathBuf {
    data_dir.join("runtime").join("events.jsonl")
}

fn load_events(
    data_dir: &std::path::Path,
) -> Result<(VecDeque<RuntimeEvent>, u64), crate::RuntimeError> {
    let content = match std::fs::read_to_string(event_log_path(data_dir)) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((VecDeque::new(), 1));
        }
        Err(error) => return Err(error.into()),
    };
    let mut events = VecDeque::with_capacity(EVENT_REPLAY_CAPACITY);
    let mut max_seq = 0_u64;
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        let event: RuntimeEvent = serde_json::from_str(line)?;
        max_seq = max_seq.max(event.seq);
        events.push_back(event);
        if events.len() > EVENT_REPLAY_CAPACITY {
            events.pop_front();
        }
    }
    let next_seq = max_seq.checked_add(1).ok_or_else(sequence_exhausted)?;
    Ok((events, next_seq))
}

fn append_event(
    data_dir: &std::path::Path,
    event: &RuntimeEvent,
) -> Result<(), crate::RuntimeError> {
    let path = event_log_path(data_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    serde_json::to_writer(&mut file, event)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn sequence_exhausted() -> crate::RuntimeError {
    std::io::Error::other("runtime event sequence exhausted").into()
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::io::Write;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    use homie_proto::EventsWaitRequest;
    use homie_proto::model::RuntimeEvent;
    use homie_proto::stream::{EventStreamOpen, StreamReset, StreamResetReason};
    use homie_proto::transport::{EndpointRole, Frame, FrameKind};
    use serde_json::json;
    use tokio::io::{AsyncRead, AsyncReadExt};
    use tokio::sync::{Notify, broadcast};

    use super::{
        EVENT_REPLAY_CAPACITY, EventBackend, EventBounds, EventReplay, EventReplayFuture,
        EventStore, EventStreamService, MAX_EVENT_WAIT_TIMEOUT, RuntimeEventWaitHandler,
        bounded_wait_timeout,
    };
    use crate::dispatcher::AsyncWaitHandler;
    use crate::writer::writer_channel;

    #[test]
    fn event_store_is_the_concrete_event_backend() {
        fn assert_event_backend<T: EventBackend>() {}

        assert_event_backend::<EventStore>();
    }

    #[test]
    fn event_store_load_caps_ring_and_continues_after_global_max_sequence() {
        let temp = tempfile::tempdir().expect("tempdir");
        let runtime_dir = temp.path().join("runtime");
        std::fs::create_dir_all(&runtime_dir).expect("runtime dir");
        let mut log = std::fs::File::create(runtime_dir.join("events.jsonl")).expect("event log");
        for seq in 1..=(EVENT_REPLAY_CAPACITY as u64 + 1) {
            serde_json::to_writer(&mut log, &event(seq, "session.updated")).expect("event");
            log.write_all(b"\n").expect("newline");
        }
        drop(log);

        let store = EventStore::open(temp.path().to_path_buf()).expect("event store");
        let replay = store.replay(0);
        let published = store
            .publish("session.completed", Some("session-1".to_string()), None)
            .expect("publish");

        assert_eq!(
            (
                store.bounds(),
                replay.events.first().map(|event| event.seq),
                replay.events.len(),
                published.seq,
            ),
            (
                EventBounds {
                    oldest_seq: 3,
                    latest_seq: 1026,
                },
                Some(2),
                EVENT_REPLAY_CAPACITY,
                1026,
            )
        );
    }

    #[test]
    fn event_store_rejects_loaded_max_sequence() {
        let temp = tempfile::tempdir().expect("tempdir");
        let runtime_dir = temp.path().join("runtime");
        std::fs::create_dir_all(&runtime_dir).expect("runtime dir");
        let mut log = std::fs::File::create(runtime_dir.join("events.jsonl")).expect("event log");
        serde_json::to_writer(&mut log, &event(u64::MAX, "session.updated")).expect("event");
        log.write_all(b"\n").expect("newline");
        drop(log);

        let Err(error) = EventStore::open(temp.path().to_path_buf()) else {
            panic!("exhausted sequence must fail");
        };

        assert_eq!(
            error.to_string(),
            "I/O error: runtime event sequence exhausted"
        );
    }

    #[test]
    fn event_store_rejects_publish_when_next_sequence_cannot_advance() {
        let temp = tempfile::tempdir().expect("tempdir");
        let runtime_dir = temp.path().join("runtime");
        std::fs::create_dir_all(&runtime_dir).expect("runtime dir");
        let event = event(u64::MAX - 1, "session.updated");
        let log_path = runtime_dir.join("events.jsonl");
        let mut log = std::fs::File::create(&log_path).expect("event log");
        serde_json::to_writer(&mut log, &event).expect("event");
        log.write_all(b"\n").expect("newline");
        drop(log);
        let original_log = std::fs::read_to_string(&log_path).expect("original event log");
        let store = EventStore::open(temp.path().to_path_buf()).expect("event store");

        let error = store
            .publish("session.completed", Some("session-1".to_string()), None)
            .expect_err("sequence without a successor must fail");

        assert_eq!(
            (
                error.to_string(),
                store.replay(0).events,
                std::fs::read_to_string(log_path).expect("event log"),
            ),
            (
                "I/O error: runtime event sequence exhausted".to_string(),
                vec![event],
                original_log,
            )
        );
    }

    #[test]
    fn event_store_syncs_the_persisted_log() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = EventStore::open(temp.path().to_path_buf()).expect("event store");
        store
            .publish("session.updated", Some("session-1".to_string()), None)
            .expect("publish");

        store.sync().expect("sync event log");

        assert!(temp.path().join("runtime/events.jsonl").is_file());
    }

    #[tokio::test]
    async fn event_store_publish_is_persisted_replayed_and_broadcast_live() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = EventStore::open(temp.path().to_path_buf()).expect("event store");
        let mut live = store.subscribe();

        let published = store
            .publish(
                "session.status",
                Some("session-1".to_string()),
                Some("running".to_string()),
            )
            .expect("publish");
        let received = live.recv().await.expect("live event");
        let replayed = store.replay(0).events;
        drop(live);
        drop(store);
        let reopened = EventStore::open(temp.path().to_path_buf()).expect("reopen event store");

        assert_eq!(
            (
                published.clone(),
                received,
                replayed,
                reopened.replay(0).events
            ),
            (
                published.clone(),
                published.clone(),
                vec![published.clone()],
                vec![published],
            )
        );
    }

    #[tokio::test]
    async fn events_wait_returns_existing_event_immediately() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = Arc::new(EventStore::open(temp.path().to_path_buf()).expect("event store"));
        let event = store
            .publish(
                "session.status",
                Some("session-1".to_string()),
                Some("running".to_string()),
            )
            .expect("publish");
        let handler = RuntimeEventWaitHandler::new(store);

        let response = tokio::time::timeout(
            Duration::from_millis(50),
            handler.wait(EventsWaitRequest {
                after_seq: 0,
                timeout_ms: 30_000,
                event_filter: Vec::new(),
            }),
        )
        .await
        .expect("existing event must not wait")
        .expect("wait response");

        assert_eq!(
            response,
            json!({
                "timedOut": false,
                "events": [event],
                "cursor": {"nextSeq": 1}
            })
        );
    }

    #[tokio::test]
    async fn events_wait_receives_event_published_after_subscription() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = Arc::new(EventStore::open(temp.path().to_path_buf()).expect("event store"));
        let handler = RuntimeEventWaitHandler::new(store.clone());
        let waiter = tokio::spawn(async move {
            handler
                .wait(EventsWaitRequest {
                    after_seq: 0,
                    timeout_ms: 1_000,
                    event_filter: Vec::new(),
                })
                .await
        });
        wait_for_store_subscriber_count(&store, 1).await;

        let event = store
            .publish("session.output", Some("session-1".to_string()), None)
            .expect("publish");
        let response = waiter.await.expect("wait task").expect("wait response");

        assert_eq!(
            response,
            json!({
                "timedOut": false,
                "events": [event],
                "cursor": {"nextSeq": 1}
            })
        );
    }

    #[tokio::test]
    async fn events_wait_timeout_returns_empty_success_response() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = Arc::new(EventStore::open(temp.path().to_path_buf()).expect("event store"));
        let handler = RuntimeEventWaitHandler::new(store);
        let started = Instant::now();

        let response = handler
            .wait(EventsWaitRequest {
                after_seq: 9,
                timeout_ms: 10,
                event_filter: Vec::new(),
            })
            .await
            .expect("wait response");

        assert_eq!(
            (response, started.elapsed() >= Duration::from_millis(10)),
            (
                json!({
                    "timedOut": true,
                    "events": [],
                    "cursor": {"nextSeq": 9}
                }),
                true,
            )
        );
    }

    #[test]
    fn events_wait_timeout_is_capped() {
        assert_eq!(bounded_wait_timeout(u64::MAX), MAX_EVENT_WAIT_TIMEOUT);
    }

    #[tokio::test]
    async fn events_wait_filter_returns_only_matching_events() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = Arc::new(EventStore::open(temp.path().to_path_buf()).expect("event store"));
        store
            .publish("session.output", Some("session-1".to_string()), None)
            .expect("publish ignored event");
        let matching = store
            .publish(
                "session.status",
                Some("session-1".to_string()),
                Some("idle".to_string()),
            )
            .expect("publish matching event");
        let handler = RuntimeEventWaitHandler::new(store);

        let response = handler
            .wait(EventsWaitRequest {
                after_seq: 0,
                timeout_ms: 1_000,
                event_filter: vec!["session.status".to_string()],
            })
            .await
            .expect("wait response");

        assert_eq!(
            response,
            json!({
                "timedOut": false,
                "events": [matching],
                "cursor": {"nextSeq": 2}
            })
        );
    }

    #[tokio::test]
    async fn valid_replay_is_sent_in_order_after_stream_opened() {
        let backend = Arc::new(FakeBackend::with_events([
            event(1, "session.created"),
            event(2, "session.updated"),
            event(3, "session.completed"),
        ]));
        let service = EventStreamService::new(backend);
        let (writer, driver) = writer_channel();
        let handle = service
            .open(
                1,
                EventStreamOpen {
                    after_seq: 1,
                    event_filter: Vec::new(),
                },
                writer.clone(),
            )
            .expect("open event stream");
        let (socket, mut peer) = tokio::io::duplex(64 * 1024);
        let driver_task = tokio::spawn(driver.run(socket));

        let opened = read_frame(&mut peer).await;
        let first = read_frame(&mut peer).await;
        let second = read_frame(&mut peer).await;

        assert_eq!(opened.header.kind, FrameKind::StreamOpened);
        assert_eq!(opened.header.stream_id, 1);
        assert_eq!(decode_event(&first), event(2, "session.updated"));
        assert_eq!(first.header.sequence, 2);
        assert_eq!(decode_event(&second), event(3, "session.completed"));
        assert_eq!(second.header.sequence, 3);

        handle.close();
        drop(writer);
        driver_task
            .await
            .expect("writer task")
            .expect("writer result");
    }

    #[tokio::test]
    async fn too_old_cursor_resets_with_latest_sequence_without_partial_replay() {
        let backend = Arc::new(FakeBackend::with_events([
            event(10, "session.updated"),
            event(11, "session.completed"),
        ]));
        let service = EventStreamService::new(backend);
        let (writer, driver) = writer_channel();
        let handle = service
            .open(
                1,
                EventStreamOpen {
                    after_seq: 3,
                    event_filter: Vec::new(),
                },
                writer.clone(),
            )
            .expect("open event stream");
        let (socket, mut peer) = tokio::io::duplex(64 * 1024);
        let driver_task = tokio::spawn(driver.run(socket));

        let opened = read_frame(&mut peer).await;
        let reset = read_frame(&mut peer).await;
        let payload: StreamReset =
            serde_json::from_slice(&reset.payload).expect("stream reset payload");

        assert_eq!(opened.header.kind, FrameKind::StreamOpened);
        assert_eq!(reset.header.kind, FrameKind::StreamReset);
        assert_eq!(payload.reason, StreamResetReason::EventGap);
        assert_eq!(payload.latest_seq, Some(11));
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(20), read_frame(&mut peer))
                .await
                .is_err(),
            "gap reset must not be followed by partial replay"
        );

        handle.close();
        drop(writer);
        driver_task
            .await
            .expect("writer task")
            .expect("writer result");
    }

    #[tokio::test]
    async fn replay_forward_gap_resets_with_replay_latest_without_partial_delivery() {
        let backend = Arc::new(FakeBackend::with_events([
            event(2, "session.updated"),
            event(4, "session.completed"),
        ]));
        let service = EventStreamService::new(backend.clone());
        let (writer, driver) = writer_channel();
        let handle = service
            .open(
                1,
                EventStreamOpen {
                    after_seq: 1,
                    event_filter: Vec::new(),
                },
                writer.clone(),
            )
            .expect("open event stream");
        wait_for_subscriber_count(&backend, 0).await;
        let (socket, mut peer) = tokio::io::duplex(64 * 1024);
        let driver_task = tokio::spawn(driver.run(socket));

        let opened = read_frame(&mut peer).await;
        let reset = read_frame(&mut peer).await;
        let payload: StreamReset =
            serde_json::from_slice(&reset.payload).expect("stream reset payload");

        assert_eq!(opened.header.kind, FrameKind::StreamOpened);
        assert_eq!(reset.header.kind, FrameKind::StreamReset);
        assert_eq!(payload.reason, StreamResetReason::EventGap);
        assert_eq!(payload.latest_seq, Some(4));
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(20), read_frame(&mut peer))
                .await
                .is_err(),
            "replay gap reset must remove partial replay frames"
        );

        handle.close();
        drop(writer);
        driver_task
            .await
            .expect("writer task")
            .expect("writer result");
    }

    #[tokio::test]
    async fn daemon_cursor_reset_from_nonzero_latest_resets_stale_after_seq() {
        assert_cursor_ahead_resets([event(5, "session.updated")], 9, 5).await;
    }

    #[tokio::test]
    async fn daemon_cursor_reset_to_zero_resets_stale_after_seq() {
        assert_cursor_ahead_resets([], 9, 0).await;
    }

    #[tokio::test]
    async fn event_filter_applies_to_replay_and_live_events() {
        let backend = Arc::new(FakeBackend::with_events([
            event(1, "session.updated"),
            event(2, "session.completed"),
        ]));
        let service = EventStreamService::new(backend.clone());
        let (writer, driver) = writer_channel();
        let handle = service
            .open(
                1,
                EventStreamOpen {
                    after_seq: 0,
                    event_filter: vec!["session.updated".to_string()],
                },
                writer.clone(),
            )
            .expect("open event stream");
        backend.publish(event(3, "session.completed"));
        backend.publish(event(4, "session.updated"));
        let (socket, mut peer) = tokio::io::duplex(64 * 1024);
        let driver_task = tokio::spawn(driver.run(socket));

        let opened = read_frame(&mut peer).await;
        let replayed = read_frame(&mut peer).await;
        let live = read_frame(&mut peer).await;

        assert_eq!(opened.header.kind, FrameKind::StreamOpened);
        assert_eq!(decode_event(&replayed), event(1, "session.updated"));
        assert_eq!(decode_event(&live), event(4, "session.updated"));

        handle.close();
        drop(writer);
        driver_task
            .await
            .expect("writer task")
            .expect("writer result");
    }

    #[tokio::test]
    async fn event_published_during_replay_is_delivered_once() {
        let backend = Arc::new(FakeBackend::with_replay_race(
            [event(1, "session.created"), event(2, "session.updated")],
            event(3, "session.completed"),
        ));
        let service = EventStreamService::new(backend);
        let (writer, driver) = writer_channel();
        let handle = service
            .open(
                1,
                EventStreamOpen {
                    after_seq: 0,
                    event_filter: Vec::new(),
                },
                writer.clone(),
            )
            .expect("open event stream");
        let (socket, mut peer) = tokio::io::duplex(64 * 1024);
        let driver_task = tokio::spawn(driver.run(socket));

        let opened = read_frame(&mut peer).await;
        let first = read_frame(&mut peer).await;
        let second = read_frame(&mut peer).await;
        let raced = read_frame(&mut peer).await;

        assert_eq!(opened.header.kind, FrameKind::StreamOpened);
        assert_eq!(decode_event(&first).seq, 1);
        assert_eq!(decode_event(&second).seq, 2);
        assert_eq!(decode_event(&raced).seq, 3);
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(20), read_frame(&mut peer))
                .await
                .is_err(),
            "event present in replay and live receiver must not be duplicated"
        );

        handle.close();
        drop(writer);
        driver_task
            .await
            .expect("writer task")
            .expect("writer result");
    }

    #[tokio::test]
    async fn forward_live_gap_resets_stream_with_observed_latest_sequence() {
        let backend = Arc::new(FakeBackend::with_events([
            event(1, "session.created"),
            event(2, "session.updated"),
        ]));
        let service = EventStreamService::new(backend.clone());
        let (writer, driver) = writer_channel();
        let handle = service
            .open(
                1,
                EventStreamOpen {
                    after_seq: 2,
                    event_filter: Vec::new(),
                },
                writer.clone(),
            )
            .expect("open event stream");
        backend.publish(event(4, "session.completed"));
        let (socket, mut peer) = tokio::io::duplex(64 * 1024);
        let driver_task = tokio::spawn(driver.run(socket));

        let opened = read_frame(&mut peer).await;
        let reset = read_frame(&mut peer).await;
        let payload: StreamReset =
            serde_json::from_slice(&reset.payload).expect("stream reset payload");

        assert_eq!(opened.header.kind, FrameKind::StreamOpened);
        assert_eq!(reset.header.kind, FrameKind::StreamReset);
        assert_eq!(payload.reason, StreamResetReason::EventGap);
        assert_eq!(payload.latest_seq, Some(4));

        handle.close();
        drop(writer);
        driver_task
            .await
            .expect("writer task")
            .expect("writer result");
    }

    #[tokio::test]
    async fn broadcast_lag_resets_stream_with_backend_latest_sequence() {
        let (backend, replay_entered, release_replay) = FakeBackend::with_blocked_replay([]);
        let backend = Arc::new(backend);
        let service = EventStreamService::new(backend.clone());
        let (writer, driver) = writer_channel();
        let handle = service
            .open(
                1,
                EventStreamOpen {
                    after_seq: 0,
                    event_filter: Vec::new(),
                },
                writer.clone(),
            )
            .expect("open event stream");
        replay_entered.notified().await;
        for seq in 1..=(EVENT_REPLAY_CAPACITY as u64 + 1) {
            backend.publish_retained(event(seq, "session.updated"));
        }
        release_replay.notify_one();
        let (socket, mut peer) = tokio::io::duplex(64 * 1024);
        let driver_task = tokio::spawn(driver.run(socket));

        let opened = read_frame(&mut peer).await;
        let reset = tokio::time::timeout(std::time::Duration::from_secs(1), read_frame(&mut peer))
            .await
            .expect("broadcast lag must reset stream");
        let payload: StreamReset =
            serde_json::from_slice(&reset.payload).expect("stream reset payload");

        assert_eq!(opened.header.kind, FrameKind::StreamOpened);
        assert_eq!(reset.header.kind, FrameKind::StreamReset);
        assert_eq!(payload.reason, StreamResetReason::EventGap);
        assert_eq!(payload.latest_seq, Some(1025));

        handle.close();
        drop(writer);
        driver_task
            .await
            .expect("writer task")
            .expect("writer result");
    }

    #[tokio::test]
    async fn event_queue_overflow_resets_gap_and_is_isolated_to_one_stream() {
        let mut events = (1..=257).map(|seq| event(seq, "slow")).collect::<Vec<_>>();
        events.push(event(258, "healthy"));
        let backend = Arc::new(FakeBackend::with_events(events));
        let service = EventStreamService::new(backend.clone());
        let (writer, driver) = writer_channel();
        let slow = service
            .open(
                1,
                EventStreamOpen {
                    after_seq: 0,
                    event_filter: vec!["slow".to_string()],
                },
                writer.clone(),
            )
            .expect("open slow stream");
        let healthy = service
            .open(
                3,
                EventStreamOpen {
                    after_seq: 0,
                    event_filter: vec!["healthy".to_string()],
                },
                writer.clone(),
            )
            .expect("open healthy stream");
        wait_for_subscriber_count(&backend, 1).await;
        let (socket, mut peer) = tokio::io::duplex(64 * 1024);
        let driver_task = tokio::spawn(driver.run(socket));

        let slow_opened = read_frame(&mut peer).await;
        let healthy_opened = read_frame(&mut peer).await;
        let slow_reset = read_frame(&mut peer).await;
        let reset_payload: StreamReset =
            serde_json::from_slice(&slow_reset.payload).expect("slow reset payload");
        let healthy_replay = read_frame(&mut peer).await;

        assert_eq!(slow_opened.header.stream_id, 1);
        assert_eq!(healthy_opened.header.stream_id, 3);
        assert_eq!(slow_reset.header.stream_id, 1);
        assert_eq!(reset_payload.reason, StreamResetReason::EventGap);
        assert_eq!(reset_payload.latest_seq, Some(0));
        assert_eq!(decode_event(&healthy_replay), event(258, "healthy"));
        assert!(!writer.is_closed());

        backend.publish(event(259, "healthy"));
        let healthy_live = read_frame(&mut peer).await;
        assert_eq!(decode_event(&healthy_live), event(259, "healthy"));

        slow.close();
        healthy.close();
        drop(writer);
        driver_task
            .await
            .expect("writer task")
            .expect("writer result");
    }

    #[tokio::test]
    async fn backend_replay_latest_is_snapshot_cursor_for_next_live_event() {
        let backend = Arc::new(FakeBackend::with_events(
            (1..=1025).map(|seq| event(seq, "session.updated")),
        ));
        let snapshot = backend.replay(0).await;

        assert_eq!(snapshot.oldest_seq, 2);
        assert_eq!(snapshot.latest_seq, 1025);
        assert_eq!(snapshot.events.len(), EVENT_REPLAY_CAPACITY);

        let service = EventStreamService::new(backend.clone());
        let (writer, driver) = writer_channel();
        let handle = service
            .open(
                1,
                EventStreamOpen {
                    after_seq: snapshot.latest_seq,
                    event_filter: Vec::new(),
                },
                writer.clone(),
            )
            .expect("open event stream from snapshot cursor");
        backend.publish(event(1026, "session.completed"));
        let (socket, mut peer) = tokio::io::duplex(64 * 1024);
        let driver_task = tokio::spawn(driver.run(socket));

        let opened = read_frame(&mut peer).await;
        let next = read_frame(&mut peer).await;

        assert_eq!(opened.header.kind, FrameKind::StreamOpened);
        assert_eq!(decode_event(&next), event(1026, "session.completed"));

        handle.close();
        drop(writer);
        driver_task
            .await
            .expect("writer task")
            .expect("writer result");
    }

    #[tokio::test]
    async fn close_stops_producer_without_closing_connection() {
        let backend = Arc::new(FakeBackend::with_events([]));
        let service = EventStreamService::new(backend.clone());
        let (writer, driver) = writer_channel();
        let handle = service
            .open(
                1,
                EventStreamOpen {
                    after_seq: 0,
                    event_filter: Vec::new(),
                },
                writer.clone(),
            )
            .expect("open event stream");

        handle.close();
        wait_for_subscriber_count(&backend, 0).await;

        assert!(!writer.is_closed());
        drop(writer);
        drop(driver);
    }

    #[tokio::test]
    async fn drop_stops_producer_without_closing_connection() {
        let backend = Arc::new(FakeBackend::with_events([]));
        let service = EventStreamService::new(backend.clone());
        let (writer, driver) = writer_channel();
        let handle = service
            .open(
                1,
                EventStreamOpen {
                    after_seq: 0,
                    event_filter: Vec::new(),
                },
                writer.clone(),
            )
            .expect("open event stream");

        drop(handle);
        wait_for_subscriber_count(&backend, 0).await;

        assert!(!writer.is_closed());
        drop(writer);
        drop(driver);
    }

    #[tokio::test]
    async fn handle_reports_finished_after_server_resets_producer() {
        let backend = Arc::new(FakeBackend::with_events([event(5, "session.updated")]));
        let service = EventStreamService::new(backend.clone());
        let (writer, driver) = writer_channel();
        let handle = service
            .open(
                1,
                EventStreamOpen {
                    after_seq: 9,
                    event_filter: Vec::new(),
                },
                writer.clone(),
            )
            .expect("open event stream");
        wait_for_subscriber_count(&backend, 0).await;

        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            while !handle.is_finished() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("server-reset producer must finish");

        handle.close();
        drop(writer);
        drop(driver);
    }

    #[derive(Clone)]
    struct FakeBackend {
        inner: Arc<FakeBackendInner>,
    }

    struct FakeBackendInner {
        events: Mutex<VecDeque<RuntimeEvent>>,
        replay_gate: Mutex<Option<ReplayGate>>,
        replay_race: Mutex<Option<RuntimeEvent>>,
        sender: broadcast::Sender<RuntimeEvent>,
    }

    struct ReplayGate {
        entered: Arc<Notify>,
        release: Arc<Notify>,
    }

    impl FakeBackend {
        fn with_events(events: impl IntoIterator<Item = RuntimeEvent>) -> Self {
            let (sender, _) = broadcast::channel(EVENT_REPLAY_CAPACITY);
            let mut events = events.into_iter().collect::<VecDeque<_>>();
            while events.len() > EVENT_REPLAY_CAPACITY {
                events.pop_front();
            }
            Self {
                inner: Arc::new(FakeBackendInner {
                    events: Mutex::new(events),
                    replay_gate: Mutex::new(None),
                    replay_race: Mutex::new(None),
                    sender,
                }),
            }
        }

        fn with_replay_race(
            events: impl IntoIterator<Item = RuntimeEvent>,
            raced_event: RuntimeEvent,
        ) -> Self {
            let backend = Self::with_events(events);
            *backend.inner.replay_race.lock().expect("replay race") = Some(raced_event);
            backend
        }

        fn with_blocked_replay(
            events: impl IntoIterator<Item = RuntimeEvent>,
        ) -> (Self, Arc<Notify>, Arc<Notify>) {
            let backend = Self::with_events(events);
            let entered = Arc::new(Notify::new());
            let release = Arc::new(Notify::new());
            *backend.inner.replay_gate.lock().expect("replay gate") = Some(ReplayGate {
                entered: entered.clone(),
                release: release.clone(),
            });
            (backend, entered, release)
        }

        fn replay_after(&self, after_seq: u64) -> EventReplay {
            if let Some(event) = self.inner.replay_race.lock().expect("replay race").take() {
                assert_eq!(
                    self.inner.sender.receiver_count(),
                    1,
                    "live receiver must be subscribed before replay"
                );
                self.inner
                    .events
                    .lock()
                    .expect("fake event ring")
                    .push_back(event.clone());
                let _ = self.inner.sender.send(event);
            }
            let events = self.inner.events.lock().expect("fake event ring");
            EventReplay {
                oldest_seq: events.front().map_or(0, |event| event.seq),
                latest_seq: events.back().map_or(0, |event| event.seq),
                events: events
                    .iter()
                    .filter(|event| event.seq > after_seq)
                    .cloned()
                    .collect(),
            }
        }

        fn publish(&self, event: RuntimeEvent) {
            let _ = self.inner.sender.send(event);
        }

        fn publish_retained(&self, event: RuntimeEvent) {
            let mut events = self.inner.events.lock().expect("fake event ring");
            events.push_back(event.clone());
            if events.len() > EVENT_REPLAY_CAPACITY {
                events.pop_front();
            }
            drop(events);
            let _ = self.inner.sender.send(event);
        }

        fn subscriber_count(&self) -> usize {
            self.inner.sender.receiver_count()
        }
    }

    impl EventBackend for FakeBackend {
        fn replay(&self, after_seq: u64) -> EventReplayFuture<'_> {
            let replay = self.replay_after(after_seq);
            let gate = self.inner.replay_gate.lock().expect("replay gate").take();
            Box::pin(async move {
                if let Some(gate) = gate {
                    gate.entered.notify_one();
                    gate.release.notified().await;
                }
                replay
            })
        }

        fn subscribe(&self) -> broadcast::Receiver<RuntimeEvent> {
            self.inner.sender.subscribe()
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

    fn decode_event(frame: &Frame) -> RuntimeEvent {
        assert_eq!(frame.header.kind, FrameKind::Event);
        serde_json::from_slice(&frame.payload).expect("event payload")
    }

    async fn assert_cursor_ahead_resets(
        events: impl IntoIterator<Item = RuntimeEvent>,
        after_seq: u64,
        expected_latest_seq: u64,
    ) {
        let backend = Arc::new(FakeBackend::with_events(events));
        let service = EventStreamService::new(backend);
        let (writer, driver) = writer_channel();
        let handle = service
            .open(
                1,
                EventStreamOpen {
                    after_seq,
                    event_filter: Vec::new(),
                },
                writer.clone(),
            )
            .expect("open event stream");
        let (socket, mut peer) = tokio::io::duplex(64 * 1024);
        let driver_task = tokio::spawn(driver.run(socket));

        let opened = read_frame(&mut peer).await;
        let reset = tokio::time::timeout(std::time::Duration::from_secs(1), read_frame(&mut peer))
            .await
            .expect("cursor ahead of daemon must reset stream");
        let payload: StreamReset =
            serde_json::from_slice(&reset.payload).expect("stream reset payload");

        assert_eq!(opened.header.kind, FrameKind::StreamOpened);
        assert_eq!(reset.header.kind, FrameKind::StreamReset);
        assert_eq!(payload.reason, StreamResetReason::EventGap);
        assert_eq!(payload.latest_seq, Some(expected_latest_seq));

        handle.close();
        drop(writer);
        driver_task
            .await
            .expect("writer task")
            .expect("writer result");
    }

    async fn wait_for_subscriber_count(backend: &FakeBackend, expected: usize) {
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            while backend.subscriber_count() != expected {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("subscriber count");
    }

    async fn wait_for_store_subscriber_count(store: &EventStore, expected: usize) {
        tokio::time::timeout(Duration::from_secs(1), async {
            while store.sender.receiver_count() != expected {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("store subscriber count");
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
