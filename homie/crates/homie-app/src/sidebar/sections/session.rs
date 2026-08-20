use super::*;

impl Sidebar {
    pub(crate) fn session_row(
        &mut self,
        row: &crate::store::SidebarRow,
        shortcut: Option<usize>,
        colors: SemanticColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let session = &row.session;
        let id = session.id.clone();
        let (selected, multi, drag_selection, migrating) = {
            let mut store = self.store.write().expect("session store lock poisoned");
            (
                store.selected_session_id() == Some(&id),
                store.sidebar_selection().contains(&id),
                (store.sidebar_selection().len() > 1).then(|| store.sidebar_selection_ordered()),
                store.migrating().contains(&id),
            )
        };
        let hovered = self.ui.hovered_session.as_ref() == Some(&id);
        let archived = session.is_archived();
        let hibernated = session.hibernation.is_some();
        let ended = matches!(session.status, homie_proto::SessionStatus::Exited(_)) && !archived;
        let host_label = session.host.as_ref().map(|host| {
            self.store
                .read()
                .expect("session store lock poisoned")
                .host_display_name(host)
        });
        let title = display_title(session);
        let non_persistent =
            session.remote_persistence == Some(PersistenceCapability::NonPersistent);
        // Read before the title moves into the marquee below.
        let ended_chip = ended && title != ENDED_TITLE;
        let title_available_width = session_title_available_width(
            self.ui.width,
            row.depth,
            migrating,
            non_persistent,
            ended_chip,
            host_label.as_deref(),
            hibernated,
            row.pinned,
            !hovered && selected && shortcut.is_some(),
        );
        let title_marquee_id = format!("session-title-marquee:{}", id.0);
        let fill = if selected {
            RowFill::Selected
        } else if multi {
            RowFill::MultiSelected
        } else if hovered {
            RowFill::Hover
        } else {
            RowFill::Clear
        };

        if self.ui.renaming.as_ref() == Some(&id) {
            return div()
                .id(format!("rename:{}", id.0))
                .pl(px(Space::ROW_H))
                .pr(px(Space::ROW_H))
                .h(px(Metrics::ROW_HEIGHT))
                .flex()
                .items_center()
                .gap(px(8.0))
                .rounded(px(Radius::ROW))
                .bg(RowFill::Selected.color(colors))
                .children(indent_rails(row, colors))
                // The fold control is inert mid-rename, but its column stays so
                // the text does not slide sideways the moment editing starts.
                .child(div().w(px(Space::INDENT)).flex_none())
                .child(
                    div()
                        .size(px(16.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(self.status_glyph(session, migrating, colors, window, cx)),
                )
                .child(
                    div()
                        .min_w(px(0.0))
                        .flex_1()
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .text_size(px(Typo::ROW.size))
                        .text_color(colors.primary)
                        .child(query_label(&self.ui.rename_draft)),
                )
                .into_any_element();
        }

        let row_session = Arc::clone(session);
        let rename_session = Arc::clone(session);
        let close_id = id.clone();
        let hover_id = id.clone();
        let drag_item = if multi && let Some(selection) = drag_selection {
            DragItem::Sessions(selection)
        } else {
            DragItem::Session {
                id: id.clone(),
                project: session.project_id.clone(),
                parent: session.parent.clone(),
                archived,
            }
        };
        let drag_payload = DraggedSidebarItem(drag_item);
        let drag_label: SharedString = title.clone().into();
        let entity = cx.entity();
        div()
            .id(format!("session:{}", id.0))
            .pl(px(Space::ROW_H))
            .pr(px(Space::ROW_H))
            .h(px(Metrics::ROW_HEIGHT))
            .flex()
            .items_center()
            .gap(px(8.0))
            .rounded(px(Radius::ROW))
            .bg(fill.color(colors))
            .opacity(if archived {
                0.58
            } else if hibernated {
                0.74
            } else {
                1.0
            })
            .cursor_pointer()
            .on_hover(cx.listener(move |this, is_hovered: &bool, window, cx| {
                this.ui.hovered_session = is_hovered.then(|| hover_id.clone());
                this.schedule_hover_card(hover_id.clone(), *is_hovered, window, cx);
                cx.notify();
            }))
            .on_click(
                cx.listener(move |this, event: &gpui::ClickEvent, window, cx| {
                    this.commit_rename();
                    if event.click_count() == 2 {
                        this.begin_rename(&row_session, window, cx);
                        return;
                    }
                    let modifiers = event.modifiers();
                    this.store
                        .write()
                        .expect("session store lock poisoned")
                        .sidebar_click(
                            row_session.id.clone(),
                            ClickModifiers {
                                command: modifiers.platform,
                                shift: modifiers.shift,
                            },
                        );
                    if !modifiers.platform && !modifiers.shift {
                        cx.emit(SidebarEvent::SessionActivated);
                    }
                    cx.notify();
                }),
            )
            .on_mouse_up(
                MouseButton::Middle,
                cx.listener(move |this, _, _, cx| {
                    this.close_sessions(vec![close_id.clone()], cx);
                    cx.notify();
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                    cx.stop_propagation();
                    this.commit_rename();
                    this.ui.hover_card = None;
                    this.rename_focus.focus(window, cx);
                    this.ui.popover = Some(Popover::SessionActions {
                        id: rename_session.id.clone(),
                        position: event.position,
                    });
                    cx.notify();
                }),
            )
            .on_drag(drag_payload, move |_, _, _, cx| {
                cx.new(|_| DragPreview {
                    label: drag_label.clone(),
                    colors,
                })
            })
            .drag_over::<DraggedSidebarItem>({
                let target = id.clone();
                let target_project = session.project_id.clone();
                let target_parent = session.parent.clone();
                move |element, dragged, _, cx| {
                    // Siblings only. A row dropped on a cousin would shuffle
                    // the manual order without moving anything on screen,
                    // because each sibling run is sorted among itself.
                    if let DragItem::Session {
                        id: moved,
                        project,
                        parent,
                        archived: false,
                    } = &dragged.0
                        && project == &target_project
                        && parent == &target_parent
                    {
                        entity.update(cx, |this, cx| {
                            this.reorder_session(moved, &target);
                            this.ui.drag_target = Some(format!("session:{}", target.0));
                            cx.notify();
                        });
                        element.bg(colors.primary.alpha(0.08))
                    } else {
                        element
                    }
                }
            })
            .on_drop(cx.listener({
                let target_project = session.project_id.clone();
                move |this, dragged: &DraggedSidebarItem, _, cx| {
                    if let DragItem::Session {
                        id,
                        project,
                        archived: true,
                        ..
                    } = &dragged.0
                        && project == &target_project
                    {
                        this.store
                            .write()
                            .expect("session store lock poisoned")
                            .revive_sessions(vec![id.clone()]);
                    }
                    this.finish_drag();
                    cx.notify();
                }
            }))
            .children(indent_rails(row, colors))
            .child(self.disclosure(row, colors, cx))
            // The status glyph is the row's whole reason for existing at a
            // glance, so it no longer yields its slot to the close button on
            // hover -- pointing at a working agent used to hide the fact that
            // it was working. The ✕ lives at the trailing edge instead.
            .child(
                div()
                    .size(px(16.0))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(self.status_glyph(session, migrating, colors, window, cx)),
            )
            .child(
                HoverMarquee::new(
                    title_marquee_id,
                    title,
                    hovered,
                    title_available_width,
                    Typo::ROW.size,
                    colors.primary.alpha(if selected { 1.0 } else { 0.82 }),
                )
                .font_weight(Typo::ROW.weight),
            )
            .when(row.pinned, |element| element.child(pin_mark(colors)))
            // Chips, in descending order of how much they explain an otherwise
            // inert-looking row. Each is flex_none and the title absorbs the
            // remaining width, so a narrow sidebar truncates the title rather
            // than dropping the reason it is not moving.
            .when(migrating, |element| {
                element.child(state_chip("Moving…", colors.secondary, colors))
            })
            .when(non_persistent, |element| {
                // Louder than the rest of the lane on purpose: this session
                // cannot survive a detach, so closing the window loses it.
                element.child(alert_chip("No detach"))
            })
            .when(ended_chip, |element| {
                // An exited session with a real title otherwise looks alive:
                // the glyph goes quiet and nothing else says why.
                element.child(state_chip("Ended", colors.tertiary, colors))
            })
            .when(hibernated, |element| {
                // Hibernation chip. An 8px moon glyph was a smudge at this
                // size; the chip reads at a glance and matches the host badge.
                element.child(state_chip("Zzz", colors.tertiary, colors))
            })
            .when_some(host_label, |element, host| {
                // Remote-host chip: this session's agent runs on another machine.
                element.child(state_chip(host, colors.tertiary, colors))
            })
            .when(hovered, |element| {
                let close_id = id.clone();
                element.child(
                    div()
                        .id(format!("close:{}", id.0))
                        .size(px(16.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(Radius::CHIP))
                        .cursor_pointer()
                        .text_color(colors.secondary)
                        .hover(move |button| button.bg(Fill::subtle(colors)))
                        // The row is draggable, and a press that wanders
                        // 2px turns into a drag that swallows the click.
                        // Keeping mouse-down off the row makes every press
                        // on the ✕ a close.
                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .child(sf_symbol_weighted(
                            "xmark",
                            8.5,
                            SymbolWeight::Bold,
                            colors.secondary,
                        ))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            cx.stop_propagation();
                            this.close_sessions(vec![close_id.clone()], cx);
                            cx.notify();
                        })),
                )
            })
            // The hint and the ✕ share the trailing edge and never both apply:
            // hovering a row is the moment you want to close it, not the
            // moment you need to be told how to reach it from the keyboard.
            .when_some(
                (!hovered && selected).then_some(shortcut).flatten(),
                |element, index| {
                    element.child(
                        div()
                            .flex_none()
                            .text_size(px(Typo::META.size))
                            .text_color(colors.tertiary)
                            .child(format!("⌘{index}")),
                    )
                },
            )
            .into_any_element()
    }

    pub(crate) fn disclosure(
        &self,
        row: &crate::store::SidebarRow,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let slot = div().w(px(Space::INDENT)).flex_none().flex().items_center();
        if !row.has_children {
            return slot.into_any_element();
        }
        let id = row.id().clone();
        slot.id(format!("fold:{}", id.0))
            .justify_center()
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                let _ = this
                    .store
                    .write()
                    .expect("session store lock poisoned")
                    .toggle_session_collapsed(id.clone());
                cx.notify();
            }))
            .child(sf_symbol_weighted(
                if row.collapsed {
                    "chevron.right"
                } else {
                    "chevron.down"
                },
                8.0,
                SymbolWeight::Bold,
                colors.tertiary,
            ))
            .into_any_element()
    }
}
