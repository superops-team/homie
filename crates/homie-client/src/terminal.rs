use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};

use homie_proto::grid::{GridUpdate, terminal_cell_count};
use homie_proto::transport::{Frame, FrameHeader, FrameKind, WIRE_MAJOR};
use tokio::sync::{mpsc, watch};

use crate::client::{ClientError, ClientInner};
use crate::streams::StreamState;

const MAX_INPUT_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminalItem {
    ReplayBegin(u64),
    Output { offset: u64, bytes: Vec<u8> },
    ReplayEnd(u64),
    Grid(GridUpdate),
    Modes(Vec<u8>),
}

pub struct TerminalStream {
    pub(crate) stream_id: u32,
    pub(crate) receiver: mpsc::Receiver<TerminalItem>,
    pub(crate) state: watch::Receiver<StreamState>,
    pub(crate) last_confirmed_offset: Arc<AtomicU64>,
    pub(crate) inner: Weak<ClientInner>,
}

impl TerminalStream {
    pub async fn recv(&mut self) -> Result<Option<TerminalItem>, ClientError> {
        Ok(self.receiver.recv().await)
    }

    #[must_use]
    pub fn state(&self) -> watch::Receiver<StreamState> {
        self.state.clone()
    }

    #[must_use]
    pub fn last_confirmed_offset(&self) -> u64 {
        self.last_confirmed_offset.load(Ordering::Acquire)
    }

    pub fn send_input(&self, input: impl Into<Vec<u8>>) -> Result<(), ClientError> {
        let input = input.into();
        if input.len() > MAX_INPUT_BYTES {
            return Err(ClientError::BadRequest(
                "terminal input must not exceed 64 KiB".to_string(),
            ));
        }
        self.send_control_frame(FrameKind::Input, input)
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), ClientError> {
        if terminal_cell_count(cols, rows).is_none() {
            return Err(ClientError::BadRequest(
                "terminal dimensions exceed protocol limits".to_string(),
            ));
        }
        let mut payload = Vec::with_capacity(4);
        payload.extend_from_slice(&cols.to_be_bytes());
        payload.extend_from_slice(&rows.to_be_bytes());
        self.send_control_frame(FrameKind::Resize, payload)
    }

    fn send_control_frame(&self, kind: FrameKind, payload: Vec<u8>) -> Result<(), ClientError> {
        if *self.state.borrow() != StreamState::Open {
            return Err(ClientError::Unavailable);
        }
        let inner = self.inner.upgrade().ok_or(ClientError::Unavailable)?;
        let writer = inner.writer().ok_or(ClientError::Unavailable)?;
        writer.try_send_high(Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind,
                flags: 0,
                stream_id: self.stream_id,
                message_id: 0,
                sequence: 0,
            },
            payload,
        })?;
        Ok(())
    }
}

impl Drop for TerminalStream {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            inner.close_stream(self.stream_id);
        }
    }
}
