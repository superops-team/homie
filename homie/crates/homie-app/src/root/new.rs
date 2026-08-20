use super::*;

impl RootView {
    pub(crate) fn new(
        services: Arc<AppServices>,
        preview: bool,
        preview_scenario: PreviewScenario,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let sidebar_runtime = (!preview).then(|| Arc::clone(&services.store));
        let sidebar = cx.new(|cx| Sidebar::new(sidebar_runtime, preview, preview_scenario, cx));
        let terminal = (!preview).then(|| {
            let runtime = Arc::clone(&services.store);
            let tokio = Arc::clone(&services.tokio);
            cx.new(|cx| TerminalPane::new(runtime, tokio, window, cx))
        });
        let navigation = (!preview).then(|| {
            let runtime = Arc::clone(&services.store);
            cx.new(|cx| NavigationOverlay::new(runtime, window, cx))
        });
        let session_surfaces = (!preview).then(|| {
            let runtime = Arc::clone(&services.store);
            cx.new(|cx| SessionSurfaces::new(runtime, cx))
        });
        let utility_surfaces = (!preview).then(|| {
            let runtime = Arc::clone(&services.store);
            let tokio = Arc::clone(&services.tokio);
            let updates = services.updates.clone();
            cx.new(|cx| UtilitySurfaces::new(runtime, tokio, updates, window, cx))
        });
        let launcher = cx.new(|cx| LauncherOverlay::new(Arc::clone(&services), preview, cx));
        let inspector = (!preview || preview_scenario == PreviewScenario::Artifacts).then(|| {
            let runtime = Arc::clone(&services.store);
            let tokio = Arc::clone(&services.tokio);
            cx.new(|cx| WorkbenchInspector::new(runtime, tokio, cx))
        });
        if let (Some(terminal), Some(navigation), Some(utility_surfaces)) =
            (&terminal, &navigation, &utility_surfaces)
        {
            let navigation = navigation.clone();
            let utility_surfaces = utility_surfaces.clone();
            terminal.update(cx, |terminal, _| {
                terminal.set_shell_entities(navigation, utility_surfaces);
            });
        }
        if let Some(terminal) = &terminal {
            let terminal = terminal.clone();
            cx.defer_in(window, move |_, window, cx| {
                terminal.update(cx, |terminal, cx| terminal.focus(window, cx));
            });
        }
        let mut child_subscriptions = Vec::new();
        if let Some(terminal) = &terminal {
            child_subscriptions.push(cx.subscribe(terminal, |this, _, event, cx| match event {
                TerminalPaneEvent::ToggleSidebar => {
                    this.sidebar.update(cx, |sidebar, cx| sidebar.toggle(cx));
                }
                TerminalPaneEvent::ToggleInspector => this.toggle_inspector(cx),
                TerminalPaneEvent::OpenFileReference { reference, cwd, .. } => {
                    let inspector = this.inspector.clone();
                    this.reveal_inspector(cx);
                    if let Some(inspector) = inspector {
                        inspector.update(cx, |inspector, cx| {
                            inspector.open_file_reference(cwd.clone(), reference.clone(), cx);
                        });
                    }
                }
            }));
        }
        child_subscriptions.push(cx.subscribe_in(
            &sidebar,
            window,
            |this, _, event, window, cx| {
                if matches!(event, SidebarEvent::SessionActivated)
                    && let Some(terminal) = &this.terminal
                {
                    terminal.update(cx, |terminal, cx| terminal.focus(window, cx));
                    this.sync_auxiliary_terminal(window, cx);
                }
                if let SidebarEvent::Update(command) = event {
                    this.services.updates.send(command.clone());
                }
                if matches!(event, SidebarEvent::OpenSettings)
                    && let Some(surfaces) = &this.utility_surfaces
                {
                    surfaces.update(cx, |surfaces, cx| surfaces.open_settings(cx));
                }
                if matches!(event, SidebarEvent::AddRemoteHost)
                    && let Some(surfaces) = &this.utility_surfaces
                {
                    surfaces.update(cx, |surfaces, cx| {
                        surfaces.open_add_remote_host(window, cx);
                    });
                }
                if matches!(event, SidebarEvent::VisibilityChanged) {
                    this.begin_sidebar_slide(cx);
                }
                cx.notify();
            },
        ));
        child_subscriptions.push(cx.subscribe_in(
            &launcher,
            window,
            |this, _, _: &LauncherEvent, window, cx| {
                if let Some(terminal) = &this.terminal {
                    terminal.update(cx, |terminal, cx| terminal.focus(window, cx));
                } else {
                    window.focus(&this.focus, cx);
                }
                // The launcher is a main-pane destination, so closing it must
                // make RootView swap the terminal branch back into the row.
                cx.notify();
            },
        ));
        if let Some(navigation) = &navigation {
            child_subscriptions.push(cx.subscribe(navigation, |this, _, event, cx| match event {
                NavigationEvent::ToggleSidebar => {
                    this.sidebar.update(cx, |sidebar, cx| sidebar.toggle(cx));
                }
                NavigationEvent::OpenOverview => {
                    if let Some(surfaces) = &this.session_surfaces {
                        surfaces.update(cx, |surfaces, cx| surfaces.open_overview(cx));
                    }
                }
                NavigationEvent::OpenWorktrees => {
                    if let Some(surfaces) = &this.utility_surfaces {
                        surfaces.update(cx, |surfaces, cx| surfaces.open_worktrees(cx));
                    }
                }
                NavigationEvent::OpenSettings => {
                    if let Some(surfaces) = &this.utility_surfaces {
                        surfaces.update(cx, |surfaces, cx| surfaces.open_settings(cx));
                    }
                }
                NavigationEvent::CheckForUpdates => {
                    this.services.updates.check(true);
                }
            }));
        }
        if let Some(inspector) = &inspector {
            child_subscriptions.push(cx.subscribe(inspector, |this, _, event, cx| {
                if matches!(event, InspectorEvent::Close) {
                    this.set_inspector_open(false, cx);
                }
            }));
        }

        let mut status_events = services.store.status_events();
        let mut snapshots = services.store.snapshots();
        let mut usage = services.usage_tx.subscribe();
        let mut updates = services.updates.subscribe();
        sidebar.update(cx, |sidebar, cx| sidebar.set_usage(*usage.borrow(), cx));
        // Seed the current state: `watch` only wakes on changes, and an
        // unsupported build settles before this view exists.
        let initial_update = services.updates.state();
        sidebar.update(cx, |sidebar, cx| sidebar.set_update(initial_update, cx));

        #[cfg(target_os = "macos")]
        let mut menu_bar = objc2_foundation::MainThreadMarker::new()
            .and_then(|mtm| NativeMenuBar::new(mtm, Arc::clone(&services.store.store)));
        #[cfg(target_os = "macos")]
        if let Some(menu_bar) = &mut menu_bar {
            menu_bar.update(&snapshots.borrow());
        }
        #[cfg(target_os = "macos")]
        let notifier = NativeNotifier::new(services.store.notification_action_sender());

        let activation_services = Arc::clone(&services);
        let activation = cx.observe_window_activation(window, move |_this, window, _cx| {
            activation_services
                .store
                .store
                .write()
                .expect("session store lock poisoned")
                .set_active(window.is_window_active());
        });
        let bounds_observer = (!preview).then(|| {
            cx.observe_window_bounds(window, |this, window, cx| {
                this.window_bounds_changed(window, cx);
            })
        });

        let service_sidebar = sidebar.clone();
        let service_events = cx.spawn(async move |this, cx| {
            loop {
                tokio::select! {
                    status = status_events.recv() => {
                        let Ok(status) = status else { break };
                        let _ = this.update(cx, |this, cx| {
                            #[cfg(target_os = "macos")]
                            let app_is_active = this
                                .services
                                .store
                                .store
                                .read()
                                .expect("session store lock poisoned")
                                .app_is_active();
                            if let Some(sound) = status.sound {
                                let sound = match sound {
                                    NotificationSound::NeedsInput => StatusSound::NeedsInput,
                                    NotificationSound::Done => StatusSound::Done,
                                    NotificationSound::Frozen => StatusSound::Frozen,
                                };
                                if this.sound_gate.should_play(sound, Instant::now()) {
                                    let _ = sounds::play(&AfplayPlayer, sound);
                                }
                            }
                            #[cfg(target_os = "macos")]
                            if let Some(notification) = &status.notification
                                && (!app_is_active || status.in_app_banner.is_none())
                            {
                                this.notifier.post(notification);
                            }
                            if let Some(banner) = status.in_app_banner {
                                this.status_banner_generation =
                                    this.status_banner_generation.wrapping_add(1);
                                let generation = this.status_banner_generation;
                                this.status_banner = Some(banner);
                                cx.notify();
                                cx.spawn(async move |this, cx| {
                                    cx.background_executor()
                                        .timer(Duration::from_secs(7))
                                        .await;
                                    let _ = this.update(cx, |this, cx| {
                                        if this.status_banner_generation == generation {
                                            this.status_banner = None;
                                            cx.notify();
                                        }
                                    });
                                })
                                .detach();
                            }
                        });
                    }
                    changed = snapshots.changed() => {
                        if changed.is_err() { break; }
                        let snapshot = snapshots.borrow_and_update().clone();
                        let _ = this.update(cx, |this, _cx| {
                            #[cfg(target_os = "macos")]
                            if let Some(menu_bar) = &mut this.menu_bar {
                                menu_bar.update(&snapshot);
                            }
                        });
                    }
                    changed = usage.changed() => {
                        if changed.is_err() { break; }
                        let snapshot = *usage.borrow_and_update();
                        service_sidebar.update(cx, |sidebar, cx| {
                            sidebar.set_usage(snapshot, cx);
                        });
                    }
                    changed = updates.changed() => {
                        if changed.is_err() { break; }
                        let state = updates.borrow_and_update().clone();
                        let installing = state.phase == UpdatePhase::Installing;
                        service_sidebar.update(cx, |sidebar, cx| {
                            sidebar.set_update(state, cx);
                        });
                        // The swap helper is already polling for this process
                        // to exit; quitting is what lets the install proceed.
                        if installing {
                            cx.update(|cx| cx.quit());
                        }
                    }
                }
            }
        });
        let surface_sync =
            terminal
                .as_ref()
                .zip(session_surfaces.as_ref())
                .map(|(terminal, surfaces)| {
                    let terminal = terminal.clone();
                    let surfaces = surfaces.clone();
                    let mut changes = services.store.changes();
                    cx.spawn(async move |_this, cx| {
                        loop {
                            match changes.recv().await {
                                Ok(())
                                | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                                    let buffers = terminal
                                        .update(cx, |terminal, _| terminal.resident_buffers());
                                    surfaces.update(cx, |surfaces, _| {
                                        surfaces.sync_resident_buffers(buffers);
                                    });
                                }
                                Err(tokio::sync::broadcast::error::RecvError::Closed) => return,
                            }
                        }
                    })
                });
        let mut workbench_changes = services.store.changes();
        let workbench_sync = cx.spawn_in(window, async move |this, cx| {
            loop {
                match workbench_changes.recv().await {
                    Ok(()) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        if this
                            .update_in(cx, |this, window, cx| {
                                this.sync_auxiliary_terminal(window, cx);
                            })
                            .is_err()
                        {
                            return;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return,
                }
            }
        });
        let (workbench_layout, inspector_open, inspector_width) = {
            let store = services
                .store
                .store
                .read()
                .expect("session store lock poisoned");
            let prefs = store.preferences();
            (
                WorkbenchLayout::from_fraction(prefs.workbench_primary_fraction),
                prefs.inspector_open,
                prefs.inspector_width,
            )
        };
        if inspector_open && let Some(inspector) = &inspector {
            inspector.update(cx, |inspector, cx| inspector.set_visible(true, cx));
        }
        // Seed both seams from the restored layout so the first frame paints
        // the settled panels instead of sliding them open at launch.
        let sidebar_seam = if sidebar.read(cx).is_visible() {
            sidebar.read(cx).width()
        } else {
            0.0
        };
        let inspector_seam = if inspector_open { inspector_width } else { 0.0 };
        let mut root = Self {
            sidebar,
            terminal,
            navigation,
            session_surfaces,
            utility_surfaces,
            launcher,
            inspector,
            services,
            focus: cx.focus_handle(),
            resize_origin: None,
            sidebar_slide: None,
            sidebar_seam,
            auxiliary_terminal: None,
            auxiliary_id: None,
            auxiliary_parent: None,
            auxiliary_spawn_parent: None,
            collapsed_auxiliary_parents: HashSet::new(),
            workbench_layout,
            terminal_resize_origin: None,
            terminal_available_height: 0.0,
            inspector_open,
            inspector_width,
            inspector_max_width: 720.0,
            inspector_slide: None,
            inspector_seam,
            inspector_toggled_at: None,
            inspector_resize_origin: None,
            window_bounds_save: None,
            status_banner: None,
            status_banner_generation: 0,
            sound_gate: SoundGate::default(),
            preview,
            preview_scenario,
            #[cfg(target_os = "macos")]
            menu_bar,
            #[cfg(target_os = "macos")]
            notifier,
            _subscriptions: std::iter::once(activation)
                .chain(bounds_observer)
                .chain(child_subscriptions)
                .collect(),
            _service_events: service_events,
            _surface_sync: surface_sync,
            _workbench_sync: workbench_sync,
        };
        root.sync_auxiliary_terminal(window, cx);
        if !preview {
            // Do not rely on AppKit emitting a move/resize after the observer
            // is installed: even an untouched first launch should become the
            // placement restored by the next launch.
            root.window_bounds_changed(window, cx);
        }
        root
    }
}
