use super::*;

impl Sidebar {
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
}
