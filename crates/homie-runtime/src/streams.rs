use std::sync::Arc;

use homie_proto::stream::{StreamKind, StreamOpenRequest, StreamResetReason};
use homie_proto::transport::{Frame, FrameKind};

use crate::capabilities::stream_capabilities;
use crate::connection::{
    ActiveStream, ActiveStreamFuture, StreamError, StreamFuture, StreamHandler,
};
use crate::event_stream::{EventBounds, EventStore, EventStreamHandle, EventStreamService};
use crate::terminal_stream::{
    TerminalBackend, TerminalSourceManager, TerminalSourceStats, TerminalStreamError,
    TerminalSubscription,
};
use crate::writer::WriterHandle;

pub struct RuntimeStreamHandler<B> {
    event_store: Arc<EventStore>,
    events: EventStreamService,
    terminals: TerminalSourceManager<B>,
}

impl<B: TerminalBackend> RuntimeStreamHandler<B> {
    #[must_use]
    pub fn new(event_store: Arc<EventStore>, terminal_backend: Arc<B>) -> Self {
        Self {
            events: EventStreamService::new(event_store.clone()),
            event_store,
            terminals: TerminalSourceManager::new(terminal_backend),
        }
    }

    pub async fn terminal_stats(&self) -> TerminalSourceStats {
        self.terminals.stats().await
    }
}

impl<B: TerminalBackend> StreamHandler for RuntimeStreamHandler<B> {
    fn capabilities(&self) -> Vec<StreamKind> {
        stream_capabilities().to_vec()
    }

    fn event_bounds(&self) -> EventBounds {
        self.event_store.bounds()
    }

    fn open<'a>(
        &'a self,
        stream_id: u32,
        request: StreamOpenRequest,
        writer: WriterHandle,
    ) -> StreamFuture<'a> {
        Box::pin(async move {
            match request {
                StreamOpenRequest::Events(request) => {
                    let handle = self.events.open(stream_id, request, writer)?;
                    Ok(Box::new(ActiveEventStream(handle)) as Box<dyn ActiveStream>)
                }
                StreamOpenRequest::Terminal(request) => {
                    let cleanup_writer = writer.clone();
                    let subscription = self
                        .terminals
                        .open(stream_id, request, writer)
                        .await
                        .map_err(|error| {
                            cleanup_writer.reset_stream(stream_id);
                            map_terminal_error(error)
                        })?;
                    Ok(Box::new(ActiveTerminalStream(subscription)) as Box<dyn ActiveStream>)
                }
            }
        })
    }
}

struct ActiveEventStream(EventStreamHandle);

impl ActiveStream for ActiveEventStream {
    fn is_finished(&self) -> bool {
        self.0.is_finished()
    }

    fn handle<'a>(&'a mut self, _frame: Frame) -> ActiveStreamFuture<'a> {
        Box::pin(async { Err(StreamError::Protocol) })
    }

    fn close(self: Box<Self>) -> ActiveStreamFuture<'static> {
        Box::pin(async move {
            self.0.close();
            Ok(())
        })
    }
}

struct ActiveTerminalStream(TerminalSubscription);

impl ActiveStream for ActiveTerminalStream {
    fn is_finished(&self) -> bool {
        self.0.is_finished()
    }

    fn handle<'a>(&'a mut self, frame: Frame) -> ActiveStreamFuture<'a> {
        Box::pin(async move {
            if !matches!(frame.header.kind, FrameKind::Input | FrameKind::Resize) {
                return Err(StreamError::Protocol);
            }
            self.0.handle_frame(frame).await.map_err(map_terminal_error)
        })
    }

    fn close(self: Box<Self>) -> ActiveStreamFuture<'static> {
        Box::pin(async move { self.0.close().await.map_err(map_terminal_error) })
    }
}

fn map_terminal_error(error: TerminalStreamError) -> StreamError {
    match error {
        TerminalStreamError::InvalidFrame => StreamError::Protocol,
        TerminalStreamError::Backpressure => StreamError::Reset(StreamResetReason::SlowConsumer),
        TerminalStreamError::SlowConsumer => StreamError::ResetSent,
        TerminalStreamError::Writer(error) => StreamError::Writer(error),
        TerminalStreamError::Backend
        | TerminalStreamError::InvalidDescriptor
        | TerminalStreamError::SourceUnavailable
        | TerminalStreamError::Io(_)
        | TerminalStreamError::SequenceOverflow => {
            StreamError::Reset(StreamResetReason::ProtocolError)
        }
    }
}
