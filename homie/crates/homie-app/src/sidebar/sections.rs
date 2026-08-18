use super::*;

impl Sidebar {
    pub(crate) fn new_agent_row(
        &self,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let hovering = self.ui.hovered_control == Some("new-agent");
        let (agent, location) = {
            let store = self.store.read().expect("session store lock poisoned");
            let agent = store.preferences().default_agent.display_name().to_owned();
            let location = store
                .default_spawn_host()
                .map_or_else(|| "This Mac".to_owned(), |id| store.host_display_name(&id));
            (agent, location)
        };
        div()
            .id("new-agent")
            .mx(px(Space::INSET))
            .mb(px(4.0))
            .px(px(Space::ROW_H))
            .h(px(44.0))
            .flex()
            .items_center()
            .gap(px(8.0))
            .rounded(px(Radius::ROW))
            .bg(Fill::hover(colors, hovering))
            .cursor_pointer()
            .text_size(px(Typo::ROW.size))
            .text_color(colors.text(homie_ui::TextTone::Label))
            .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                this.ui.hovered_control = hovered.then_some("new-agent");
                cx.notify();
            }))
            .on_click(cx.listener(|this, _, _, cx| {
                this.commit_rename();
                this.open_new_agent_popover(None, cx);
            }))
            .child(
                div()
                    .w(px(16.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(sf_symbol("square.and.pencil", 13.0, colors.secondary)),
            )
            .child(
                div()
                    .min_w(px(0.0))
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(1.0))
                    .child(
                        div()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child("New Agent"),
                    )
                    .child(
                        div()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .text_size(px(Typo::META.size))
                            .text_color(colors.tertiary)
                            .child(format!("{agent} · {location}")),
                    ),
            )
            .child(
                div()
                    .text_size(px(Typo::META.size))
                    .text_color(colors.tertiary)
                    .child("⌘T"),
            )
            .into_any_element()
    }

    pub(crate) fn top_bar(&self, colors: SemanticColors, cx: &mut Context<Self>) -> AnyElement {
        let settings_hover = self.ui.hovered_control == Some("settings");
        let toggle_hover = self.ui.hovered_control == Some("sidebar-toggle");
        div()
            .h(px(Metrics::TITLE_BAR))
            .flex_none()
            .flex()
            .items_center()
            .justify_end()
            .pr(px(Metrics::TOOLBAR_EDGE_INSET))
            .gap(px(Metrics::TOOLBAR_COMPACT_GAP))
            .child(icon_button(
                "settings",
                "gearshape",
                settings_hover,
                colors,
                cx.listener(|this, _, _, cx| {
                    this.ui.popover = None;
                    cx.emit(SidebarEvent::OpenSettings);
                }),
                cx.listener(|this, hovered: &bool, _, cx| {
                    this.ui.hovered_control = hovered.then_some("settings");
                    cx.notify();
                }),
            ))
            .child(icon_button(
                "sidebar-toggle",
                "sidebar.left",
                toggle_hover,
                colors,
                cx.listener(|this, _, _, cx| this.toggle(cx)),
                cx.listener(|this, hovered: &bool, _, cx| {
                    this.ui.hovered_control = hovered.then_some("sidebar-toggle");
                    cx.notify();
                }),
            ))
            .into_any_element()
    }

    pub(crate) fn empty_state(&self, colors: SemanticColors, cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(12.0))
            .child(AgentLogo::new(AgentKind::ClaudeCode, 44.0, colors).badged(false))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(3.0))
                    .child(
                        div()
                            .text_size(px(Typo::ROW_EMPHASIZED.size))
                            .font_weight(Typo::ROW_EMPHASIZED.weight)
                            .text_color(colors.secondary)
                            .child("Bring up your first agent"),
                    )
                    .child(
                        div()
                            .text_size(px(Typo::META.size))
                            .text_color(colors.tertiary)
                            .child("⌘T"),
                    ),
            )
            .child(
                div()
                    .id("empty-new-agent")
                    .px(px(10.0))
                    .h(px(28.0))
                    .flex()
                    .items_center()
                    .rounded(px(Radius::ROW))
                    .text_size(px(Typo::ROW.size))
                    .text_color(colors.secondary)
                    .cursor_pointer()
                    .hover(move |element| element.bg(colors.primary.alpha(0.06)))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open_new_agent_popover(None, cx);
                    }))
                    .gap(px(7.0))
                    .child(sf_symbol("square.and.pencil", 13.0, colors.secondary))
                    .child("New Agent"),
            )
            .into_any_element()
    }

    pub(crate) fn project_section(
        &mut self,
        group: &crate::store::SidebarProject,
        colors: SemanticColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = group.project.id.clone();
        let is_hovered = self.ui.hovered_project.as_ref() == Some(&id);
        let collapsed = self
            .store
            .read()
            .expect("session store lock poisoned")
            .preferences()
            .sidebar_collapsed_projects
            .contains(&id);
        let project_for_click = group.project.clone();
        let project_root = group.project.root.clone();
        let project_host = group.host.clone();
        let project_host_label = group.host.as_deref().map(|host| {
            self.store
                .read()
                .expect("session store lock poisoned")
                .host_display_name(host)
        });
        let entity = cx.entity();
        let drag_label: SharedString = group.project.name.clone().into();
        let mut section = div().flex().flex_col().gap(px(1.0)).child(
            div()
                .id(format!("project:{}", id.0))
                .debug_selector({
                    let id = id.clone();
                    move || format!("PROJECT_{}", id.0)
                })
                .mt(px(6.0))
                .px(px(Space::ROW_H))
                .py(px(5.0))
                .min_h(px(Metrics::ROW_HEIGHT))
                .flex()
                .items_center()
                .gap(px(8.0))
                .rounded(px(Radius::ROW))
                .bg(Fill::hover(colors, is_hovered))
                .cursor_pointer()
                .on_hover(cx.listener({
                    let id = id.clone();
                    move |this, hovered: &bool, _, cx| {
                        this.ui.hovered_project = hovered.then(|| id.clone());
                        cx.notify();
                    }
                }))
                .on_click(cx.listener({
                    let id = id.clone();
                    move |this, _, _, cx| {
                        this.commit_rename();
                        let _ = this
                            .store
                            .write()
                            .expect("session store lock poisoned")
                            .toggle_project_collapsed(id.clone());
                        cx.notify();
                    }
                }))
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener({
                        let id = id.clone();
                        move |this, event: &gpui::MouseDownEvent, window, cx| {
                            cx.stop_propagation();
                            this.commit_rename();
                            this.ui.hover_card = None;
                            this.rename_focus.focus(window, cx);
                            this.ui.popover = Some(Popover::ProjectActions {
                                id: id.clone(),
                                position: Some(event.position),
                            });
                            cx.notify();
                        }
                    }),
                )
                .on_drag(
                    DraggedSidebarItem(DragItem::Project(id.clone())),
                    move |_, _, _, cx| {
                        cx.new(|_| DragPreview {
                            label: drag_label.clone(),
                            colors,
                        })
                    },
                )
                .drag_over::<DraggedSidebarItem>({
                    let id = id.clone();
                    move |element, dragged, _, cx| {
                        if let DragItem::Project(moved) = &dragged.0 {
                            entity.update(cx, |this, cx| {
                                this.reorder_project(moved, &id);
                                this.ui.drag_target = Some(format!("project:{}", id.0));
                                cx.notify();
                            });
                            element.bg(colors.primary.alpha(0.08))
                        } else {
                            element
                        }
                    }
                })
                .on_drop(cx.listener(|this, _: &DraggedSidebarItem, _, cx| {
                    this.finish_drag();
                    cx.notify();
                }))
                .child(project_badge(colors))
                .child(
                    div()
                        .min_w(px(0.0))
                        .flex_1()
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .text_ellipsis()
                        .text_size(px(Typo::ROW_EMPHASIZED.size))
                        .font_weight(Typo::ROW_EMPHASIZED.weight)
                        .text_color(colors.primary.alpha(0.90))
                        .child(group.project.name.clone()),
                )
                .when(group.pinned, |row| row.child(pin_mark(colors)))
                .when_some(project_host_label, |row, host| {
                    row.child(
                        div()
                            .max_w(px(72.0))
                            .px(px(5.0))
                            .py(px(1.0))
                            .rounded(px(Radius::CHIP))
                            .bg(Fill::subtle(colors))
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .text_size(px(Typo::META.size - 1.0))
                            .text_color(colors.tertiary)
                            .child(host),
                    )
                })
                .child(
                    div()
                        .w(px(12.0))
                        .text_center()
                        .text_size(px(9.0))
                        .text_color(colors.secondary)
                        .child(sf_symbol_weighted(
                            if collapsed {
                                "chevron.right"
                            } else {
                                "chevron.down"
                            },
                            9.0,
                            SymbolWeight::Bold,
                            colors.secondary,
                        )),
                )
                .when(is_hovered, |row| {
                    row.child(
                        div()
                            .id(format!("project-menu:{}", id.0))
                            .w(px(20.0))
                            .h(px(20.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(Radius::CHIP))
                            .text_color(colors.secondary)
                            .child(sf_symbol_weighted(
                                "ellipsis",
                                12.0,
                                SymbolWeight::Semibold,
                                colors.secondary,
                            ))
                            .on_click(cx.listener({
                                let project = project_for_click.clone();
                                move |this, _, _, cx| {
                                    cx.stop_propagation();
                                    this.ui.popover = Some(Popover::ProjectActions {
                                        id: project.id.clone(),
                                        position: None,
                                    });
                                    cx.notify();
                                }
                            })),
                    )
                    .child(
                        div()
                            .id(format!("project-plus:{}", id.0))
                            .debug_selector({
                                let id = id.clone();
                                move || format!("PROJECT_ADD_{}", id.0)
                            })
                            .w(px(20.0))
                            .h(px(20.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(Radius::CHIP))
                            .text_color(colors.secondary)
                            .child(sf_symbol_weighted(
                                "plus",
                                12.0,
                                SymbolWeight::Medium,
                                colors.secondary,
                            ))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.open_new_agent_popover_at(
                                    Some(project_root.clone()),
                                    project_host.clone(),
                                    cx,
                                );
                            })),
                    )
                })
                .when(!is_hovered && collapsed, |row| {
                    row.child(AttentionDot::new(rollup_attention(&group.active), colors))
                }),
        );

        // The projection already folds a collapsed project away, so an empty
        // row list here means "collapsed" without asking a second source.
        for row in &group.sessions {
            let shortcut = self.shortcut_for(row.id());
            section = section.child(self.session_row(row, shortcut, colors, window, cx));
        }
        if !collapsed && !group.archived.is_empty() {
            section = section.child(self.archived_bucket(group, colors, window, cx));
        }
        section.into_any_element()
    }

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

    /// The fold control for a row that spawned children, drawn in the same
    /// column a deeper row's rail occupies so titles stay on one axis whether
    /// or not a row has children.
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

    pub(crate) fn archived_bucket(
        &mut self,
        group: &crate::store::SidebarProject,
        colors: SemanticColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let project_id = group.project.id.clone();
        let expanded = self
            .store
            .read()
            .expect("session store lock poisoned")
            .preferences()
            .sidebar_expanded_archives
            .contains(&project_id);
        let targeted =
            self.ui.drag_target.as_deref() == Some(format!("archive:{}", project_id.0).as_str());
        let mut bucket = div()
            .id(format!("archive:{}", project_id.0))
            .flex()
            .flex_col()
            .rounded(px(Radius::ROW))
            .when(targeted, |element| {
                element
                    .bg(colors.primary.alpha(0.08))
                    .border_1()
                    .border_color(colors.primary.alpha(0.18))
            })
            .drag_over::<DraggedSidebarItem>({
                let entity = cx.entity();
                let project_id = project_id.clone();
                move |element, dragged, _, cx| {
                    let valid = match &dragged.0 {
                        DragItem::Session {
                            project, archived, ..
                        } => project == &project_id && !archived,
                        DragItem::Sessions(_) => true,
                        DragItem::Project(_) => false,
                    };
                    if valid {
                        entity.update(cx, |this, cx| {
                            this.ui.drag_target = Some(format!("archive:{}", project_id.0));
                            cx.notify();
                        });
                        element.bg(colors.primary.alpha(0.08))
                    } else {
                        element
                    }
                }
            })
            .on_drop(cx.listener({
                let project_id = project_id.clone();
                move |this, dragged: &DraggedSidebarItem, _, cx| {
                    let ids = match &dragged.0 {
                        DragItem::Session {
                            id,
                            project,
                            archived: false,
                            ..
                        } if project == &project_id => vec![id.clone()],
                        DragItem::Sessions(ids) => ids.clone(),
                        _ => Vec::new(),
                    };
                    this.archive_sessions(ids);
                    this.finish_drag();
                    cx.notify();
                }
            }))
            .child(
                div()
                    .mx(px(Space::ROW_H))
                    .mt(px(3.0))
                    .mb(px(1.0))
                    .h(px(1.0))
                    .bg(colors.primary.alpha(0.06)),
            )
            .child(
                div()
                    .id(format!("archive-header:{}", project_id.0))
                    .pl(px(Space::ROW_H + Space::INDENT))
                    .pr(px(Space::ROW_H))
                    .h(px(22.0))
                    .flex()
                    .items_center()
                    .gap(px(5.0))
                    .cursor_pointer()
                    .text_size(px(Typo::SECTION_HEADER.size))
                    .font_weight(Typo::SECTION_HEADER.weight)
                    .text_color(colors.tertiary)
                    .on_click(cx.listener({
                        let project_id = project_id.clone();
                        move |this, _, _, cx| {
                            let _ = this
                                .store
                                .write()
                                .expect("session store lock poisoned")
                                .toggle_archive_expanded(project_id.clone());
                            cx.notify();
                        }
                    }))
                    .child(sf_symbol_weighted(
                        "archivebox",
                        9.0,
                        SymbolWeight::Semibold,
                        colors.tertiary,
                    ))
                    .child(format!("Archived · {}", group.archived.len()))
                    .child(sf_symbol_weighted(
                        if expanded {
                            "chevron.down"
                        } else {
                            "chevron.right"
                        },
                        8.0,
                        SymbolWeight::Bold,
                        colors.tertiary,
                    )),
            );
        if expanded {
            for session in &group.archived {
                bucket = bucket.child(self.archived_row(session, colors, window, cx));
            }
        }
        bucket.into_any_element()
    }

    pub(crate) fn archived_row(
        &mut self,
        session: &SessionRecord,
        colors: SemanticColors,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = session.id.clone();
        let hovered = self.ui.hovered_session.as_ref() == Some(&id);
        let selected = self
            .store
            .read()
            .expect("session store lock poisoned")
            .selected_session_id()
            == Some(&id);
        let row_session = session.clone();
        let revive_id = id.clone();
        let title = display_title(session);
        let drag_label: SharedString = title.clone().into();
        div()
            .id(format!("archived-session:{}", id.0))
            .pl(px(Space::ROW_H + Space::INDENT))
            .pr(px(Space::ROW_H))
            .h(px(Metrics::ROW_HEIGHT))
            .flex()
            .items_center()
            .gap(px(8.0))
            .rounded(px(Radius::ROW))
            .opacity(0.58)
            .bg(if selected {
                RowFill::Selected.color(colors)
            } else if hovered {
                RowFill::Hover.color(colors)
            } else {
                RowFill::Clear.color(colors)
            })
            .cursor_pointer()
            .on_hover(cx.listener({
                let id = id.clone();
                move |this, is_hovered: &bool, _, cx| {
                    this.ui.hovered_session = is_hovered.then(|| id.clone());
                    cx.notify();
                }
            }))
            .on_click(cx.listener(move |this, event: &gpui::ClickEvent, _, cx| {
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
            }))
            .on_drag(
                DraggedSidebarItem(DragItem::Session {
                    id: id.clone(),
                    project: session.project_id.clone(),
                    parent: session.parent.clone(),
                    archived: true,
                }),
                move |_, _, _, cx| {
                    cx.new(|_| DragPreview {
                        label: drag_label.clone(),
                        colors,
                    })
                },
            )
            .child(
                div()
                    .id(format!("revive:{}", id.0))
                    .size(px(16.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(Radius::CHIP))
                    .bg(if hovered {
                        Fill::subtle(colors)
                    } else {
                        colors.primary.alpha(0.0)
                    })
                    .text_size(px(if hovered { 9.0 } else { 10.0 }))
                    .text_color(colors.secondary)
                    .child(sf_symbol_weighted(
                        if hovered {
                            "tray.and.arrow.up.fill"
                        } else {
                            "archivebox.fill"
                        },
                        if hovered { 8.0 } else { 10.0 },
                        if hovered {
                            SymbolWeight::Bold
                        } else {
                            SymbolWeight::Regular
                        },
                        colors.secondary,
                    ))
                    .when(hovered, |button| {
                        button.on_click(cx.listener(move |this, _, _, cx| {
                            cx.stop_propagation();
                            this.store
                                .write()
                                .expect("session store lock poisoned")
                                .revive_sessions(vec![revive_id.clone()]);
                            cx.notify();
                        }))
                    }),
            )
            .child(
                div()
                    .min_w(px(0.0))
                    .flex_1()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_size(px(Typo::ROW.size))
                    .text_color(colors.primary.alpha(if selected { 1.0 } else { 0.82 }))
                    .child(title),
            )
            .into_any_element()
    }

    /// The update indicator above the account row.
    ///
    /// This is the whole of homie's update UI in the main window, and it stays
    /// out of the way on purpose: a background check that finds something
    /// lights this row and nothing else. Clicking it advances one step —
    /// download, then restart — so an update never begins or completes without
    /// a deliberate click. Manual checks additionally show their outcome here
    /// so "Check for Updates…" is not a command that appears to do nothing.
    pub(crate) fn update_pill(
        &self,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if self.preview || !self.update.is_noteworthy() {
            return None;
        }
        let (symbol, tint, command) = match &self.update.phase {
            UpdatePhase::Available(_) => (
                "arrow.down.circle",
                homie_ui::Ink::FRESH,
                Some(UpdateCommand::Download),
            ),
            UpdatePhase::Downloading { .. } => ("arrow.down.circle", colors.secondary, None),
            UpdatePhase::Ready(_) => (
                "arrow.clockwise.circle",
                homie_ui::Ink::FRESH,
                Some(UpdateCommand::Install),
            ),
            UpdatePhase::Installing => ("arrow.clockwise.circle", colors.secondary, None),
            UpdatePhase::Failed(_) => (
                "exclamationmark.triangle",
                homie_ui::Ink::DANGER,
                Some(UpdateCommand::Dismiss),
            ),
            UpdatePhase::Checking => ("arrow.triangle.2.circlepath", colors.secondary, None),
            UpdatePhase::UpToDate => (
                "checkmark.circle",
                colors.secondary,
                Some(UpdateCommand::Dismiss),
            ),
            UpdatePhase::Idle | UpdatePhase::Unsupported(_) => return None,
        };
        let interactive = command.is_some();
        let hovered = interactive && self.ui.hovered_control == Some("update");
        let mut pill = div()
            .id("update-pill")
            .mb(px(3.0))
            .px(px(Space::ROW_H))
            .h(px(Metrics::ROW_HEIGHT))
            .flex()
            .items_center()
            .gap(px(8.0))
            .rounded(px(Radius::ROW))
            .bg(Fill::hover(colors, hovered))
            .child(
                div()
                    .w(px(16.0))
                    .text_center()
                    .child(sf_symbol(symbol, 12.5, tint)),
            )
            .child(
                div()
                    .min_w(px(0.0))
                    .flex_1()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_size(px(Typo::ROW.size))
                    .text_color(if interactive { tint } else { colors.secondary })
                    .child(self.update.summary()),
            )
            .on_hover(cx.listener(move |this, is_hovered: &bool, _, cx| {
                this.ui.hovered_control = (interactive && *is_hovered).then_some("update");
                cx.notify();
            }));
        if let Some(command) = command {
            pill = pill.cursor_pointer().on_click(cx.listener(
                move |_, _, _, cx: &mut Context<Self>| {
                    cx.emit(SidebarEvent::Update(command.clone()));
                },
            ));
        }
        Some(pill.into_any_element())
    }

    pub(crate) fn account_footer(
        &self,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let hovered = self.ui.hovered_control == Some("account");
        let cost = if self.preview {
            Some(PREVIEW_USAGE)
        } else {
            self.usage
                .map(|snapshot| snapshot.today().cost)
                .filter(|cost| *cost > 0.0)
        };
        div()
            .flex_none()
            .px(px(Space::INSET))
            .pt(px(5.0))
            .pb(px(10.0))
            .border_t_1()
            .border_color(colors.primary.alpha(0.06))
            .children(self.update_pill(colors, cx))
            .child(
                div()
                    .id("account")
                    .px(px(Space::ROW_H))
                    .h(px(Metrics::ROW_HEIGHT))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .rounded(px(Radius::ROW))
                    .bg(Fill::hover(colors, hovered))
                    .cursor_pointer()
                    .on_hover(cx.listener(|this, is_hovered: &bool, _, cx| {
                        this.ui.hovered_control = is_hovered.then_some("account");
                        cx.notify();
                    }))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.ui.popover = Some(Popover::Account);
                        cx.notify();
                    }))
                    .child(
                        div()
                            .w(px(16.0))
                            .text_center()
                            .text_size(px(13.0))
                            .text_color(colors.secondary)
                            .child(sf_symbol("person.crop.circle", 12.5, colors.secondary)),
                    )
                    .child(
                        div()
                            .min_w(px(0.0))
                            .flex_1()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .text_size(px(Typo::ROW.size))
                            .text_color(colors.text(homie_ui::TextTone::Label))
                            .child(if self.preview {
                                "preview@homie.local"
                            } else {
                                "Local agents"
                            }),
                    )
                    .when_some(cost, |row, cost| {
                        row.child(
                            div()
                                .font_family(crate::fonts::mono_family())
                                .text_size(px(Typo::META_MONO.size))
                                .text_color(colors.tertiary)
                                .child(UsageFormat::money(cost)),
                        )
                    })
                    .child(div().text_size(px(9.0)).text_color(colors.tertiary).child(
                        sf_symbol_weighted(
                            "chevron.up.chevron.down",
                            8.5,
                            SymbolWeight::Semibold,
                            colors.tertiary,
                        ),
                    )),
            )
            .into_any_element()
    }
}
