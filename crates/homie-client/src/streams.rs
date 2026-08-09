use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

use homie_proto::grid::{
    ChangedRow, GridCell, GridUpdate, TermColor, TermStyle, terminal_cell_count,
};
use homie_proto::model::{RuntimeEvent, StateSnapshot};
use homie_proto::stream::{EventStreamOpen, StreamOpenRequest, StreamReset, TerminalStreamOpen};
use homie_proto::transport::{Frame, FrameHeader, FrameKind, WIRE_MAJOR};
use tokio::sync::{mpsc, oneshot, watch};

use crate::client::{ClientError, ClientInner};
use crate::events::{EventStream, EventStreamItem};
use crate::terminal::{TerminalItem, TerminalStream};
use crate::writer::WriterHandle;

const MAX_STREAMS: usize = 64;
const DECODED_QUEUE_CAPACITY: usize = 256;

type OpenReceiver = oneshot::Receiver<Result<(), ClientError>>;
type EventRegistration = (EventStream, OpenReceiver, Frame);
type TerminalRegistration = (TerminalStream, OpenReceiver, Frame);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamState {
    Opening,
    Open,
    Reconnecting,
    ResyncRequired { last_confirmed_offset: Option<u64> },
    Closed,
}

pub(crate) enum StreamAction {
    None,
    RecoverEvent(u32),
    ReopenTerminal(u32),
}

#[derive(Default)]
pub(crate) struct StreamRegistry {
    next_stream_id: AtomicU32,
    streams: Mutex<HashMap<u32, Descriptor>>,
}

enum Descriptor {
    Event(EventDescriptor),
    Terminal(TerminalDescriptor),
}

struct EventDescriptor {
    request: EventStreamOpen,
    sender: mpsc::Sender<EventStreamItem>,
    state: watch::Sender<StreamState>,
    opened: Option<oneshot::Sender<Result<(), ClientError>>>,
    last_sequence: u64,
    recovering: bool,
    close_remote_before_recovery: bool,
    closed: bool,
}

struct TerminalDescriptor {
    request: TerminalStreamOpen,
    sender: mpsc::Sender<TerminalItem>,
    state: watch::Sender<StreamState>,
    opened: Option<oneshot::Sender<Result<(), ClientError>>>,
    last_sequence: u64,
    awaiting_full_grid: bool,
    last_confirmed_offset: Arc<AtomicU64>,
    inner: Weak<ClientInner>,
    closed: bool,
}

enum DispatchOutcome {
    Action(StreamAction),
    Remove,
    RemoveAndClose(Weak<ClientInner>),
    Send(Weak<ClientInner>, Frame),
}

impl StreamRegistry {
    pub(crate) fn insert_event(
        &self,
        inner: &Arc<ClientInner>,
        request: EventStreamOpen,
    ) -> Result<EventRegistration, ClientError> {
        let stream_id = self.allocate_id()?;
        let (sender, receiver) = mpsc::channel(DECODED_QUEUE_CAPACITY);
        let (state, state_rx) = watch::channel(StreamState::Opening);
        let (opened, opened_rx) = oneshot::channel();
        let frame = stream_open_frame(stream_id, &StreamOpenRequest::Events(request.clone()))?;
        self.streams
            .lock()
            .expect("stream registry lock poisoned")
            .insert(
                stream_id,
                Descriptor::Event(EventDescriptor {
                    last_sequence: request.after_seq,
                    request,
                    sender,
                    state,
                    opened: Some(opened),
                    closed: false,
                    recovering: false,
                    close_remote_before_recovery: false,
                }),
            );
        Ok((
            EventStream {
                stream_id,
                receiver,
                state: state_rx,
                inner: Arc::downgrade(inner),
            },
            opened_rx,
            frame,
        ))
    }

    pub(crate) fn insert_terminal(
        &self,
        inner: &Arc<ClientInner>,
        request: TerminalStreamOpen,
    ) -> Result<TerminalRegistration, ClientError> {
        let stream_id = self.allocate_id()?;
        let (sender, receiver) = mpsc::channel(DECODED_QUEUE_CAPACITY);
        let (state, state_rx) = watch::channel(StreamState::Opening);
        let (opened, opened_rx) = oneshot::channel();
        let last_confirmed_offset = Arc::new(AtomicU64::new(request.output_offset));
        let frame = stream_open_frame(stream_id, &StreamOpenRequest::Terminal(request.clone()))?;
        self.streams
            .lock()
            .expect("stream registry lock poisoned")
            .insert(
                stream_id,
                Descriptor::Terminal(TerminalDescriptor {
                    request,
                    sender,
                    state,
                    opened: Some(opened),
                    last_sequence: 0,
                    awaiting_full_grid: true,
                    last_confirmed_offset: last_confirmed_offset.clone(),
                    inner: Arc::downgrade(inner),
                    closed: false,
                }),
            );
        Ok((
            TerminalStream {
                stream_id,
                receiver,
                state: state_rx,
                last_confirmed_offset,
                inner: Arc::downgrade(inner),
            },
            opened_rx,
            frame,
        ))
    }

    pub(crate) fn remove(&self, stream_id: u32) -> bool {
        if let Some(descriptor) = self
            .streams
            .lock()
            .expect("stream registry lock poisoned")
            .remove(&stream_id)
        {
            match descriptor {
                Descriptor::Event(descriptor) => {
                    descriptor.state.send_replace(StreamState::Closed);
                }
                Descriptor::Terminal(descriptor) => {
                    descriptor.state.send_replace(StreamState::Closed);
                }
            }
            true
        } else {
            false
        }
    }

    pub(crate) fn close_all(&self) -> Vec<u32> {
        let streams =
            std::mem::take(&mut *self.streams.lock().expect("stream registry lock poisoned"));
        streams
            .into_iter()
            .map(|(stream_id, descriptor)| {
                match descriptor {
                    Descriptor::Event(descriptor) => {
                        descriptor.state.send_replace(StreamState::Closed);
                    }
                    Descriptor::Terminal(descriptor) => {
                        descriptor.state.send_replace(StreamState::Closed);
                    }
                }
                stream_id
            })
            .collect()
    }

    pub(crate) fn connection_lost(&self) {
        for descriptor in self
            .streams
            .lock()
            .expect("stream registry lock poisoned")
            .values_mut()
        {
            match descriptor {
                Descriptor::Event(descriptor) if !descriptor.closed => {
                    descriptor.state.send_replace(StreamState::Reconnecting);
                }
                Descriptor::Terminal(descriptor) if !descriptor.closed => {
                    descriptor.awaiting_full_grid = true;
                    descriptor.state.send_replace(StreamState::Reconnecting);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn reopen_all(&self, writer: &WriterHandle) -> Result<(), ClientError> {
        let mut streams = self.streams.lock().expect("stream registry lock poisoned");
        for (stream_id, descriptor) in streams.iter_mut() {
            let request = match descriptor {
                Descriptor::Event(descriptor) if !descriptor.closed => {
                    descriptor.request.after_seq = descriptor.last_sequence;
                    descriptor.state.send_replace(StreamState::Reconnecting);
                    StreamOpenRequest::Events(descriptor.request.clone())
                }
                Descriptor::Terminal(descriptor) if !descriptor.closed => {
                    descriptor.request.output_offset =
                        descriptor.last_confirmed_offset.load(Ordering::Acquire);
                    descriptor.request.last_grid_sequence = None;
                    descriptor.last_sequence = 0;
                    descriptor.awaiting_full_grid = true;
                    descriptor.state.send_replace(StreamState::Reconnecting);
                    StreamOpenRequest::Terminal(descriptor.request.clone())
                }
                _ => continue,
            };
            writer.try_send_high(stream_open_frame(*stream_id, &request)?)?;
        }
        Ok(())
    }

    pub(crate) fn dispatch(&self, frame: Frame) -> Result<StreamAction, ClientError> {
        let stream_id = frame.header.stream_id;
        let mut streams = self.streams.lock().expect("stream registry lock poisoned");
        let Some(descriptor) = streams.get_mut(&stream_id) else {
            return Ok(StreamAction::None);
        };
        let outcome = match descriptor {
            Descriptor::Event(descriptor) => dispatch_event(descriptor, frame),
            Descriptor::Terminal(descriptor) => dispatch_terminal(descriptor, frame),
        }?;
        match outcome {
            DispatchOutcome::Action(action) => Ok(action),
            DispatchOutcome::Remove => {
                streams.remove(&stream_id);
                Ok(StreamAction::None)
            }
            DispatchOutcome::RemoveAndClose(inner) => {
                streams.remove(&stream_id);
                drop(streams);
                if let Some(writer) = inner.upgrade().and_then(|inner| inner.writer()) {
                    writer.close_stream(stream_id);
                    writer.try_send_high(stream_close_frame(stream_id))?;
                }
                Ok(StreamAction::None)
            }
            DispatchOutcome::Send(inner, frame) => {
                drop(streams);
                if let Some(writer) = inner.upgrade().and_then(|inner| inner.writer()) {
                    writer.try_send_high(frame)?;
                }
                Ok(StreamAction::None)
            }
        }
    }

    pub(crate) fn take_event_recovery_close(&self, stream_id: u32) -> Option<bool> {
        let mut streams = self.streams.lock().expect("stream registry lock poisoned");
        let Some(Descriptor::Event(descriptor)) = streams.get_mut(&stream_id) else {
            return None;
        };
        if !descriptor.recovering {
            return None;
        }
        Some(std::mem::take(&mut descriptor.close_remote_before_recovery))
    }

    pub(crate) fn event_recovery_active(&self, stream_id: u32) -> bool {
        matches!(
            self.streams
                .lock()
                .expect("stream registry lock poisoned")
                .get(&stream_id),
            Some(Descriptor::Event(EventDescriptor {
                recovering: true,
                ..
            }))
        )
    }

    pub(crate) async fn complete_event_recovery(
        &self,
        stream_id: u32,
        snapshot: StateSnapshot,
    ) -> Result<Frame, ClientError> {
        let sender = {
            let streams = self.streams.lock().expect("stream registry lock poisoned");
            let Some(Descriptor::Event(descriptor)) = streams.get(&stream_id) else {
                return Err(ClientError::ResyncRequired);
            };
            if !descriptor.recovering {
                return Err(ClientError::ResyncRequired);
            }
            descriptor.sender.clone()
        };
        sender
            .send(EventStreamItem::Snapshot(snapshot.clone()))
            .await
            .map_err(|_| ClientError::ResyncRequired)?;

        let mut streams = self.streams.lock().expect("stream registry lock poisoned");
        let Some(Descriptor::Event(descriptor)) = streams.get_mut(&stream_id) else {
            return Err(ClientError::ResyncRequired);
        };
        if !descriptor.recovering {
            return Err(ClientError::ResyncRequired);
        }
        descriptor.request.after_seq = snapshot.event_cursor;
        descriptor.last_sequence = snapshot.event_cursor;
        descriptor.state.send_replace(StreamState::Reconnecting);
        stream_open_frame(
            stream_id,
            &StreamOpenRequest::Events(descriptor.request.clone()),
        )
    }

    pub(crate) fn reopen_terminal_frame(&self, stream_id: u32) -> Result<Frame, ClientError> {
        let mut streams = self.streams.lock().expect("stream registry lock poisoned");
        let Some(Descriptor::Terminal(descriptor)) = streams.get_mut(&stream_id) else {
            return Err(ClientError::ResyncRequired);
        };
        prepare_terminal_reopen(stream_id, descriptor)
    }

    fn allocate_id(&self) -> Result<u32, ClientError> {
        let streams = self.streams.lock().expect("stream registry lock poisoned");
        if streams.len() >= MAX_STREAMS {
            return Err(ClientError::Backpressure);
        }
        drop(streams);
        loop {
            let current = self.next_stream_id.fetch_add(2, Ordering::Relaxed);
            let stream_id = if current == 0 { 1 } else { current | 1 };
            if !self
                .streams
                .lock()
                .expect("stream registry lock poisoned")
                .contains_key(&stream_id)
            {
                return Ok(stream_id);
            }
        }
    }
}

fn dispatch_event(
    descriptor: &mut EventDescriptor,
    frame: Frame,
) -> Result<DispatchOutcome, ClientError> {
    if descriptor.closed {
        return Ok(DispatchOutcome::Action(StreamAction::None));
    }
    match frame.header.kind {
        FrameKind::StreamOpened => {
            descriptor.recovering = false;
            descriptor.close_remote_before_recovery = false;
            descriptor.state.send_replace(StreamState::Open);
            if let Some(opened) = descriptor.opened.take() {
                let _ = opened.send(Ok(()));
            }
        }
        FrameKind::Event => {
            if descriptor.recovering {
                return Ok(DispatchOutcome::Action(StreamAction::None));
            }
            if frame.header.sequence <= descriptor.last_sequence {
                return Ok(start_event_recovery(
                    descriptor,
                    frame.header.stream_id,
                    true,
                ));
            }
            let event: RuntimeEvent = serde_json::from_slice(&frame.payload)?;
            if event.seq != frame.header.sequence {
                return Err(ClientError::Protocol(
                    "event payload sequence did not match frame".to_string(),
                ));
            }
            if descriptor
                .sender
                .try_send(EventStreamItem::Event(event))
                .is_err()
            {
                descriptor.state.send_replace(StreamState::ResyncRequired {
                    last_confirmed_offset: None,
                });
                return Ok(start_event_recovery(
                    descriptor,
                    frame.header.stream_id,
                    true,
                ));
            }
            descriptor.last_sequence = frame.header.sequence;
        }
        FrameKind::StreamReset => {
            let _reset: StreamReset = serde_json::from_slice(&frame.payload)?;
            if descriptor.recovering {
                return Ok(DispatchOutcome::Action(StreamAction::None));
            }
            return Ok(start_event_recovery(
                descriptor,
                frame.header.stream_id,
                false,
            ));
        }
        FrameKind::StreamClose => {
            descriptor.closed = true;
            descriptor.state.send_replace(StreamState::Closed);
            return Ok(DispatchOutcome::Remove);
        }
        _ => {
            return Err(ClientError::Protocol(
                "invalid frame for event stream".to_string(),
            ));
        }
    }
    Ok(DispatchOutcome::Action(StreamAction::None))
}

fn start_event_recovery(
    descriptor: &mut EventDescriptor,
    stream_id: u32,
    close_remote: bool,
) -> DispatchOutcome {
    descriptor.recovering = true;
    descriptor.close_remote_before_recovery = close_remote;
    descriptor.state.send_replace(StreamState::ResyncRequired {
        last_confirmed_offset: None,
    });
    DispatchOutcome::Action(StreamAction::RecoverEvent(stream_id))
}

fn dispatch_terminal(
    descriptor: &mut TerminalDescriptor,
    frame: Frame,
) -> Result<DispatchOutcome, ClientError> {
    if descriptor.closed {
        return Ok(DispatchOutcome::Action(StreamAction::None));
    }
    if frame.header.kind == FrameKind::StreamReset {
        let _reset: StreamReset = serde_json::from_slice(&frame.payload)?;
        let last_confirmed_offset = descriptor.last_confirmed_offset.load(Ordering::Acquire);
        descriptor.state.send_replace(StreamState::ResyncRequired {
            last_confirmed_offset: Some(last_confirmed_offset),
        });
        let reopen = prepare_terminal_reopen(frame.header.stream_id, descriptor)?;
        return Ok(DispatchOutcome::Send(descriptor.inner.clone(), reopen));
    }
    if frame.header.kind != FrameKind::StreamOpened {
        if descriptor.last_sequence.checked_add(1) != Some(frame.header.sequence) {
            return Ok(DispatchOutcome::Action(StreamAction::ReopenTerminal(
                frame.header.stream_id,
            )));
        }
        descriptor.last_sequence = frame.header.sequence;
    }
    let mut confirmed_offset = None;
    let item = match frame.header.kind {
        FrameKind::StreamOpened => {
            descriptor.last_sequence = 0;
            descriptor.state.send_replace(StreamState::Open);
            if let Some(opened) = descriptor.opened.take() {
                let _ = opened.send(Ok(()));
            }
            return Ok(DispatchOutcome::Action(StreamAction::None));
        }
        FrameKind::ReplayBegin => {
            let offset = read_u64(&frame.payload)?;
            confirmed_offset = Some(offset);
            TerminalItem::ReplayBegin(offset)
        }
        FrameKind::Output => {
            let offset = read_u64(&frame.payload)?;
            let bytes = frame.payload[8..].to_vec();
            let expected = descriptor.last_confirmed_offset.load(Ordering::Acquire);
            if offset != expected {
                return Ok(DispatchOutcome::Action(StreamAction::ReopenTerminal(
                    frame.header.stream_id,
                )));
            }
            confirmed_offset =
                Some(offset.checked_add(bytes.len() as u64).ok_or_else(|| {
                    ClientError::Protocol("terminal offset overflow".to_string())
                })?);
            TerminalItem::Output { offset, bytes }
        }
        FrameKind::ReplayEnd => TerminalItem::ReplayEnd(read_u64(&frame.payload)?),
        FrameKind::Grid => {
            let update = decode_grid(&frame.payload)?;
            if descriptor.awaiting_full_grid && !update.is_full_snapshot {
                return Ok(DispatchOutcome::Action(StreamAction::None));
            }
            if update.is_full_snapshot {
                descriptor.awaiting_full_grid = false;
                descriptor.request.last_grid_sequence = Some(frame.header.sequence);
            }
            TerminalItem::Grid(update)
        }
        FrameKind::Modes => TerminalItem::Modes(frame.payload),
        FrameKind::StreamClose => {
            descriptor.closed = true;
            descriptor.state.send_replace(StreamState::Closed);
            return Ok(DispatchOutcome::Remove);
        }
        _ => {
            return Err(ClientError::Protocol(
                "invalid frame for terminal stream".to_string(),
            ));
        }
    };

    if descriptor.sender.try_send(item).is_err() {
        descriptor.state.send_replace(StreamState::ResyncRequired {
            last_confirmed_offset: Some(descriptor.last_confirmed_offset.load(Ordering::Acquire)),
        });
        return Ok(DispatchOutcome::RemoveAndClose(descriptor.inner.clone()));
    }
    if let Some(offset) = confirmed_offset {
        descriptor
            .last_confirmed_offset
            .store(offset, Ordering::Release);
    }
    Ok(DispatchOutcome::Action(StreamAction::None))
}

fn prepare_terminal_reopen(
    stream_id: u32,
    descriptor: &mut TerminalDescriptor,
) -> Result<Frame, ClientError> {
    descriptor.request.output_offset = descriptor.last_confirmed_offset.load(Ordering::Acquire);
    descriptor.last_sequence = 0;
    descriptor.awaiting_full_grid = true;
    descriptor.state.send_replace(StreamState::Reconnecting);
    stream_open_frame(
        stream_id,
        &StreamOpenRequest::Terminal(descriptor.request.clone()),
    )
}

fn stream_open_frame(stream_id: u32, request: &StreamOpenRequest) -> Result<Frame, ClientError> {
    Ok(Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::StreamOpen,
            flags: 0,
            stream_id,
            message_id: 0,
            sequence: 0,
        },
        payload: serde_json::to_vec(request)?,
    })
}

pub(crate) fn stream_close_frame(stream_id: u32) -> Frame {
    Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::StreamClose,
            flags: 0,
            stream_id,
            message_id: 0,
            sequence: 0,
        },
        payload: Vec::new(),
    }
}

fn read_u64(payload: &[u8]) -> Result<u64, ClientError> {
    let bytes: [u8; 8] = payload
        .get(..8)
        .ok_or_else(|| ClientError::Protocol("stream offset payload is truncated".to_string()))?
        .try_into()
        .map_err(|_| ClientError::Protocol("stream offset payload is invalid".to_string()))?;
    Ok(u64::from_be_bytes(bytes))
}

fn decode_grid(payload: &[u8]) -> Result<GridUpdate, ClientError> {
    let mut cursor = GridCursor { payload, offset: 0 };
    let cols = cursor.u16()?;
    let rows = cursor.u16()?;
    terminal_cell_count(cols, rows).ok_or_else(|| {
        ClientError::Protocol("terminal grid geometry exceeds protocol limits".to_string())
    })?;
    let cursor_col = cursor.u16()?;
    let cursor_row = cursor.u16()?;
    let flags = cursor.u8()?;
    let row_count = cursor.u16()?;
    if row_count > rows {
        return Err(ClientError::Protocol(
            "terminal grid row count exceeds geometry".to_string(),
        ));
    }
    let mut changed_rows = Vec::new();
    changed_rows
        .try_reserve_exact(usize::from(row_count))
        .map_err(|_| ClientError::Protocol("terminal grid allocation failed".to_string()))?;
    for _ in 0..row_count {
        let y = cursor.u16()?;
        let row_cols = cursor.u16()? as usize;
        if y >= rows || row_cols != usize::from(cols) {
            return Err(ClientError::Protocol(
                "terminal grid row geometry is invalid".to_string(),
            ));
        }
        let mut cells = Vec::new();
        cells
            .try_reserve_exact(row_cols)
            .map_err(|_| ClientError::Protocol("terminal grid allocation failed".to_string()))?;
        while cells.len() < row_cols {
            let run = cursor.u8()? as usize;
            let run_end = cells.len().checked_add(run).ok_or_else(|| {
                ClientError::Protocol("terminal grid RLE run overflowed".to_string())
            })?;
            if run == 0 || run_end > row_cols {
                return Err(ClientError::Protocol(
                    "terminal grid RLE run is invalid".to_string(),
                ));
            }
            let scalar = cursor.u32()?;
            let fg = TermColor::unpack(cursor.u32()?);
            let bg = TermColor::unpack(cursor.u32()?);
            let style = TermStyle::from_bits_retain(cursor.u16()?);
            cells.extend(std::iter::repeat_n(
                GridCell::new(scalar, fg, bg, style),
                run,
            ));
        }
        changed_rows.push(ChangedRow::new(y, cells));
    }
    if cursor.offset != payload.len() {
        return Err(ClientError::Protocol(
            "terminal grid payload has trailing bytes".to_string(),
        ));
    }
    Ok(GridUpdate {
        cols,
        rows,
        cursor_col,
        cursor_row,
        cursor_visible: flags & 1 != 0,
        is_full_snapshot: flags & 2 != 0,
        changed_rows,
    })
}

struct GridCursor<'a> {
    payload: &'a [u8],
    offset: usize,
}

impl GridCursor<'_> {
    fn u8(&mut self) -> Result<u8, ClientError> {
        let value = *self
            .payload
            .get(self.offset)
            .ok_or_else(|| ClientError::Protocol("terminal grid is truncated".to_string()))?;
        self.offset += 1;
        Ok(value)
    }

    fn u16(&mut self) -> Result<u16, ClientError> {
        let bytes = self.take::<2>()?;
        Ok(u16::from_be_bytes(bytes))
    }

    fn u32(&mut self) -> Result<u32, ClientError> {
        let bytes = self.take::<4>()?;
        Ok(u32::from_be_bytes(bytes))
    }

    fn take<const N: usize>(&mut self) -> Result<[u8; N], ClientError> {
        let end = self
            .offset
            .checked_add(N)
            .ok_or_else(|| ClientError::Protocol("terminal grid is invalid".to_string()))?;
        let bytes = self
            .payload
            .get(self.offset..end)
            .ok_or_else(|| ClientError::Protocol("terminal grid is truncated".to_string()))?;
        self.offset = end;
        bytes
            .try_into()
            .map_err(|_| ClientError::Protocol("terminal grid is invalid".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use homie_proto::grid::{MAX_TERMINAL_CELLS, MAX_TERMINAL_COLS};
    use homie_proto::stream::StreamResetReason;
    use homie_proto::transport::ClientRole;

    use super::*;

    #[test]
    fn decode_grid_rejects_geometry_over_shared_cell_limit_before_allocating_rows() {
        let rows = u16::try_from(MAX_TERMINAL_CELLS / usize::from(MAX_TERMINAL_COLS) + 1)
            .expect("test rows fit u16");
        let mut payload = Vec::new();
        payload.extend_from_slice(&MAX_TERMINAL_COLS.to_be_bytes());
        payload.extend_from_slice(&rows.to_be_bytes());
        payload.extend_from_slice(&0_u16.to_be_bytes());
        payload.extend_from_slice(&0_u16.to_be_bytes());
        payload.push(2);
        payload.extend_from_slice(&0_u16.to_be_bytes());

        let error = decode_grid(&payload).expect_err("oversized grid must fail");

        assert!(matches!(error, ClientError::Protocol(_)));
    }

    #[test]
    fn terminal_reset_with_lower_server_offset_preserves_local_cursor() {
        assert_terminal_reset_preserves_local_cursor(Some(7));
    }

    #[test]
    fn terminal_reset_with_higher_server_offset_preserves_local_cursor() {
        assert_terminal_reset_preserves_local_cursor(Some(99));
    }

    #[test]
    fn terminal_reset_without_server_offset_preserves_local_cursor() {
        assert_terminal_reset_preserves_local_cursor(None);
    }

    fn assert_terminal_reset_preserves_local_cursor(server_offset: Option<u64>) {
        const STREAM_ID: u32 = 1;
        const LOCAL_OFFSET: u64 = 13;

        let (sender, _receiver) = mpsc::channel(1);
        let (state, state_rx) = watch::channel(StreamState::Open);
        let registry = StreamRegistry::default();
        registry
            .streams
            .lock()
            .expect("stream registry lock poisoned")
            .insert(
                STREAM_ID,
                Descriptor::Terminal(TerminalDescriptor {
                    request: TerminalStreamOpen {
                        session_id: "session-1".to_string(),
                        output_offset: 0,
                        client_role: ClientRole::Cli,
                        last_grid_sequence: None,
                    },
                    sender,
                    state,
                    opened: None,
                    last_sequence: 0,
                    awaiting_full_grid: false,
                    last_confirmed_offset: Arc::new(AtomicU64::new(LOCAL_OFFSET)),
                    inner: Weak::new(),
                    closed: false,
                }),
            );
        let action = registry
            .dispatch(Frame {
                header: FrameHeader {
                    version: WIRE_MAJOR,
                    kind: FrameKind::StreamReset,
                    flags: 0,
                    stream_id: STREAM_ID,
                    message_id: 0,
                    sequence: 0,
                },
                payload: serde_json::to_vec(&StreamReset {
                    reason: StreamResetReason::ResyncRequired,
                    last_confirmed_offset: server_offset,
                    latest_seq: None,
                })
                .expect("encode reset"),
            })
            .expect("dispatch reset");

        assert!(matches!(action, StreamAction::None));
        let local_cursor = {
            let streams = registry
                .streams
                .lock()
                .expect("stream registry lock poisoned");
            let Some(Descriptor::Terminal(descriptor)) = streams.get(&STREAM_ID) else {
                panic!("terminal descriptor");
            };
            descriptor.last_confirmed_offset.load(Ordering::Acquire)
        };
        assert_eq!(
            (local_cursor, state_rx.borrow().clone()),
            (LOCAL_OFFSET, StreamState::Reconnecting)
        );

        let reopened = registry
            .reopen_terminal_frame(STREAM_ID)
            .expect("reopen terminal");
        let request: StreamOpenRequest =
            serde_json::from_slice(&reopened.payload).expect("decode terminal reopen");
        assert!(matches!(
            request,
            StreamOpenRequest::Terminal(TerminalStreamOpen {
                output_offset: LOCAL_OFFSET,
                ..
            })
        ));
    }
}
