//! Terminal attachment lifecycle: daemon connection, command channel, and the
//! retry/backoff loop that keeps a session resident.

use std::path::PathBuf;

use homie_client::attachment::SessionAttachment;
use homie_proto::SessionId;
use tokio::runtime::Handle;
use tokio::sync::mpsc;

use super::{PaneEvent, REATTACH_DELAY};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AttachmentState {
    Attaching,
    Live,
    Reconnecting,
}

pub(crate) enum AttachmentCommand {
    Input(Vec<u8>),
    Resize(u16, u16),
    Scroll {
        direction: u8,
        lines: u16,
        col: u16,
        row: u16,
    },
    Close,
}

#[derive(Clone)]
pub(crate) struct AttachmentControl {
    tx: mpsc::UnboundedSender<AttachmentCommand>,
    pane_tx: mpsc::UnboundedSender<PaneEvent>,
}

impl AttachmentControl {
    pub(crate) fn input(&self, bytes: Vec<u8>) {
        if bytes.is_empty() {
            return;
        }
        // Queue the priority marker before the bytes leave for the daemon, so
        // an echo that returns immediately cannot land behind the UI's
        // background-output repaint timer.
        let _ = self.pane_tx.send(PaneEvent::InteractiveInput);
        let _ = self.tx.send(AttachmentCommand::Input(bytes));
    }

    pub(crate) fn resize(&self, cols: u16, rows: u16) {
        let _ = self.tx.send(AttachmentCommand::Resize(cols, rows));
    }

    pub(crate) fn scroll(&self, direction: u8, lines: u16, col: u16, row: u16) {
        let _ = self.tx.send(AttachmentCommand::Scroll {
            direction,
            lines,
            col,
            row,
        });
    }

    pub(crate) fn close(&self) {
        let _ = self.tx.send(AttachmentCommand::Close);
    }
}

pub(crate) fn spawn_attachment(
    runtime: &Handle,
    socket: PathBuf,
    id: SessionId,
    pane_tx: mpsc::UnboundedSender<PaneEvent>,
) -> AttachmentControl {
    let (command_tx, mut commands) = mpsc::unbounded_channel();
    let control = AttachmentControl {
        tx: command_tx,
        pane_tx: pane_tx.clone(),
    };
    runtime.spawn(async move {
        // The first resize must be the measured pane geometry: deferred agent
        // launch waits for it. Do not seed an arbitrary 80×24 size.
        let mut last_resize = None;
        loop {
            let _ = pane_tx.send(PaneEvent::AttachmentState(
                id.clone(),
                AttachmentState::Attaching,
            ));
            let mut attachment = match SessionAttachment::connect(&socket, id.clone()).await {
                Ok(attachment) => attachment,
                Err(_) => {
                    let _ = pane_tx.send(PaneEvent::AttachmentState(
                        id.clone(),
                        AttachmentState::Reconnecting,
                    ));
                    if wait_for_retry(&mut commands, &mut last_resize).await {
                        return;
                    }
                    continue;
                }
            };
            let writer = attachment.handle();
            if let Some((cols, rows)) = last_resize {
                let _ = writer.resize(cols, rows);
            }
            let _ = pane_tx.send(PaneEvent::AttachmentState(
                id.clone(),
                AttachmentState::Live,
            ));

            let should_close = loop {
                tokio::select! {
                    chunk = attachment.chunks.recv() => {
                        let Some(chunk) = chunk else { break false };
                        if pane_tx.send(PaneEvent::Chunk(id.clone(), chunk)).is_err() {
                            break true;
                        }
                    }
                    command = commands.recv() => {
                        match command {
                            Some(AttachmentCommand::Input(bytes)) => {
                                let _ = writer.send_input(bytes);
                            }
                            Some(AttachmentCommand::Resize(cols, rows)) => {
                                last_resize = Some((cols, rows));
                                let _ = writer.resize(cols, rows);
                            }
                            Some(AttachmentCommand::Scroll { direction, lines, col, row }) => {
                                let _ = writer.scroll(direction, lines, col, row);
                            }
                            Some(AttachmentCommand::Close) | None => break true,
                        }
                    }
                }
            };
            attachment.close().await;
            if should_close {
                return;
            }
            let _ = pane_tx.send(PaneEvent::AttachmentState(
                id.clone(),
                AttachmentState::Reconnecting,
            ));
            if wait_for_retry(&mut commands, &mut last_resize).await {
                return;
            }
        }
    });
    control
}

async fn wait_for_retry(
    commands: &mut mpsc::UnboundedReceiver<AttachmentCommand>,
    last_resize: &mut Option<(u16, u16)>,
) -> bool {
    let delay = tokio::time::sleep(REATTACH_DELAY);
    tokio::pin!(delay);
    loop {
        tokio::select! {
            () = &mut delay => return false,
            command = commands.recv() => match command {
                Some(AttachmentCommand::Resize(cols, rows)) => *last_resize = Some((cols, rows)),
                Some(AttachmentCommand::Close) | None => return true,
                Some(AttachmentCommand::Input(_)) | Some(AttachmentCommand::Scroll { .. }) => {}
            }
        }
    }
}
