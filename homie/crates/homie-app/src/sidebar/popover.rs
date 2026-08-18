use super::*;

impl Sidebar {
    pub(crate) fn popover(
        &self,
        colors: SemanticColors,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        match self.ui.popover.clone()? {
            Popover::NewAgent { directory, host } => {
                Some(self.new_agent_popover(directory, host, colors, cx))
            }
            Popover::Account => Some(self.account_popover(colors, window, cx)),
            Popover::ProjectActions { id, position } => {
                Some(self.project_actions_popover(id, position, colors, cx))
            }
            Popover::SessionActions { id, position } => {
                Some(self.session_actions_popover(id, position, colors, cx))
            }
        }
    }

    pub(crate) fn popover_shell(
        &self,
        top: f32,
        child: impl IntoElement,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.popover_shell_at(
            point(px(12.0), px(top)),
            Anchor::TopLeft,
            244.0,
            child,
            colors,
            cx,
        )
    }

    /// Anchors above the account footer (Swift: popover opens upward).
    pub(crate) fn popover_shell_above_footer(
        &self,
        child: impl IntoElement,
        colors: SemanticColors,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let footer_top = f32::from(window.viewport_size().height) - 44.0;
        self.popover_shell_at(
            point(px(12.0), px(footer_top)),
            Anchor::BottomLeft,
            244.0,
            child,
            colors,
            cx,
        )
    }

    /// Menu-style floating panel: the palette's FloatingSurface recipe, a
    /// sidebar-wide scrim so stray clicks only dismiss, and mouse-down-out so
    /// clicking anywhere else in the window also dismisses. The panel itself
    /// is deferred + anchored in window coordinates so it escapes the sidebar
    /// wrapper's overflow clip and never gets cut off at narrow widths.
    pub(crate) fn popover_shell_at(
        &self,
        position: Point<Pixels>,
        anchor: Anchor,
        width: f32,
        child: impl IntoElement,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .absolute()
            .inset_0()
            .occlude()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.ui.popover = None;
                    cx.notify();
                }),
            )
            .child(
                deferred(
                    anchored()
                        .position(position)
                        .anchor(anchor)
                        .snap_to_window_with_margin(px(8.0))
                        .child(
                            div()
                                .w(px(width))
                                .occlude()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|_, _, _, cx| cx.stop_propagation()),
                                )
                                .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                                    this.ui.popover = None;
                                    cx.notify();
                                }))
                                .child(FloatingSurface::new(
                                    colors,
                                    div()
                                        .rounded(px(Radius::PANEL))
                                        .overflow_hidden()
                                        .py(px(4.0))
                                        .child(child),
                                )),
                        ),
                )
                .with_priority(1),
            )
            .into_any_element()
    }

    pub(crate) fn new_agent_popover(
        &self,
        directory: Option<String>,
        host: Option<String>,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (local_target, default_kind, hosts, active_session, repo_state, syncing, options) = {
            let store = self.store.read().expect("session store lock poisoned");
            let selected_host_id = host.as_deref();
            (
                directory
                    .clone()
                    .unwrap_or_else(|| store.default_new_agent_directory()),
                store.preferences().default_agent.kind(),
                store.hosts().to_vec(),
                store.selected_session().cloned(),
                store.repo_target(selected_host_id).cloned(),
                store.syncing_prefs().clone(),
                agent_picker_options(store.agent_catalog()),
            )
        };
        let selected_host = host
            .clone()
            .and_then(|id| hosts.iter().find(|entry| entry.id == id).cloned());
        let active_host = active_session
            .as_ref()
            .and_then(|session| session.host.clone());
        // A selected remote host owns its cwd. Repo preservation is useful
        // only for returning from a remote session to the corresponding local
        // checkout; it must never override a remote host's default cwd.
        let preserve_repo = should_resolve_active_repo(
            directory.as_deref(),
            selected_host.as_ref().map(|host| host.id.as_str()),
            active_host.as_deref(),
        );
        // Fallback target when the repo isn't resolvable: the host's default
        // cwd remotely; locally the active project (or, for a remote active
        // session, the first project that exists on this machine).
        let fallback_target = match &selected_host {
            Some(host) => remote_picker_target(directory.as_deref(), host.default_cwd.as_deref()),
            None if directory.is_none() && active_host.is_some() => self
                .store
                .read()
                .expect("session store lock poisoned")
                .local_fallback_directory(),
            None => local_target,
        };
        let repo_name = active_session.as_ref().map(|session| {
            session
                .cwd
                .rsplit('/')
                .next()
                .unwrap_or(&session.cwd)
                .to_owned()
        });
        let (target, subtitle) = if preserve_repo {
            match repo_state {
                Some(crate::store::RepoTarget::Resolved(path)) => (path, None),
                Some(crate::store::RepoTarget::Pending) => {
                    (fallback_target, Some("locating repo…".to_owned()))
                }
                Some(crate::store::RepoTarget::NotCloned) => {
                    let place = selected_host
                        .as_ref()
                        .map_or_else(|| "this Mac".to_owned(), |h| h.display_name().to_owned());
                    let folder = fallback_target
                        .rsplit('/')
                        .next()
                        .unwrap_or(&fallback_target)
                        .to_owned();
                    (
                        fallback_target,
                        Some(format!(
                            "{} not on {place} — opens in {folder}",
                            repo_name.as_deref().unwrap_or("repo")
                        )),
                    )
                }
                _ => (fallback_target, None),
            }
        } else {
            (fallback_target, None)
        };
        let folder = target.rsplit('/').next().unwrap_or(&target).to_owned();
        let location = selected_host.as_ref().map_or_else(
            || "This Mac".to_owned(),
            |host| host.display_name().to_owned(),
        );
        let mut header = div()
            .px(px(12.0))
            .pt(px(10.0))
            .pb(px(8.0))
            .flex()
            .flex_col()
            .gap(px(3.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_size(px(Typo::ROW_EMPHASIZED.size))
                            .font_weight(Typo::ROW_EMPHASIZED.weight)
                            .text_color(colors.primary)
                            .child("New Agent"),
                    )
                    .child(
                        div()
                            .min_w(px(0.0))
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .text_size(px(Typo::META.size))
                            .text_color(colors.secondary)
                            .child(location),
                    ),
            )
            .child({
                let browse_host = selected_host.as_ref().map(|entry| entry.id.clone());
                let browse_target = target.clone();
                div()
                    .id("new-agent-directory")
                    .px(px(4.0))
                    .py(px(3.0))
                    .ml(px(-4.0))
                    .flex()
                    .items_center()
                    .gap(px(5.0))
                    .rounded(px(Radius::CHIP))
                    .cursor_pointer()
                    .hover(move |row| row.bg(colors.primary.alpha(0.06)))
                    .text_size(px(Typo::META.size))
                    .text_color(colors.secondary)
                    .child(sf_symbol("folder.fill", 11.0, colors.secondary))
                    .child(
                        div()
                            .min_w(px(0.0))
                            .flex_1()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(folder),
                    )
                    .child(sf_symbol("chevron.right", 9.0, colors.tertiary))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.directory_picker_open = true;
                        this.directory_scroll = ScrollHandle::new();
                        // Pin the currently visible target. A concurrent repo
                        // lookup must not switch the directory underneath an
                        // open browser and leave it waiting on the wrong key.
                        this.ui.popover = Some(Popover::NewAgent {
                            directory: Some(browse_target.clone()),
                            host: browse_host.clone(),
                        });
                        this.store
                            .write()
                            .expect("session store lock poisoned")
                            .request_directory_listing(browse_host.clone(), browse_target.clone());
                        cx.notify();
                    }))
            });
        if let Some(subtitle) = subtitle {
            // Repo-resolution state: "locating repo…" or the visible fallback
            // ("anara not on Forge — opens in code").
            header = header.child(
                div()
                    .text_size(px(Typo::META.size))
                    .text_color(colors.tertiary)
                    .child(subtitle),
            );
        }
        let mut content = div()
            .flex()
            .flex_col()
            .child(header)
            .child(HairlineDivider::horizontal(colors));
        // Host selector — only when hosts.json configures remote hosts:
        // "This Mac" plus one row per host, checkmark on the selection.
        //
        // This list is the single surface that owns the persisted shortcut
        // destination, so it must always be able to undo itself: "This Mac" is
        // the first row and is a real target, not just the absence of one, and
        // it is worded exactly like the destination printed on the always-
        // visible New Agent row ("Claude Code · This Mac · ⌘T") so the label a
        // user reads is the label they come here to change.
        if !hosts.is_empty() {
            content = content.child(
                div()
                    .px(px(12.0))
                    .pt(px(7.0))
                    .pb(px(3.0))
                    .text_size(px(Typo::SECTION_HEADER.size))
                    .font_weight(Typo::SECTION_HEADER.weight)
                    .text_color(colors.tertiary)
                    .child("Run shortcuts on"),
            );
            let mut targets: Vec<(Option<String>, String, &'static str)> =
                vec![(None, "This Mac".to_owned(), "desktopcomputer")];
            for entry in &hosts {
                targets.push((
                    Some(entry.id.clone()),
                    entry.display_name().to_owned(),
                    "network",
                ));
            }
            for (index, (target_host, label, symbol)) in targets.into_iter().enumerate() {
                let selected =
                    target_host.as_deref() == selected_host.as_ref().map(|entry| entry.id.as_str());
                let directory = directory.clone();
                let previous_host = host.clone();
                let active_host = active_host.clone();
                let sync_host = target_host.clone();
                let is_syncing = sync_host.as_deref().is_some_and(|id| syncing.contains(id));
                content = content.child(
                    div()
                        .id(format!("host-option-{index}"))
                        .debug_selector(move || format!("HOST_OPTION_{index}"))
                        .mx(px(6.0))
                        .my(px(1.0))
                        .px(px(8.0))
                        .h(px(28.0))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .rounded(px(Radius::ROW))
                        .cursor_pointer()
                        .hover(move |element| element.bg(colors.primary.alpha(0.06)))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            cx.stop_propagation();
                            let next_directory = if target_host != previous_host {
                                None
                            } else {
                                directory.clone()
                            };
                            // Choosing a row here is an explicit, persistent
                            // setting, not last-used memory: one destination
                            // drives this spawn, ⌘T, ⌥⌘T and the palette, so
                            // the checkmark never disagrees with where a
                            // shortcut lands. Persisting is only defensible
                            // because the choice stays visible (the New Agent
                            // row and every palette title name the target) and
                            // reversible (the "This Mac" row above sets it
                            // back, and a host deleted from hosts.json is
                            // repaired to local on load).
                            this.store
                                .write()
                                .expect("session store lock poisoned")
                                .set_default_spawn_host(target_host.clone());
                            // Only remote -> local needs a matching checkout.
                            // A remote destination starts at its configured cwd.
                            if should_resolve_active_repo(
                                next_directory.as_deref(),
                                target_host.as_deref(),
                                active_host.as_deref(),
                            ) {
                                this.store
                                    .write()
                                    .expect("session store lock poisoned")
                                    .request_repo_target(target_host.clone());
                            }
                            this.directory_picker_open = false;
                            this.ui.popover = Some(Popover::NewAgent {
                                directory: next_directory,
                                host: target_host.clone(),
                            });
                            cx.notify();
                        }))
                        .child(sf_symbol(symbol, 11.0, colors.secondary))
                        .child(
                            div()
                                .flex_1()
                                .text_size(px(Typo::ROW.size))
                                .text_color(colors.primary)
                                .child(label),
                        )
                        .when(selected && sync_host.is_some(), |row| {
                            // Push local agent prefs to this host (rsync over
                            // ssh, daemon-side). Spins tertiary while running.
                            let sync_host = sync_host.clone();
                            row.child(
                                div()
                                    .id(format!("host-sync-{index}"))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .size(px(18.0))
                                    .rounded(px(Radius::CHIP))
                                    .cursor_pointer()
                                    .hover(move |element| element.bg(colors.primary.alpha(0.08)))
                                    .child(sf_symbol(
                                        "arrow.triangle.2.circlepath",
                                        10.0,
                                        if is_syncing {
                                            colors.tertiary
                                        } else {
                                            colors.secondary
                                        },
                                    ))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        cx.stop_propagation();
                                        if let Some(host) = sync_host.clone() {
                                            this.store
                                                .write()
                                                .expect("session store lock poisoned")
                                                .sync_prefs(host);
                                        }
                                        cx.notify();
                                    })),
                            )
                        })
                        .when(selected, |row| {
                            row.child(sf_symbol_weighted(
                                "checkmark",
                                10.0,
                                SymbolWeight::Semibold,
                                colors.secondary,
                            ))
                        }),
                );
            }
            content = content.child(HairlineDivider::horizontal(colors));
        }
        if self.directory_picker_open {
            content = content.child(self.directory_picker(
                selected_host.as_ref().map(|entry| entry.id.clone()),
                target,
                colors,
                cx,
            ));
            return self.popover_shell_at(
                point(px(12.0), px(70.0)),
                Anchor::TopLeft,
                320.0,
                content.pb(px(6.0)),
                colors,
                cx,
            );
        }
        // Carried on repo-preserving spawns so the daemon re-resolves the
        // checkout itself (covers a click that lands while still "locating").
        let same_repo_reference = if preserve_repo {
            active_session.as_ref().map(|session| session.id.clone())
        } else {
            None
        };
        for (index, (title, kind, shortcut)) in options.into_iter().enumerate() {
            let row_id = format!("agent-option-{index}");
            let target = target.clone();
            let spawn_host = selected_host.as_ref().map(|entry| entry.id.clone());
            let same_repo_as = same_repo_reference.clone();
            // The picker selection is also the global shortcut destination,
            // so every shortcut stays visible and follows the checkmark.
            let shortcut = agent_picker_shortcut(&kind, &default_kind, shortcut);
            let shortcut = shortcut.to_owned();
            let agent_kind = ui_agent_kind(&kind);
            let spawn_kind = kind.clone();
            content = content.child(
                div()
                    .id(row_id)
                    .debug_selector(move || format!("AGENT_OPTION_{index}"))
                    .mx(px(6.0))
                    .my(px(1.0))
                    .px(px(8.0))
                    .h(px(34.0))
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .rounded(px(Radius::ROW))
                    .cursor_pointer()
                    .hover(move |element| element.bg(colors.primary.alpha(0.06)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.store
                            .write()
                            .expect("session store lock poisoned")
                            .spawn_kind(
                                spawn_kind.clone(),
                                SpawnOptions {
                                    cwd: Some(target.clone()),
                                    host: spawn_host.clone(),
                                    same_repo_as: same_repo_as.clone(),
                                    ..SpawnOptions::default()
                                },
                            );
                        this.ui.popover = None;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .w(px(24.0))
                            .flex_none()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(AgentLogo::new(agent_kind, 20.0, colors).badged(false)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(Typo::ROW.size))
                            .text_color(colors.primary)
                            .child(title),
                    )
                    .when(!shortcut.is_empty(), |row| {
                        row.child(
                            div()
                                .px(px(5.0))
                                .py(px(2.0))
                                .rounded(px(Radius::CHIP))
                                .bg(Fill::subtle(colors))
                                .text_size(px(Typo::META.size))
                                .font_weight(Typo::META.weight)
                                .text_color(colors.tertiary)
                                .child(shortcut),
                        )
                    }),
            );
        }
        self.popover_shell(70.0, content.pb(px(6.0)), colors, cx)
    }

    pub(crate) fn directory_picker(
        &self,
        host: Option<String>,
        requested_path: String,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let state = self
            .store
            .read()
            .expect("session store lock poisoned")
            .directory_listing(host.as_deref(), &requested_path)
            .cloned();
        let mut panel = div().flex().flex_col();
        match state {
            Some(DirectoryListingState::Ready(result)) => {
                let use_path = result.path.clone();
                let use_host = host.clone();
                panel = panel.child(
                    div()
                        .px(px(10.0))
                        .py(px(7.0))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .min_w(px(0.0))
                                .flex_1()
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .text_ellipsis()
                                .text_size(px(Typo::META_MONO.size))
                                .font_family(crate::fonts::mono_family())
                                .text_color(colors.secondary)
                                .child(result.path.clone()),
                        )
                        .child(
                            div()
                                .id("use-new-agent-directory")
                                .px(px(8.0))
                                .py(px(4.0))
                                .rounded(px(Radius::CHIP))
                                .bg(Ink::FRESH)
                                .text_size(px(Typo::META.size))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors.background)
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.directory_picker_open = false;
                                    this.ui.popover = Some(Popover::NewAgent {
                                        directory: Some(use_path.clone()),
                                        host: use_host.clone(),
                                    });
                                    cx.notify();
                                }))
                                .child("Use folder"),
                        ),
                );
                let mut rows = div()
                    .id("directory-picker-list")
                    .track_scroll(&self.directory_scroll)
                    .max_h(px(260.0))
                    .overflow_y_scroll()
                    .flex()
                    .flex_col();
                if let Some(parent) = result.parent {
                    let parent_host = host.clone();
                    rows = rows.child(directory_row(
                        "arrow.up",
                        "Parent folder".to_owned(),
                        colors,
                        cx.listener(move |this, _, _, cx| {
                            this.directory_scroll = ScrollHandle::new();
                            this.ui.popover = Some(Popover::NewAgent {
                                directory: Some(parent.clone()),
                                host: parent_host.clone(),
                            });
                            this.store
                                .write()
                                .expect("session store lock poisoned")
                                .request_directory_listing(parent_host.clone(), parent.clone());
                            cx.notify();
                        }),
                    ));
                }
                for entry in result.entries {
                    let next_host = host.clone();
                    let next_path = entry.path;
                    rows = rows.child(directory_row(
                        "folder",
                        entry.name,
                        colors,
                        cx.listener(move |this, _, _, cx| {
                            this.directory_scroll = ScrollHandle::new();
                            this.ui.popover = Some(Popover::NewAgent {
                                directory: Some(next_path.clone()),
                                host: next_host.clone(),
                            });
                            this.store
                                .write()
                                .expect("session store lock poisoned")
                                .request_directory_listing(next_host.clone(), next_path.clone());
                            cx.notify();
                        }),
                    ));
                }
                if result.truncated {
                    rows = rows.child(
                        div()
                            .px(px(12.0))
                            .py(px(7.0))
                            .text_size(px(Typo::META.size))
                            .text_color(colors.tertiary)
                            .child("Showing the first 512 folders"),
                    );
                }
                panel = panel.child(HairlineDivider::horizontal(colors)).child(rows);
            }
            Some(DirectoryListingState::Error(error)) => {
                let retry_host = host.clone();
                let retry_path = requested_path.clone();
                panel = panel.child(
                    div()
                        .px(px(12.0))
                        .py(px(12.0))
                        .flex()
                        .items_start()
                        .gap(px(8.0))
                        .text_size(px(Typo::META.size))
                        .text_color(colors.secondary)
                        .child(sf_symbol(
                            "exclamationmark.triangle.fill",
                            12.0,
                            Ink::ATTENTION,
                        ))
                        .child(div().min_w(px(0.0)).flex_1().child(error))
                        .child(
                            div()
                                .id("retry-directory-listing")
                                .cursor_pointer()
                                .text_color(Ink::FRESH)
                                .child("Retry")
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.store
                                        .write()
                                        .expect("session store lock poisoned")
                                        .request_directory_listing(
                                            retry_host.clone(),
                                            retry_path.clone(),
                                        );
                                    cx.notify();
                                })),
                        ),
                );
            }
            Some(DirectoryListingState::Loading) | None => {
                panel = panel.child(
                    div()
                        .px(px(12.0))
                        .py(px(16.0))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .text_size(px(Typo::META.size))
                        .text_color(colors.secondary)
                        .child(LoadingIndicator::new(
                            "directory-listing-loading",
                            14.0,
                            colors.tertiary,
                        ))
                        .child("Loading folders…"),
                );
            }
        }
        panel.into_any_element()
    }

    /// Version line in the account popover, doubling as the manual check.
    ///
    /// Whatever the pill is showing wins here, so the popover never contradicts
    /// the footer two pixels above it; with nothing pending it falls back to
    /// the running version and a click starts a check.
    pub(crate) fn update_menu_row(
        &self,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let unsupported = matches!(self.update.phase, UpdatePhase::Unsupported(_));
        let command = match &self.update.phase {
            UpdatePhase::Available(_) => Some(UpdateCommand::Download),
            UpdatePhase::Ready(_) => Some(UpdateCommand::Install),
            UpdatePhase::Checking | UpdatePhase::Downloading { .. } | UpdatePhase::Installing => {
                None
            }
            _ if unsupported => None,
            _ => Some(UpdateCommand::Check {
                user_initiated: true,
            }),
        };
        let label = if self.preview {
            format!("homie {}", crate::updates::CURRENT_VERSION)
        } else {
            self.update.summary()
        };
        let mut row = div()
            .id("account-version")
            .mx(px(6.0))
            .px(px(8.0))
            .h(px(28.0))
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.0))
            .rounded(px(Radius::ROW))
            .text_size(px(Typo::ROW.size))
            .text_color(if unsupported {
                colors.tertiary
            } else {
                colors.primary
            })
            .child(
                div()
                    .min_w(px(0.0))
                    .flex_1()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(label),
            );
        if let Some(command) = command {
            let action = match command {
                UpdateCommand::Download => "Download",
                UpdateCommand::Install => "Restart",
                _ => "Check",
            };
            row = row
                .cursor_pointer()
                .hover(move |element| element.bg(colors.primary.alpha(0.06)))
                .child(
                    div()
                        .flex_none()
                        .text_size(px(Typo::META.size))
                        .text_color(colors.secondary)
                        .child(action),
                )
                .on_click(cx.listener(move |this, _, _, cx: &mut Context<Self>| {
                    cx.emit(SidebarEvent::Update(command.clone()));
                    this.ui.popover = None;
                    cx.notify();
                }));
        }
        row.into_any_element()
    }

    pub(crate) fn account_popover(
        &self,
        colors: SemanticColors,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut usage = div()
            .flex()
            .flex_col()
            .child(section_label("Usage", colors));
        if self.preview {
            usage = usage
                .child(usage_row("Session", "resets in 2h 14m", "$2.31", colors))
                .child(usage_row("Today", "1.8M tokens", "$4.82", colors))
                .child(usage_row("This month", "", "$86.40", colors));
        } else if let Some(snapshot) = self.usage {
            usage = usage
                .child(usage_row(
                    "Session",
                    snapshot
                        .session_remaining_seconds
                        .map(|seconds| format!("resets in {}", compact_duration(seconds)))
                        .as_deref()
                        .unwrap_or("idle"),
                    &snapshot
                        .session_cost
                        .map(UsageFormat::money)
                        .unwrap_or_else(|| "—".into()),
                    colors,
                ))
                .child(usage_row(
                    "Today",
                    &format!(
                        "{} tokens",
                        UsageFormat::tokens(snapshot.today().total_tokens())
                    ),
                    &UsageFormat::money(snapshot.today().cost),
                    colors,
                ))
                .child(usage_row(
                    "This month",
                    "",
                    &UsageFormat::money(snapshot.month().cost),
                    colors,
                ));
        } else {
            usage = usage.child(
                div()
                    .px(px(14.0))
                    .py(px(6.0))
                    .text_size(px(Typo::ROW.size))
                    .text_color(colors.tertiary)
                    .child("Measuring…"),
            );
        }
        let content = div()
            .flex()
            .flex_col()
            .child(usage)
            .child(div().mt(px(8.0)).h(px(1.0)).bg(colors.primary.alpha(0.06)))
            .child(section_label("Version", colors))
            .child(self.update_menu_row(colors, cx))
            .child(div().mt(px(8.0)).h(px(1.0)).bg(colors.primary.alpha(0.06)))
            .child(section_label("Remote", colors))
            .child(
                div()
                    .id("quick-add-remote-host")
                    .debug_selector(|| "quick-add-remote-host".into())
                    .mx(px(6.0))
                    .px(px(8.0))
                    .h(px(30.0))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .rounded(px(Radius::ROW))
                    .cursor_pointer()
                    .hover(move |element| element.bg(colors.primary.alpha(0.06)))
                    .text_size(px(Typo::ROW.size))
                    .text_color(colors.primary)
                    .child(sf_symbol("plus", 11.0, colors.secondary))
                    .child(div().flex_1().child("Add remote host"))
                    .child(
                        div()
                            .text_size(px(Typo::META.size))
                            .text_color(colors.tertiary)
                            .child("SSH"),
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.ui.popover = None;
                        cx.emit(SidebarEvent::AddRemoteHost);
                        cx.notify();
                    })),
            )
            .child(div().mt(px(8.0)).h(px(1.0)).bg(colors.primary.alpha(0.06)))
            .child(section_label("Account", colors))
            .child(
                div()
                    .id("account-active")
                    .mx(px(6.0))
                    .px(px(8.0))
                    .h(px(28.0))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .rounded(px(Radius::ROW))
                    .text_size(px(Typo::ROW.size))
                    .text_color(colors.primary)
                    .child(sf_symbol_weighted(
                        "checkmark",
                        10.0,
                        SymbolWeight::Semibold,
                        colors.secondary,
                    ))
                    .child(if self.preview {
                        "preview@homie.local"
                    } else {
                        "Local agents"
                    }),
            )
            .child(
                div()
                    .id("dismiss-account")
                    .mx(px(6.0))
                    .my(px(6.0))
                    .px(px(8.0))
                    .h(px(28.0))
                    .flex()
                    .items_center()
                    .rounded(px(Radius::ROW))
                    .cursor_pointer()
                    .hover(move |element| element.bg(colors.primary.alpha(0.06)))
                    .text_size(px(Typo::ROW.size))
                    .text_color(colors.secondary)
                    .child("Done")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.ui.popover = None;
                        cx.notify();
                    })),
            );
        self.popover_shell_above_footer(content, colors, window, cx)
    }

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

    /// Right-click context menu for a session row, anchored at the click.
    /// Mirrors the Swift SessionContextMenu, limited to actions the Rust
    /// store implements.
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
