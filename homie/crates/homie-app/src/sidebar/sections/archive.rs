use super::*;

impl Sidebar {
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
}
