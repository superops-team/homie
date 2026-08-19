use super::widgets::*;
use super::*;

impl UtilitySurfaces {
    pub(super) fn remote_settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.settings_colors();
        settings_page(
            "Remote",
            div()
                .flex()
                .flex_col()
                .gap(px(SETTINGS_SECTION_GAP))
                .child(self.remote_hosts_section(cx))
                .child(
                    settings_note(
                        "lock.shield",
                        Some("OpenSSH transport"),
                        "Homie uses your SSH configuration without changing the host. Private-network and Tailscale names work transparently when OpenSSH can resolve them.",
                        colors.secondary,
                        colors.primary.alpha(0.08),
                        colors.primary.alpha(0.035),
                        colors,
                    ),
                ),
            colors,
        )
    }

    fn remote_hosts_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.settings_colors();
        let default_host = self
            .store
            .read()
            .expect("session store lock poisoned")
            .default_spawn_host();
        let mut catalog = div()
            .rounded(px(Radius::ROW))
            .border_1()
            .border_color(colors.primary.alpha(0.065))
            .bg(colors.primary.alpha(0.02))
            .overflow_hidden();

        if let Some(editor) = &self.host_editor {
            catalog = catalog.child(self.host_editor_panel(editor, cx));
        } else if self.hosts.is_empty() {
            catalog = catalog.child(
                div()
                    .px(px(14.0))
                    .py(px(16.0))
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .child(
                        div()
                            .size(px(34.0))
                            .rounded(px(10.0))
                            .bg(colors.primary.alpha(0.065))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(sf_symbol("network", 17.0, colors.tertiary)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(3.0))
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("No execution hosts yet"),
                            )
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(colors.tertiary)
                                    .child("Add any machine you can reach over SSH."),
                            ),
                    ),
            );
        } else {
            for (index, host) in self.hosts.iter().enumerate() {
                if index > 0 {
                    catalog = catalog.child(setting_divider(colors));
                }
                let id = host.id.clone();
                let name = host.display_name().to_owned();
                let destination = host.ssh.clone();
                let folder = host.default_cwd.clone();
                let first_party = host.node.is_some();
                let is_default = default_host.as_deref() == Some(host.id.as_str());
                catalog = catalog.child(
                    div()
                        .id(SharedString::from(format!("remote-host-{id}")))
                        .min_h(px(SETTINGS_ROW_HEIGHT))
                        .px(px(12.0))
                        .flex()
                        .items_center()
                        .gap(px(11.0))
                        .cursor_pointer()
                        .hover(move |style| style.bg(colors.primary.alpha(0.055)))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.begin_editing_host(&id, window, cx);
                        }))
                        .child(
                            div()
                                .size(px(30.0))
                                .rounded(px(9.0))
                                .bg(colors.primary.alpha(0.06))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(sf_symbol("network", 15.0, colors.secondary)),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.0))
                                .flex()
                                .flex_col()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .flex()
                                        .min_w(px(0.0))
                                        .items_center()
                                        .gap(px(7.0))
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(
                                            div()
                                                .min_w(px(0.0))
                                                .flex_1()
                                                .overflow_hidden()
                                                .whitespace_nowrap()
                                                .text_ellipsis()
                                                .child(name),
                                        )
                                        .when(first_party, |row| {
                                            row.child(
                                                div()
                                                    .px(px(6.0))
                                                    .py(px(2.0))
                                                    .rounded(px(Radius::BADGE))
                                                    .bg(Ink::FRESH.alpha(0.10))
                                                    .text_size(px(9.0))
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(Ink::FRESH)
                                                    .child("NODE"),
                                            )
                                        })
                                        .when(is_default, |row| {
                                            row.child(
                                                div()
                                                    .px(px(6.0))
                                                    .py(px(2.0))
                                                    .rounded(px(Radius::BADGE))
                                                    .bg(Palette::CLAY.alpha(0.12))
                                                    .text_size(px(9.0))
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(Palette::CLAY)
                                                    .child("DEFAULT"),
                                            )
                                        }),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .min_w(px(0.0))
                                        .items_center()
                                        .gap(px(7.0))
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .text_ellipsis()
                                        .font_family(crate::fonts::mono_family())
                                        .text_size(px(10.5))
                                        .text_color(colors.tertiary)
                                        .child(destination)
                                        .when_some(folder, |line, folder| {
                                            line.child(
                                                div()
                                                    .text_color(colors.primary.alpha(0.22))
                                                    .child("·"),
                                            )
                                            .child(folder)
                                        }),
                                ),
                        )
                        .child(sf_symbol("chevron.right", 10.0, colors.tertiary)),
                );
            }
        }

        div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(
                div()
                    .h(px(28.0))
                    .px(px(2.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(Typo::SECTION_HEADER.size))
                            .font_weight(Typo::SECTION_HEADER.weight)
                            .text_color(colors.tertiary)
                            .child("Execution hosts"),
                    )
                    .when(self.host_editor.is_none(), |header| {
                        header.child(settings_primary_button(
                            "Add Host",
                            "add-remote-host",
                            Some("plus"),
                            cx,
                            |this, window, cx| this.begin_adding_host(window, cx),
                        ))
                    }),
            )
            .when_some(self.host_initialization.clone(), |section, state| {
                section.child(self.host_initialization_card(state, cx))
            })
            .child(catalog)
    }

    fn host_initialization_card(
        &self,
        state: HostInitialization,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = self.settings_colors();
        let HostInitializationCardModel {
            id,
            name,
            symbol,
            title,
            detail,
            tone,
            action,
            retry_kind,
        } = match state {
            HostInitialization::Running { id, name, kind, .. } => {
                let (title, detail) = match kind {
                    HostPreparationKind::Initialize => (
                        format!("Setting up {name}"),
                        "Connecting with SSH, verifying homie-remote, loading the login environment, and testing session persistence."
                            .to_owned(),
                    ),
                    HostPreparationKind::Reinstall => (
                        format!("Reinstalling {name}"),
                        "Uploading and verifying the packaged homie-remote build, then refreshing the remote environment checks. Running sessions are not interrupted."
                            .to_owned(),
                    ),
                };
                HostInitializationCardModel {
                    id,
                    name,
                    symbol: None,
                    title,
                    detail,
                    tone: Palette::CLAY,
                    action: None,
                    retry_kind: None,
                }
            }
            HostInitialization::Ready {
                id,
                name,
                kind,
                result,
                ..
            } => {
                let persistence = match result.persistence {
                    homie_proto::remote_pty::PersistenceCapability::NativeDetach => "native detach",
                    homie_proto::remote_pty::PersistenceCapability::UserSupervisor => {
                        "user supervisor"
                    }
                    homie_proto::remote_pty::PersistenceCapability::NonPersistent => {
                        "non-persistent"
                    }
                };
                let title = match kind {
                    HostPreparationKind::Initialize => format!("{name} is ready"),
                    HostPreparationKind::Reinstall => {
                        format!("Remote environment reinstalled on {name}")
                    }
                };
                let action = (kind == HostPreparationKind::Initialize).then_some("Use by default");
                HostInitializationCardModel {
                    id,
                    name: name.clone(),
                    symbol: Some("checkmark.circle.fill"),
                    title,
                    detail: format!(
                        "{} · {} · build {} · protocol {}.{} · {persistence}",
                        result.cwd,
                        result.shell,
                        result.helper_build_id,
                        result.protocol.major,
                        result.protocol.minor
                    ),
                    tone: Ink::FRESH,
                    action,
                    retry_kind: None,
                }
            }
            HostInitialization::Failed {
                id,
                name,
                kind,
                message,
                ..
            } => {
                let title = match kind {
                    HostPreparationKind::Initialize => {
                        format!("Could not initialize {name}")
                    }
                    HostPreparationKind::Reinstall => {
                        format!("Could not reinstall the remote environment on {name}")
                    }
                };
                HostInitializationCardModel {
                    id,
                    name,
                    symbol: Some("exclamationmark.triangle.fill"),
                    title,
                    detail: message,
                    tone: Ink::DANGER,
                    action: Some("Retry"),
                    retry_kind: Some(kind),
                }
            }
        };
        let action_id = id.clone();
        let status_mark = symbol.map_or_else(
            || LoadingIndicator::new("host-initialization-loading", 16.0, tone).into_any_element(),
            |symbol| sf_symbol(symbol, 13.0, tone),
        );
        div()
            .id("host-initialization")
            .debug_selector(|| "HOST_INITIALIZATION".into())
            .rounded(px(Radius::ROW))
            .border_1()
            .border_color(tone.alpha(0.22))
            .bg(tone.alpha(0.055))
            .px(px(12.0))
            .py(px(10.0))
            .flex()
            .items_start()
            .gap(px(9.0))
            .child(div().pt(px(1.0)).text_color(tone).child(status_mark))
            .child(
                div()
                    .min_w(px(0.0))
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(3.0))
                    .child(
                        div()
                            .text_size(px(11.5))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.primary)
                            .child(title),
                    )
                    .child(
                        div()
                            .min_w(px(0.0))
                            .w_full()
                            .whitespace_normal()
                            .text_size(px(10.5))
                            .line_height(px(15.0))
                            .text_color(colors.tertiary)
                            .child(wrappable_setting_copy(detail.into())),
                    ),
            )
            .when_some(action, |card, label| {
                card.child(
                    div()
                        .debug_selector(|| "HOST_INITIALIZATION_ACTION".into())
                        .child(surface_button(
                            label,
                            "host-initialization-action",
                            colors,
                            cx,
                            move |this, cx| {
                                if let Some(kind) = retry_kind {
                                    this.retry_host_initialization(&action_id, kind, cx);
                                } else {
                                    this.store
                                        .write()
                                        .expect("session store lock poisoned")
                                        .set_default_spawn_host(Some(action_id.clone()));
                                    this.activity =
                                        format!("{name} is now the default execution host");
                                    cx.notify();
                                }
                            },
                        )),
                )
            })
            .into_any_element()
    }

    fn host_editor_panel(&self, editor: &HostEditor, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.settings_colors();
        let editing = editor.original_id.is_some();
        let title = if editing { "Edit host" } else { "Add a host" };
        let mut form = div()
            .p(px(14.0))
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(
                div()
                    .flex()
                    .items_start()
                    .justify_between()
                    .gap(px(12.0))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .flex()
                            .flex_col()
                            .gap(px(3.0))
                            .child(
                                div()
                                    .text_size(px(Typo::TITLE.size))
                                    .font_weight(Typo::TITLE.weight)
                                    .child(title),
                            )
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(colors.tertiary)
                                    .child("SSH aliases from ~/.ssh/config work too."),
                            ),
                    )
                    .child(
                        div()
                            .id("cancel-host-editor")
                            .size(px(24.0))
                            .rounded(px(Radius::BADGE))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(move |style| style.bg(colors.primary.alpha(0.08)))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.host_editor = None;
                                cx.notify();
                            }))
                            .child(sf_symbol("xmark", 11.0, colors.secondary)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .child(
                        div()
                            .flex()
                            .gap(px(10.0))
                            .child(
                                div().min_w(px(0.0)).flex_1().child(self.host_text_field(
                                    "Name",
                                    "Forge",
                                    &editor.name,
                                    HostFormField::Name,
                                    cx,
                                )),
                            )
                            .child(
                                div().min_w(px(0.0)).flex_1().child(self.host_text_field(
                                    "SSH destination",
                                    "you@forge",
                                    &editor.ssh,
                                    HostFormField::Ssh,
                                    cx,
                                )),
                            ),
                    )
                    .child(self.host_text_field(
                        "Default folder",
                        "~/code (optional)",
                        &editor.default_cwd,
                        HostFormField::DefaultCwd,
                        cx,
                    ))
                    .child(
                        div()
                            .pt(px(2.0))
                            .text_size(px(10.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.secondary)
                            .child("First-party node (optional)"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(10.0))
                            .child(
                                div().min_w(px(0.0)).flex_1().child(self.host_text_field(
                                    "Node endpoint",
                                    "tcp://100.64.0.2:7337",
                                    &editor.node_endpoint,
                                    HostFormField::NodeEndpoint,
                                    cx,
                                )),
                            )
                            .child(
                                div().min_w(px(0.0)).flex_1().child(self.host_text_field(
                                    "Local token file",
                                    "~/.config/homie/forge.token",
                                    &editor.node_token_file,
                                    HostFormField::NodeTokenFile,
                                    cx,
                                )),
                            ),
                    )
                    .child(self.host_text_field(
                        "Pinned node ID",
                        "node-a1b2c3d4 (recommended after first hello)",
                        &editor.node_id,
                        HostFormField::NodeId,
                        cx,
                    ))
                    .child(
                        div()
                            .min_w(px(0.0))
                            .w_full()
                            .whitespace_normal()
                            .text_size(px(10.0))
                            .line_height(px(15.0))
                            .text_color(colors.tertiary)
                            .child(wrappable_setting_copy(
                                "The token stays in that owner-only file. SSH remains available for install and recovery."
                                    .into(),
                            )),
                    ),
            );

        if let Some(error) = &editor.error {
            form = form.child(
                div()
                    .px(px(10.0))
                    .py(px(8.0))
                    .rounded(px(Radius::BADGE))
                    .bg(Ink::DANGER.alpha(0.08))
                    .flex()
                    .items_center()
                    .gap(px(7.0))
                    .text_size(px(11.0))
                    .text_color(Ink::DANGER)
                    .child(sf_symbol("exclamationmark.triangle", 12.0, Ink::DANGER))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .whitespace_normal()
                            .child(wrappable_setting_copy(error.clone().into())),
                    ),
            );
        }

        if editor.confirm_remove {
            let name = if editor.name.is_empty() {
                "this host".to_owned()
            } else {
                editor.name.text().to_owned()
            };
            form = form.child(
                div()
                    .pt(px(2.0))
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap(px(12.0))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .whitespace_normal()
                            .text_size(px(11.0))
                            .text_color(colors.secondary)
                            .child(wrappable_setting_copy(
                                format!("Remove {name}? Existing sessions must be moved first.")
                                    .into(),
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(7.0))
                            .child(surface_button(
                                "Keep Host",
                                "keep-host",
                                colors,
                                cx,
                                |this, cx| {
                                    if let Some(editor) = &mut this.host_editor {
                                        editor.confirm_remove = false;
                                    }
                                    cx.notify();
                                },
                            ))
                            .child(settings_danger_button(
                                "Remove Host",
                                "confirm-remove-host",
                                cx,
                                |this, cx| this.request_remove_host(cx),
                            )),
                    ),
            );
        } else {
            let reinstall_host_id = editor.original_id.clone();
            form = form.child(
                div()
                    .pt(px(2.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(8.0))
                    .child(div().when(editing, |left| {
                        let host_id = reinstall_host_id.expect("editing host id");
                        left.flex()
                            .flex_wrap()
                            .items_center()
                            .gap(px(7.0))
                            .child(settings_danger_button(
                                "Remove",
                                "remove-host",
                                cx,
                                |this, cx| this.request_remove_host(cx),
                            ))
                            .child(
                                div()
                                    .debug_selector(|| "REINSTALL_REMOTE_ENVIRONMENT".into())
                                    .child(surface_button(
                                        "Reinstall Environment",
                                        "reinstall-remote-environment",
                                        colors,
                                        cx,
                                        move |this, cx| this.reinstall_host(&host_id, cx),
                                    )),
                            )
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(7.0))
                            .child(surface_button(
                                "Cancel",
                                "cancel-host",
                                colors,
                                cx,
                                |this, cx| {
                                    this.host_editor = None;
                                    cx.notify();
                                },
                            ))
                            .child(settings_primary_button(
                                if editing { "Save Host" } else { "Add Host" },
                                "save-host",
                                None,
                                cx,
                                |this, _, cx| this.save_host(cx),
                            )),
                    ),
            );
        }
        form
    }

    fn host_text_field(
        &self,
        label: &'static str,
        placeholder: &'static str,
        editor: &QueryEditor,
        field: HostFormField,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let colors = self.settings_colors();
        let active = self
            .host_editor
            .as_ref()
            .is_some_and(|host_editor| host_editor.active_field == field);
        let value = host_field_value(editor, placeholder, active, field, colors);
        let bounds_slot = Rc::clone(&self.host_field_bounds[field.index()]);
        div()
            .min_w(px(0.0))
            .flex()
            .flex_col()
            .gap(px(5.0))
            .child(
                div()
                    .text_size(px(10.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(colors.secondary)
                    .child(label),
            )
            .child({
                let debug_name = field.debug_name();
                div()
                    .id(SharedString::from(format!("host-field-{field:?}")))
                    .debug_selector(move || format!("HOST_FIELD_{debug_name}"))
                    .relative()
                    .min_w(px(0.0))
                    .h(px(34.0))
                    .px(px(10.0))
                    .rounded(px(Radius::BADGE))
                    .border_1()
                    .border_color(colors.primary.alpha(if active { 0.26 } else { 0.11 }))
                    .bg(colors.primary.alpha(if active { 0.075 } else { 0.04 }))
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .font_family(crate::fonts::mono_family())
                    .text_size(px(11.0))
                    .text_color(colors.primary)
                    .cursor(CursorStyle::IBeam)
                    .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                        this.select_host_field(field, window, cx);
                        let Some(bounds) = this.host_field_bounds[field.index()].get() else {
                            return;
                        };
                        let x = (event.position().x - bounds.left() - px(10.0)).max(px(0.0));
                        let offset = this.host_editor.as_ref().map_or(0, |editor| {
                            text_offset_for_x(editor.field(field).text(), x, window, colors)
                        });
                        if let Some(editor) = &mut this.host_editor {
                            editor
                                .field_mut()
                                .set_cursor(offset, event.modifiers().shift);
                        }
                        cx.notify();
                    }))
                    .child(value)
                    .child(
                        canvas(
                            move |bounds, _, _| bounds_slot.set(Some(bounds)),
                            |_, _, _, _| {},
                        )
                        .absolute()
                        .inset_0(),
                    )
            })
    }
}
