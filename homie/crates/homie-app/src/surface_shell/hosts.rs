use std::sync::Arc;

use gpui::{Context, KeyDownEvent, Window};

use homie_proto::{HostEntry, HostsConfig};

use super::host_editor::{HostEditor, HostFormField};
use super::host_init::{HostInitialization, HostPreparationKind, expire_completed_reinstall};
use super::{HOST_REINSTALL_SUCCESS_VISIBILITY, Surface};

use crate::query_editor::{self, ClipboardEdit, Edit};
use crate::settings::SettingsTab;

impl super::UtilitySurfaces {
    pub(super) fn reload_hosts(&mut self) {
        self.hosts = HostsConfig::load(&self.hosts_path).hosts;
        self.store
            .write()
            .expect("session store lock poisoned")
            .set_hosts(self.hosts.clone());
    }

    pub(super) fn begin_adding_host(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.settings_menu = None;
        self.host_editor = Some(HostEditor::adding());
        self.focus.focus(window, cx);
        cx.notify();
    }

    pub(super) fn begin_editing_host(
        &mut self,
        id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(host) = self.hosts.iter().find(|host| host.id == id) else {
            return;
        };
        self.settings_menu = None;
        self.host_editor = Some(HostEditor::editing(host));
        self.focus.focus(window, cx);
        cx.notify();
    }

    pub(super) fn select_host_field(
        &mut self,
        field: HostFormField,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(editor) = &mut self.host_editor {
            editor.active_field = field;
            editor.error = None;
            editor.confirm_remove = false;
            self.focus.focus(window, cx);
            cx.notify();
        }
    }

    pub(super) fn save_host(&mut self, cx: &mut Context<Self>) {
        let Some(editor) = &self.host_editor else {
            return;
        };
        let entry = match editor.draft().entry(&self.hosts) {
            Ok(entry) => entry,
            Err(error) => {
                if let Some(editor) = &mut self.host_editor {
                    editor.error = Some(error);
                    editor.confirm_remove = false;
                }
                cx.notify();
                return;
            }
        };
        let is_new = editor.original_id.is_none();
        let mut hosts = self.hosts.clone();
        if let Some(index) = hosts.iter().position(|host| host.id == entry.id) {
            hosts[index] = entry.clone();
        } else {
            hosts.push(entry.clone());
        }
        self.persist_hosts(hosts, format!("{} is ready", entry.display_name()), cx);
        if is_new && self.host_editor.is_none() {
            self.initialize_host(entry, cx);
        }
    }

    fn initialize_host(&mut self, host: HostEntry, cx: &mut Context<Self>) {
        self.prepare_host(host, HostPreparationKind::Initialize, cx);
    }

    pub(super) fn reinstall_host(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(host) = self.hosts.iter().find(|host| host.id == id).cloned() else {
            return;
        };
        // Reinstallation always targets the saved HostEntry. Discarding the
        // editor avoids implying that unsaved SSH credentials are in use and
        // exposes the progress card immediately.
        self.host_editor = None;
        self.prepare_host(host, HostPreparationKind::Reinstall, cx);
    }

    fn prepare_host(&mut self, host: HostEntry, kind: HostPreparationKind, cx: &mut Context<Self>) {
        let id = host.id.clone();
        let name = host.display_name().to_owned();
        self.host_initialization_generation = self.host_initialization_generation.wrapping_add(1);
        let operation = self.host_initialization_generation;
        self.host_initialization = Some(HostInitialization::Running {
            id: id.clone(),
            name: name.clone(),
            kind,
            operation,
        });
        self.activity = match kind {
            HostPreparationKind::Initialize => format!("Setting up {name} over SSH…"),
            HostPreparationKind::Reinstall => {
                format!("Reinstalling the remote environment on {name}…")
            }
        };
        cx.notify();

        let client = Arc::clone(self.store_runtime.client());
        let runtime = Arc::clone(&self.runtime);
        cx.spawn(async move |this, cx| {
            let task = runtime.spawn(async move {
                match kind {
                    HostPreparationKind::Initialize => client.initialize_host(&id).await,
                    HostPreparationKind::Reinstall => client.reinstall_host(&id).await,
                }
            });
            let outcome = match task.await {
                Ok(result) => result.map_err(|error| error.to_string()),
                Err(error) => Err(error.to_string()),
            };
            let expire_success = outcome.is_ok() && kind == HostPreparationKind::Reinstall;
            let expiration_id = host.id.clone();
            let _ = this.update(cx, |this, cx| {
                if this
                    .host_initialization
                    .as_ref()
                    .is_none_or(|state| state.id() != host.id || state.operation() != operation)
                {
                    return;
                }
                match outcome {
                    Ok(result) => {
                        this.activity = match kind {
                            HostPreparationKind::Initialize => {
                                format!("{} is ready", host.display_name())
                            }
                            HostPreparationKind::Reinstall => {
                                format!("Remote environment reinstalled on {}", host.display_name())
                            }
                        };
                        this.host_initialization = Some(HostInitialization::Ready {
                            id: host.id.clone(),
                            name: host.display_name().to_owned(),
                            kind,
                            operation,
                            result,
                        });
                    }
                    Err(message) => {
                        this.activity = match kind {
                            HostPreparationKind::Initialize => {
                                format!("Could not initialize {}", host.display_name())
                            }
                            HostPreparationKind::Reinstall => format!(
                                "Could not reinstall the remote environment on {}",
                                host.display_name()
                            ),
                        };
                        this.host_initialization = Some(HostInitialization::Failed {
                            id: host.id.clone(),
                            name: host.display_name().to_owned(),
                            kind,
                            operation,
                            message,
                        });
                    }
                }
                cx.notify();
            });
            if expire_success {
                cx.background_executor()
                    .timer(HOST_REINSTALL_SUCCESS_VISIBILITY)
                    .await;
                let _ = this.update(cx, |this, cx| {
                    if expire_completed_reinstall(
                        &mut this.host_initialization,
                        &expiration_id,
                        operation,
                    ) {
                        cx.notify();
                    }
                });
            }
        })
        .detach();
    }

    pub(super) fn retry_host_initialization(
        &mut self,
        id: &str,
        kind: HostPreparationKind,
        cx: &mut Context<Self>,
    ) {
        if let Some(host) = self.hosts.iter().find(|host| host.id == id).cloned() {
            self.prepare_host(host, kind, cx);
        }
    }

    pub(super) fn request_remove_host(&mut self, cx: &mut Context<Self>) {
        let Some(editor) = &mut self.host_editor else {
            return;
        };
        if !editor.confirm_remove {
            editor.confirm_remove = true;
            editor.error = None;
            cx.notify();
            return;
        }
        let Some(id) = editor.original_id.clone() else {
            return;
        };
        let in_use = self
            .store
            .read()
            .expect("session store lock poisoned")
            .sessions()
            .values()
            .any(|session| session.host.as_deref() == Some(id.as_str()));
        if in_use {
            editor.confirm_remove = false;
            editor.error =
                Some("Move or close every session on this host before removing it.".to_owned());
            cx.notify();
            return;
        }
        let name = self
            .hosts
            .iter()
            .find(|host| host.id == id)
            .map_or_else(|| id.clone(), |host| host.display_name().to_owned());
        let hosts = self
            .hosts
            .iter()
            .filter(|host| host.id != id)
            .cloned()
            .collect();
        self.persist_hosts(hosts, format!("Removed {name}"), cx);
    }

    fn persist_hosts(&mut self, hosts: Vec<HostEntry>, activity: String, cx: &mut Context<Self>) {
        match (HostsConfig {
            hosts: hosts.clone(),
        })
        .save(&self.hosts_path)
        {
            Ok(()) => {
                self.hosts = hosts;
                if self
                    .host_initialization
                    .as_ref()
                    .is_some_and(|state| !self.hosts.iter().any(|host| host.id == state.id()))
                {
                    self.host_initialization = None;
                }
                self.store
                    .write()
                    .expect("session store lock poisoned")
                    .set_hosts(self.hosts.clone());
                self.host_editor = None;
                self.activity = activity;
            }
            Err(error) => {
                if let Some(editor) = &mut self.host_editor {
                    editor.error = Some(format!("Could not save hosts: {error}"));
                    editor.confirm_remove = false;
                }
            }
        }
        cx.notify();
    }

    fn edit_host_field(&mut self, edit: Edit, cx: &mut Context<Self>) {
        let Some(host_editor) = &mut self.host_editor else {
            return;
        };
        match edit {
            Edit::Local(local) => {
                host_editor.field_mut().apply(local);
            }
            Edit::Clipboard(ClipboardEdit::Copy) => {
                query_editor::copy_selection(host_editor.field_mut(), cx);
            }
            Edit::Clipboard(ClipboardEdit::Cut) => {
                query_editor::cut_selection(host_editor.field_mut(), cx);
            }
            Edit::Clipboard(ClipboardEdit::Paste) => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    host_editor.field_mut().insert(&text);
                }
            }
        }
        host_editor.error = None;
        host_editor.confirm_remove = false;
        cx.notify();
    }

    pub(super) fn handle_host_editor_key(
        &mut self,
        event: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.surface != Surface::Settings
            || self.settings_tab != SettingsTab::Remote
            || self.host_editor.is_none()
        {
            return false;
        }
        let key = &event.keystroke;
        match key.key.as_str() {
            "escape" => {
                if let Some(editor) = &mut self.host_editor
                    && editor.confirm_remove
                {
                    editor.confirm_remove = false;
                } else {
                    self.host_editor = None;
                }
                cx.notify();
            }
            "tab" => {
                if let Some(editor) = &mut self.host_editor {
                    editor.active_field = editor.active_field.adjacent(key.modifiers.shift);
                    editor.error = None;
                }
                cx.notify();
            }
            "enter" => self.save_host(cx),
            _ => {
                let Some(edit) = query_editor::edit_for(key) else {
                    return false;
                };
                self.edit_host_field(edit, cx);
            }
        }
        true
    }
}
