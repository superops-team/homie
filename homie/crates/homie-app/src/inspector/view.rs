use super::*;

impl WorkbenchInspector {
    fn render_header(
        &self,
        session: Option<&SessionRecord>,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let changes_count = match &self.state {
            LoadState::Ready(snapshot) if snapshot.files > 0 => Some(snapshot.files),
            _ => None,
        };
        let artifacts_count = session.map(artifact_count).filter(|count| *count > 0);
        let selected_tab = self.selected_tab;
        let mut tabs = div()
            .min_w(px(0.0))
            .flex_1()
            .flex()
            .items_center()
            .gap(px(2.0));

        for tab in InspectorTab::ALL {
            let count = match tab {
                InspectorTab::Info => None,
                InspectorTab::Changes => changes_count,
                InspectorTab::Code => None,
                InspectorTab::Artifacts => artifacts_count,
            };
            let active = tab == selected_tab;
            tabs = tabs.child(
                div()
                    .id(SharedString::from(format!("inspector-tab-{}", tab.label())))
                    .debug_selector(move || tab.debug_selector().to_owned())
                    .h(px(28.0))
                    .px(px(6.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(5.0))
                    .rounded(px(Radius::BADGE))
                    .cursor_pointer()
                    .bg(if active {
                        colors.primary.alpha(0.09)
                    } else {
                        colors.primary.alpha(0.0)
                    })
                    .hover(move |button| {
                        button.bg(colors.primary.alpha(if active { 0.11 } else { 0.055 }))
                    })
                    .text_size(px(12.0))
                    .font_weight(if active {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::MEDIUM
                    })
                    .text_color(if active {
                        colors.primary
                    } else {
                        colors.secondary
                    })
                    .child(tab.label())
                    // Counts are useful context once a destination is open,
                    // but four always-visible badges make the 300pt compact
                    // inspector overlap its close control.
                    .when_some(count.filter(|_| active), |tab, count| {
                        tab.child(
                            div()
                                .min_w(px(16.0))
                                .h(px(16.0))
                                .px(px(4.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded_full()
                                .bg(colors.primary.alpha(if active { 0.10 } else { 0.06 }))
                                .text_size(px(9.5))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(if active {
                                    colors.secondary
                                } else {
                                    colors.tertiary
                                })
                                .child(count.to_string()),
                        )
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.select_tab(tab, cx);
                        cx.stop_propagation();
                    })),
            );
        }

        div()
            .h(px(Metrics::TITLE_BAR))
            .flex_none()
            .pl(px(8.0))
            .pr(px(Metrics::TOOLBAR_EDGE_INSET))
            .flex()
            .items_center()
            .gap(px(Metrics::TOOLBAR_COMPACT_GAP))
            .child(tabs)
            .child(
                div()
                    .id("close-inspector")
                    .debug_selector(|| "INSPECTOR_CLOSE".to_owned())
                    .size(px(Metrics::TOOLBAR_CONTROL_SIZE))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(Radius::BADGE))
                    .cursor_pointer()
                    .hover(move |button| button.bg(Fill::subtle(colors)))
                    .child(sf_symbol_weighted(
                        "xmark",
                        13.5,
                        SymbolWeight::Bold,
                        colors.secondary,
                    ))
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.emit(InspectorEvent::Close);
                        cx.stop_propagation();
                    })),
            )
    }

    fn render_info(
        &mut self,
        session: Option<&SessionRecord>,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(session) = session else {
            return self
                .render_message(
                    colors,
                    "sidebar.left",
                    "Select a session",
                    "Info follows the active agent.",
                )
                .into_any_element();
        };

        let (project_name, host_name) = {
            let store = self
                .runtime
                .store
                .read()
                .expect("session store lock poisoned");
            let project_name = store
                .projects()
                .get(&session.project_id)
                .map(|project| project.name.clone())
                .unwrap_or_else(|| folder_name(&session.cwd));
            let host_name = session
                .host
                .as_deref()
                .map(|host| store.host_display_name(host));
            (project_name, host_name)
        };
        let kind = ui_agent_kind(session.effective_kind());
        let (status_label, status_color) = session_status(session, colors);
        let artifact_total = artifact_count(session);

        let hero = div()
            .p(px(14.0))
            .flex()
            .flex_col()
            .gap(px(12.0))
            .rounded(px(Radius::CARD))
            .bg(colors.primary.alpha(0.035))
            .border_1()
            .border_color(colors.primary.alpha(0.065))
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap(px(11.0))
                    .child(AgentLogo::new(kind, 36.0, colors))
                    .child(
                        div()
                            .min_w(px(0.0))
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(3.0))
                            .child(
                                div()
                                    .text_size(px(Typo::DISPLAY_TITLE.size))
                                    .font_weight(Typo::DISPLAY_TITLE.weight)
                                    .text_color(colors.primary)
                                    .child(session.title.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(5.0))
                                    .text_size(px(Typo::META.size))
                                    .text_color(colors.tertiary)
                                    .child(kind.label())
                                    .child("·")
                                    .child(project_name.clone()),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(7.0))
                            .text_size(px(Typo::META.size))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(status_color)
                            .child(div().size(px(7.0)).rounded_full().bg(status_color))
                            .child(status_label),
                    )
                    .child(
                        div()
                            .text_size(px(Typo::META.size))
                            .text_color(colors.tertiary)
                            .child(format!("Updated {}", relative_time(session.updated_at.0))),
                    ),
            );

        let mut content = div()
            .id("inspector-info-scroll")
            .size_full()
            .min_h(px(0.0))
            .px(px(12.0))
            .pt(px(8.0))
            .pb(px(18.0))
            .flex()
            .flex_col()
            .gap(px(14.0))
            .overflow_y_scroll()
            .child(hero);

        if let Some(detail) = &session.needs_input {
            let risk_color = if detail.risk_hint == homie_proto::RiskHint::Destructive {
                Ink::DANGER
            } else {
                Ink::ATTENTION
            };
            content = content.child(
                div()
                    .p(px(12.0))
                    .flex()
                    .items_start()
                    .gap(px(9.0))
                    .rounded(px(Radius::CARD))
                    .bg(risk_color.alpha(0.10))
                    .border_1()
                    .border_color(risk_color.alpha(0.22))
                    .child(sf_symbol("questionmark.bubble", 15.0, risk_color))
                    .child(
                        div()
                            .min_w(px(0.0))
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(3.0))
                            .child(
                                div()
                                    .text_size(px(Typo::ROW_EMPHASIZED.size))
                                    .font_weight(Typo::ROW_EMPHASIZED.weight)
                                    .text_color(colors.primary)
                                    .child("Needs your input"),
                            )
                            .child(
                                div()
                                    .text_size(px(Typo::META.size))
                                    .text_color(colors.secondary)
                                    .child(detail.summary.clone()),
                            ),
                    ),
            );
        }

        content = content
            .child(section_label("Git status", colors))
            .child(self.render_git_summary(colors, cx));

        if let Some(pull_requests) = session.pull_requests.as_deref()
            && !pull_requests.is_empty()
        {
            content = content.child(section_label(
                if pull_requests.len() == 1 {
                    "Pull request"
                } else {
                    "Pull requests"
                },
                colors,
            ));
            let inspector = cx.entity();
            for pull_request in pull_requests.iter().take(2) {
                let body = pull_request
                    .body
                    .as_deref()
                    .filter(|body| !body.trim().is_empty())
                    .map(|body| self.markdown_document(body));
                content = content.child(render_pull_request(
                    pull_request,
                    colors,
                    inspector.clone(),
                    body,
                ));
            }
        }

        if artifact_total > 0 {
            content = content.child(section_label("Artifacts", colors)).child(
                div()
                    .id("inspector-artifacts-summary")
                    .h(px(44.0))
                    .px(px(11.0))
                    .flex()
                    .items_center()
                    .gap(px(9.0))
                    .rounded(px(Radius::ROW))
                    .bg(colors.primary.alpha(0.035))
                    .border_1()
                    .border_color(colors.primary.alpha(0.06))
                    .cursor_pointer()
                    .hover(move |row| row.bg(colors.primary.alpha(0.065)))
                    .child(sf_symbol("shippingbox", 14.0, colors.secondary))
                    .child(
                        div()
                            .min_w(px(0.0))
                            .flex_1()
                            .text_size(px(Typo::ROW.size))
                            .text_color(colors.primary)
                            .child(format!(
                                "{artifact_total} {} discovered",
                                if artifact_total == 1 {
                                    "artifact"
                                } else {
                                    "artifacts"
                                }
                            )),
                    )
                    .child(sf_symbol("chevron.right", 11.0, colors.tertiary))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.select_tab(InspectorTab::Artifacts, cx);
                        cx.stop_propagation();
                    })),
            );
        }

        let mut details = div()
            .rounded(px(Radius::CARD))
            .bg(colors.primary.alpha(0.025))
            .border_1()
            .border_color(colors.primary.alpha(0.055))
            .overflow_hidden()
            .child(detail_row("Project", project_name, false, colors))
            .child(detail_row("Directory", session.cwd.clone(), true, colors));
        if let Some(branch) = &session.git_branch {
            details = details.child(detail_row("Branch", branch.clone(), true, colors));
        }
        if let Some(host) = host_name {
            details = details.child(detail_row("Host", host, false, colors));
        }
        if let Some(bytes) = session.memory_bytes {
            details = details.child(detail_row("Memory", format_bytes(bytes), false, colors));
        }
        content
            .child(section_label("Details", colors))
            .child(details)
            .into_any_element()
    }

    fn render_git_summary(&self, colors: SemanticColors, cx: &mut Context<Self>) -> AnyElement {
        let (symbol, title, detail, accent, can_open) = match &self.state {
            LoadState::Ready(snapshot) if snapshot.files > 0 => (
                Some("arrow.left.arrow.right"),
                format!(
                    "{} {} changed",
                    snapshot.files,
                    if snapshot.files == 1 { "file" } else { "files" }
                ),
                format!("+{}  −{}", snapshot.additions, snapshot.deletions),
                rgba(0x4f8ef7ff),
                true,
            ),
            LoadState::Ready(snapshot) => (
                Some("checkmark.circle.fill"),
                "No changes".to_owned(),
                format!(
                    "Matches {}",
                    snapshot
                        .base_ref
                        .as_deref()
                        .unwrap_or(match self.comparison {
                            SessionDiffBase::DefaultBranch => "default branch",
                            SessionDiffBase::Head => "HEAD",
                        })
                ),
                Ink::FRESH,
                true,
            ),
            LoadState::Loading => (
                None,
                "Reading working tree".to_owned(),
                "Git status is updating…".to_owned(),
                colors.secondary,
                false,
            ),
            LoadState::Error(error) if git_is_not_a_repository(error) => (
                Some("folder"),
                "Not a Git repository".to_owned(),
                "This folder has no Git working tree.".to_owned(),
                colors.tertiary,
                false,
            ),
            LoadState::Error(error) if git_is_not_installed(error) => (
                Some("terminal"),
                "Git unavailable".to_owned(),
                "Git is not installed on this host.".to_owned(),
                colors.tertiary,
                false,
            ),
            LoadState::Error(error) => (
                Some("exclamationmark.triangle.fill"),
                "Git status unavailable".to_owned(),
                error.clone(),
                Ink::ATTENTION,
                false,
            ),
            LoadState::NoSession => (
                Some("minus.circle"),
                "No session selected".to_owned(),
                "Select an agent to inspect its working tree.".to_owned(),
                colors.tertiary,
                false,
            ),
        };
        let status_mark = symbol.map_or_else(
            || LoadingIndicator::new("inspector-git-loading", 16.0, accent).into_any_element(),
            |symbol| sf_symbol(symbol, 15.0, accent),
        );
        div()
            .id("inspector-git-summary")
            .min_h(px(52.0))
            .px(px(11.0))
            .py(px(9.0))
            .flex()
            .items_center()
            .gap(px(10.0))
            .rounded(px(Radius::CARD))
            .bg(colors.primary.alpha(0.035))
            .border_1()
            .border_color(colors.primary.alpha(0.06))
            .when(can_open, |row| {
                row.cursor_pointer()
                    .hover(move |row| row.bg(colors.primary.alpha(0.065)))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.select_tab(InspectorTab::Changes, cx);
                        cx.stop_propagation();
                    }))
            })
            .child(status_mark)
            .child(
                div()
                    .min_w(px(0.0))
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_size(px(Typo::ROW_EMPHASIZED.size))
                            .font_weight(Typo::ROW_EMPHASIZED.weight)
                            .text_color(colors.primary)
                            .child(title),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(px(Typo::META.size))
                            .text_color(colors.tertiary)
                            .child(detail),
                    ),
            )
            .when(can_open, |row| {
                row.child(sf_symbol("chevron.right", 11.0, colors.tertiary))
            })
            .into_any_element()
    }

    fn render_artifacts(
        &mut self,
        session: Option<&SessionRecord>,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(session) = session else {
            return self
                .render_message(
                    colors,
                    "sidebar.left",
                    "Select a session",
                    "Artifacts follow the active agent.",
                )
                .into_any_element();
        };
        if artifact_count(session) == 0 {
            return self
                .render_message(
                    colors,
                    "shippingbox",
                    "No artifacts yet",
                    "Pull requests, previews, Linear issues, and local ports appear here as they’re discovered.",
                )
                .into_any_element();
        }

        let mut content = div()
            .id("inspector-artifacts-scroll")
            .size_full()
            .min_h(px(0.0))
            .px(px(12.0))
            .pt(px(8.0))
            .pb(px(18.0))
            .flex()
            .flex_col()
            .gap(px(10.0))
            .overflow_y_scroll();

        if let Some(pull_requests) = session.pull_requests.as_deref() {
            let inspector = cx.entity();
            for pull_request in pull_requests {
                let body = pull_request
                    .body
                    .as_deref()
                    .filter(|body| !body.trim().is_empty())
                    .map(|body| self.markdown_document(body));
                content = content.child(render_pull_request(
                    pull_request,
                    colors,
                    inspector.clone(),
                    body,
                ));
            }
        }
        if let Some(artifacts) = session.artifacts.as_deref() {
            for artifact in artifacts
                .iter()
                .filter(|artifact| artifact_visible(artifact))
            {
                let represented_by_status = artifact.kind == ArtifactKind::PullRequest
                    && session.pull_requests.as_deref().is_some_and(|statuses| {
                        statuses.iter().any(|status| status.url == artifact.url)
                    });
                if !represented_by_status {
                    content = content.child(render_artifact_row(artifact, colors));
                }
            }
        }
        if let Some(ports) = session.listening_ports.as_deref() {
            for port in ports {
                let url = format!("http://localhost:{}", port.port);
                let activation = url.clone();
                content = content.child(
                    div()
                        .id(SharedString::from(format!("inspector-port-{}", port.port)))
                        .min_h(px(54.0))
                        .px(px(11.0))
                        .py(px(9.0))
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .rounded(px(Radius::ROW))
                        .bg(colors.primary.alpha(0.035))
                        .border_1()
                        .border_color(colors.primary.alpha(0.06))
                        .cursor_pointer()
                        .hover(move |row| row.bg(colors.primary.alpha(0.065)))
                        .child(artifact_icon("network", colors))
                        .child(
                            div()
                                .min_w(px(0.0))
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .text_size(px(Typo::ROW_EMPHASIZED.size))
                                        .font_weight(Typo::ROW_EMPHASIZED.weight)
                                        .text_color(colors.primary)
                                        .child(format!("localhost:{}", port.port)),
                                )
                                .child(
                                    div()
                                        .truncate()
                                        .text_size(px(Typo::META.size))
                                        .text_color(colors.tertiary)
                                        .child(port.process_name.clone()),
                                ),
                        )
                        .child(sf_symbol("arrow.up.right", 11.0, colors.tertiary))
                        .on_click(move |_, _, cx| cx.open_url(&activation)),
                );
            }
        }
        content.into_any_element()
    }

    pub(super) fn render_message(
        &self,
        colors: SemanticColors,
        symbol: &'static str,
        title: &'static str,
        body: impl Into<SharedString>,
    ) -> impl IntoElement {
        div()
            .size_full()
            .px(px(28.0))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(8.0))
            .text_center()
            .child(sf_symbol(symbol, 28.0, colors.tertiary))
            .child(
                div()
                    .text_size(px(Typo::ROW_EMPHASIZED.size))
                    .font_weight(Typo::ROW_EMPHASIZED.weight)
                    .text_color(colors.primary.alpha(0.86))
                    .child(title),
            )
            .child(
                div()
                    .max_w(px(280.0))
                    .text_size(px(Typo::META.size))
                    .text_color(colors.tertiary)
                    .child(body.into()),
            )
    }
}

impl Render for WorkbenchInspector {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = {
            let store = self
                .runtime
                .store
                .read()
                .expect("session store lock poisoned");
            crate::app_theme::sidebar_colors(&store.preferences().terminal_theme)
        };
        let session = self.selected_session();
        let body = match self.selected_tab {
            InspectorTab::Info => self.render_info(session.as_ref(), colors, cx),
            InspectorTab::Artifacts => self.render_artifacts(session.as_ref(), colors, cx),
            InspectorTab::Changes => self.render_changes(colors, window, cx),
            InspectorTab::Code => self.code_viewer.clone().into_any_element(),
        };
        let transition_id = SharedString::from(format!(
            "inspector-tab-transition-{}",
            self.tab_transition_generation
        ));
        let direction = self.tab_direction;
        let ask_composer = self.render_ask_composer(colors, cx);
        div()
            .id("workbench-inspector")
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::handle_key_down))
            .bg(colors.sidebar_surface())
            .text_color(colors.primary)
            .child(self.render_header(session.as_ref(), colors, cx))
            .child(div().min_h(px(0.0)).flex_1().overflow_hidden().child(
                div().relative().size_full().child(body).with_animation(
                    transition_id,
                    Animation::new(Duration::from_millis(190)).with_easing(ease_out_quint()),
                    move |body, delta| {
                        body.left(px(direction * (1.0 - delta) * 8.0))
                            .opacity(0.70 + 0.30 * delta)
                    },
                ),
            ))
            .when_some(ask_composer, |panel, composer| panel.child(composer))
    }
}

pub(super) fn section_label(label: &'static str, colors: SemanticColors) -> AnyElement {
    div()
        .px(px(2.0))
        .text_size(px(Typo::SECTION_HEADER.size))
        .font_weight(Typo::SECTION_HEADER.weight)
        .text_color(colors.tertiary)
        .child(label)
        .into_any_element()
}

fn detail_row(
    label: &'static str,
    value: String,
    monospaced: bool,
    colors: SemanticColors,
) -> AnyElement {
    div()
        .min_h(px(38.0))
        .px(px(11.0))
        .flex()
        .items_center()
        .gap(px(12.0))
        .border_b_1()
        .border_color(colors.primary.alpha(0.05))
        .child(
            div()
                .w(px(64.0))
                .flex_none()
                .text_size(px(Typo::META.size))
                .text_color(colors.tertiary)
                .child(label),
        )
        .child(
            div()
                .min_w(px(0.0))
                .flex_1()
                .truncate()
                .when(monospaced, |value| {
                    value.font_family(crate::fonts::mono_family())
                })
                .text_size(px(if monospaced {
                    Typo::META_MONO.size
                } else {
                    Typo::META.size
                }))
                .text_color(colors.secondary)
                .child(value),
        )
        .into_any_element()
}
