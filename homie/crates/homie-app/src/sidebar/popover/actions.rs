use super::*;

impl Sidebar {
    pub(crate) fn project_actions_popover(
        &self,
        id: ProjectId,
        position: Option<Point<Pixels>>,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (project, host, collapsed, pinned) = {
            let store = self.store.read().expect("session store lock poisoned");
            let Some(project) = store.projects().get(&id).cloned() else {
                return div().into_any_element();
            };
            (
                project,
                store
                    .sessions()
                    .values()
                    .find(|session| session.project_id == id)
                    .and_then(|session| session.host.clone()),
                store.preferences().sidebar_collapsed_projects.contains(&id),
                store.preferences().sidebar_pinned_projects.contains(&id),
            )
        };
        let content = div()
            .p(px(6.0))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .child(menu_row(
                "New Session Here",
                colors,
                cx.listener({
                    let root = project.root.clone();
                    let host = host.clone();
                    move |this, _, _, cx| {
                        this.open_new_agent_popover_at(Some(root.clone()), host.clone(), cx);
                    }
                }),
            ))
            .child(menu_row(
                if pinned {
                    "Unpin Project"
                } else {
                    "Pin Project"
                },
                colors,
                cx.listener({
                    let id = id.clone();
                    move |this, _, _, cx| {
                        let _ = this
                            .store
                            .write()
                            .expect("session store lock poisoned")
                            .toggle_project_pin(id.clone());
                        this.ui.popover = None;
                        cx.notify();
                    }
                }),
            ))
            .child(menu_row(
                if collapsed { "Expand" } else { "Collapse" },
                colors,
                cx.listener(move |this, _, _, cx| {
                    let _ = this
                        .store
                        .write()
                        .expect("session store lock poisoned")
                        .toggle_project_collapsed(id.clone());
                    this.ui.popover = None;
                    cx.notify();
                }),
            ));
        match position {
            Some(position) => {
                self.popover_shell_at(position, Anchor::TopLeft, 200.0, content, colors, cx)
            }
            None => self.popover_shell(96.0, content, colors, cx),
        }
    }

    pub(crate) fn session_actions_popover(
        &self,
        id: SessionId,
        position: Point<Pixels>,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (session, pinned, bulk, hosts, migrating) = {
            let mut store = self.store.write().expect("session store lock poisoned");
            let Some(session) = store.sessions().get(&id).cloned() else {
                return div().into_any_element();
            };
            let pinned = store.preferences().sidebar_pinned_sessions.contains(&id);
            // The whole multi-selection, when the right-clicked row is part
            // of one (Swift: bulk actions split archive/revive honestly).
            let bulk =
                if store.sidebar_selection().len() > 1 && store.sidebar_selection().contains(&id) {
                    store.sidebar_selection_ordered()
                } else {
                    Vec::new()
                };
            let hosts = store.hosts().to_vec();
            let migrating = store.migrating().contains(&id);
            (session, pinned, bulk, hosts, migrating)
        };
        let mut content = div().p(px(6.0)).flex().flex_col().gap(px(2.0));
        if bulk.len() > 1 {
            let (active, parked): (Vec<SessionId>, Vec<SessionId>) = {
                let store = self.store.read().expect("session store lock poisoned");
                bulk.iter().cloned().partition(|session_id| {
                    store
                        .sessions()
                        .get(session_id)
                        .is_none_or(|session| !session.is_archived())
                })
            };
            if !active.is_empty() {
                content = content.child(menu_row(
                    count_label("Archive", active.len()),
                    colors,
                    cx.listener(move |this, _, _, cx| {
                        this.archive_sessions(active.clone());
                        this.ui.popover = None;
                        cx.notify();
                    }),
                ));
            }
            if !parked.is_empty() {
                content = content.child(menu_row(
                    count_label("Revive", parked.len()),
                    colors,
                    cx.listener(move |this, _, _, cx| {
                        this.store
                            .write()
                            .expect("session store lock poisoned")
                            .revive_sessions(parked.clone());
                        this.ui.popover = None;
                        cx.notify();
                    }),
                ));
            }
            content = content.child(menu_row(
                count_label("Close", bulk.len()),
                colors,
                cx.listener(move |this, _, _, cx| {
                    this.close_sessions(bulk.clone(), cx);
                    this.ui.popover = None;
                    cx.notify();
                }),
            ));
        } else if session.is_archived() {
            content = content
                .child(menu_row(
                    "Revive",
                    colors,
                    cx.listener({
                        let id = id.clone();
                        move |this, _, _, cx| {
                            this.store
                                .write()
                                .expect("session store lock poisoned")
                                .revive_sessions(vec![id.clone()]);
                            this.ui.popover = None;
                            cx.notify();
                        }
                    }),
                ))
                .child(menu_row(
                    "Remove from Sidebar",
                    colors,
                    cx.listener({
                        let id = id.clone();
                        move |this, _, _, cx| {
                            this.close_sessions(vec![id.clone()], cx);
                            this.ui.popover = None;
                            cx.notify();
                        }
                    }),
                ))
                .child(menu_divider(colors))
                .child(copy_session_id_row(id, colors, cx));
        } else {
            let running = !matches!(session.status, homie_proto::SessionStatus::Exited(_));
            if !running && session.resumability == homie_proto::Resumability::Resumable {
                content = content.child(menu_row(
                    "Resume",
                    colors,
                    cx.listener({
                        let id = id.clone();
                        move |this, _, _, cx| {
                            this.store
                                .read()
                                .expect("session store lock poisoned")
                                .resume(id.clone());
                            this.ui.popover = None;
                            cx.notify();
                        }
                    }),
                ));
            }
            // Session handoff (Claude only): local sessions offer "Move to
            // <host>", remote ones "Move to Local". Hidden while a move is
            // in flight so a double-click can't queue a second migration.
            if session.kind == ProtoAgentKind::CLAUDE_CODE && !hosts.is_empty() && !migrating {
                if let Some(current) = &session.host {
                    if hosts.iter().any(|entry| &entry.id == current) {
                        content = content.child(menu_row(
                            "Move to Local",
                            colors,
                            cx.listener({
                                let id = id.clone();
                                move |this, _, _, cx| {
                                    this.store
                                        .write()
                                        .expect("session store lock poisoned")
                                        .migrate_session(id.clone(), None);
                                    this.ui.popover = None;
                                    cx.notify();
                                }
                            }),
                        ));
                    }
                } else {
                    for entry in &hosts {
                        let target = entry.id.clone();
                        content = content.child(menu_row(
                            format!("Move to {}", entry.display_name()),
                            colors,
                            cx.listener({
                                let id = id.clone();
                                move |this, _, _, cx| {
                                    this.store
                                        .write()
                                        .expect("session store lock poisoned")
                                        .migrate_session(id.clone(), Some(target.clone()));
                                    this.ui.popover = None;
                                    cx.notify();
                                }
                            }),
                        ));
                    }
                }
            }
            let rename_session = session.clone();
            content = content
                .child(menu_row(
                    // Shells/Cursor can't resume a conversation — archiving
                    // still works, but say what reviving will get you.
                    if session.resumability == homie_proto::Resumability::NotResumable {
                        "Archive (won't be resumable)"
                    } else {
                        "Archive Session"
                    },
                    colors,
                    cx.listener({
                        let id = id.clone();
                        move |this, _, _, cx| {
                            this.archive_sessions(vec![id.clone()]);
                            this.ui.popover = None;
                            cx.notify();
                        }
                    }),
                ))
                .child(menu_row(
                    "Rename…",
                    colors,
                    cx.listener(move |this, _, window, cx| {
                        this.ui.popover = None;
                        this.begin_rename(&rename_session, window, cx);
                    }),
                ))
                .child(menu_row(
                    if pinned {
                        "Unpin Session"
                    } else {
                        "Pin Session"
                    },
                    colors,
                    cx.listener({
                        let id = id.clone();
                        move |this, _, _, cx| {
                            let _ = this
                                .store
                                .write()
                                .expect("session store lock poisoned")
                                .toggle_session_pin(id.clone());
                            this.ui.popover = None;
                            cx.notify();
                        }
                    }),
                ))
                .child(menu_row(
                    "Remove from Sidebar",
                    colors,
                    cx.listener({
                        let id = id.clone();
                        move |this, _, _, cx| {
                            this.close_sessions(vec![id.clone()], cx);
                            this.ui.popover = None;
                            cx.notify();
                        }
                    }),
                ))
                .child(menu_divider(colors))
                .child(copy_session_id_row(id, colors, cx));
        }
        self.popover_shell_at(position, Anchor::TopLeft, 220.0, content, colors, cx)
    }
}
