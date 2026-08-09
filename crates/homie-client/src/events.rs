use std::sync::Weak;

use homie_proto::model::{RuntimeEvent, StateSnapshot};
use tokio::sync::{mpsc, watch};

use crate::client::{ClientError, ClientInner};
use crate::streams::StreamState;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventStreamItem {
    Event(RuntimeEvent),
    Snapshot(StateSnapshot),
}

pub struct EventStream {
    pub(crate) stream_id: u32,
    pub(crate) receiver: mpsc::Receiver<EventStreamItem>,
    pub(crate) state: watch::Receiver<StreamState>,
    pub(crate) inner: Weak<ClientInner>,
}

impl EventStream {
    pub async fn recv(&mut self) -> Result<Option<EventStreamItem>, ClientError> {
        Ok(self.receiver.recv().await)
    }

    #[must_use]
    pub fn state(&self) -> watch::Receiver<StreamState> {
        self.state.clone()
    }
}

impl Drop for EventStream {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            inner.close_stream(self.stream_id);
        }
    }
}
