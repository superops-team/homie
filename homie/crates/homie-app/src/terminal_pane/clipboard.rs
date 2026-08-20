use super::*;

impl TerminalPane {
    pub(super) fn copy_selection(
        &mut self,
        _: &CopySelection,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(id) = self.selected_id() else {
            return;
        };
        let Some(resident) = self.residents.get(&id) else {
            return;
        };
        let text = resident.element.selected_text();
        if !text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    pub(super) fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item) = cx.read_from_clipboard() else {
            return;
        };
        let Some(id) = self.selected_id() else {
            return;
        };

        if let Some((bytes, extension)) = clipboard_image(&item) {
            let in_find = self
                .residents
                .get(&id)
                .is_some_and(|resident| resident.find.is_some());
            if in_find {
                return;
            }

            let staged = match StagedClipboardImage::stage(bytes, extension) {
                Ok(staged) => staged,
                Err(error) => {
                    eprintln!("homie: could not stage clipboard image: {error}");
                    return;
                }
            };
            let ssh = {
                let store = self
                    .runtime
                    .store
                    .read()
                    .expect("session store lock poisoned");
                store
                    .selected_session()
                    .and_then(|session| session.host.as_deref())
                    .and_then(|host_id| store.host(host_id))
                    .map(|host| host.ssh.clone())
            };

            if let Some(ssh) = ssh {
                let pane_tx = self.pane_tx.clone();
                let upload_id = id.clone();
                self.tokio.spawn(async move {
                    let result = tokio::task::spawn_blocking(move || staged.upload(&ssh))
                        .await
                        .unwrap_or_else(|error| Err(format!("upload task failed: {error}")));
                    let _ = pane_tx.send(PaneEvent::ClipboardUploadFinished(upload_id, result));
                });
            } else {
                let local_path = staged.path().to_string_lossy().into_owned();
                if let Some(resident) = self.residents.get(&id) {
                    resident.attachment.input(paste(&local_path, false));
                }
                self.local_clipboard_images.push(staged);
                if self.local_clipboard_images.len() > 32 {
                    self.local_clipboard_images.remove(0);
                }
            }
            cx.stop_propagation();
            cx.notify();
            return;
        }

        let Some(text) = item.text() else {
            return;
        };
        let now = self.started_at.elapsed();
        let Some(resident) = self.residents.get_mut(&id) else {
            return;
        };
        if let Some(find) = resident.find.as_mut() {
            resident.find_query.insert(&text);
            let query = resident.find_query.text().to_owned();
            find.set_query(query, now);
            self.schedule_find(id, Duration::from_millis(200), window, cx);
        } else {
            resident.attachment.input(paste(&text, false));
        }
        cx.stop_propagation();
        cx.notify();
    }
}
