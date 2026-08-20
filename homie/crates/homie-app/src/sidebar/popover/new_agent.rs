use super::*;

impl Sidebar {
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
}
