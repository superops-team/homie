//! Overlay row, section, and floating-surface rendering for the navigation
//! overlay (command palette and Quick Open).
//!
//! These methods only read state and build GPUI elements; they mutate nothing
//! except the transient hover/click handlers attached to each row. State and
//! command execution stay in `super`.

use super::*;

impl NavigationOverlay {
    pub(super) fn render_overlay(
        &mut self,
        layout: OverlayLayout,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = {
            let store = self.store.read().expect("session store lock poisoned");
            crate::app_theme::colors(&store.preferences().terminal_theme)
        };
        let content = match self.overlay {
            Some(Overlay::CommandPalette) => self.render_command_palette(layout, colors, cx),
            Some(Overlay::QuickOpen) => self.render_quick_open(layout, colors, cx),
            None => return div().into_any_element(),
        };
        div()
            .absolute()
            .inset_0()
            // A modal owns the entire wheel gesture, including the backdrop.
            // Without this, trackpad deltas hit the terminal underneath and its
            // precise-scroll accumulator releases them after the modal closes.
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .flex()
            .items_start()
            .justify_center()
            .pt(layout.top_inset)
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .occlude()
                    .bg(rgba(0x00000040))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.close_overlay(cx);
                        }),
                    ),
            )
            .child(
                div()
                    .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                        this.close_overlay(cx);
                    }))
                    .child(FloatingSurface::new(colors, content)),
            )
            .into_any_element()
    }

    fn render_search(&self, placeholder: &'static str, colors: SemanticColors) -> AnyElement {
        let field = div()
            .flex()
            .flex_none()
            .items_center()
            .h(px(SEARCH_HEIGHT))
            // Line the query up with the rows' icon column below it.
            .px(px(LIST_PADDING_X + ROW_PADDING_X))
            .text_size(px(14.0));

        if self.query.is_empty() {
            return field
                .text_color(colors.tertiary)
                .child(placeholder)
                .into_any_element();
        }

        field
            .text_color(colors.primary)
            .child(query_label(&self.query))
            .into_any_element()
    }

    fn render_command_palette(
        &mut self,
        layout: OverlayLayout,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let actions = self.ranked_actions.clone();
        let sessions = self.ranked_sessions.clone();
        let action_count = actions.len();
        let session_count = sessions.len();
        let mut results = div()
            .id("command-palette-results")
            .track_scroll(&self.scroll_handle)
            .flex()
            .flex_col()
            .max_h(layout.list_height)
            .overflow_y_scroll()
            .px(px(LIST_PADDING_X))
            .py(px(LIST_PADDING_Y));

        if !actions.is_empty() {
            results = results.child(section_header("Actions", colors));
            for (index, action) in actions.into_iter().enumerate() {
                results = results.child(self.render_action_row(action, index, colors, cx));
            }
        }
        if !sessions.is_empty() {
            results = results.child(section_header("Sessions", colors));
            for (offset, session) in sessions.into_iter().enumerate() {
                results = results.child(self.render_session_row(
                    session,
                    action_count + offset,
                    colors,
                    cx,
                ));
            }
        }
        if action_count == 0 && session_count == 0 {
            results = results.child(empty_label("No matches", colors));
        }

        div()
            .w(layout.width)
            .text_color(colors.primary)
            .child(self.render_search("Type a command or session…", colors))
            .child(HairlineDivider::horizontal(colors))
            .child(results)
            .into_any_element()
    }

    fn render_action_row(
        &mut self,
        ranked: Ranked<PaletteAction>,
        index: usize,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let action = ranked.item;
        let command = action.command.clone();
        palette_row(
            highlighted_label(action.title, &ranked.title_matches),
            sf_symbol(action.system_image, 12.5, colors.secondary),
            action.shortcut.map(SharedString::from),
            index == self.highlight,
            index,
            colors,
        )
        .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
            if *hovered {
                this.highlight = index;
                cx.notify();
            }
        }))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.run_command_selection(CommandSelection::Action(command.clone()), cx);
        }))
        .into_any_element()
    }

    fn render_session_row(
        &mut self,
        ranked: Ranked<SessionRecord>,
        index: usize,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let session = ranked.item;
        let id = session.id.clone();
        let dot_color = attention_color(session.attention(), colors);
        let chip = kind_label(session.effective_kind());
        palette_row(
            highlighted_label(session.title, &ranked.title_matches),
            div()
                .flex_none()
                .size(px(7.0))
                .rounded_full()
                .bg(dot_color)
                .into_any_element(),
            Some(SharedString::from(chip)),
            index == self.highlight,
            index,
            colors,
        )
        .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
            if *hovered {
                this.highlight = index;
                cx.notify();
            }
        }))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.run_command_selection(CommandSelection::Session(id.clone()), cx);
        }))
        .into_any_element()
    }

    fn render_quick_open(
        &mut self,
        layout: OverlayLayout,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let searching = !self.query.text().trim().is_empty();
        let mut results = div()
            .id("quick-open-results")
            .track_scroll(&self.scroll_handle)
            .flex()
            .flex_col()
            .max_h(layout.list_height)
            .overflow_y_scroll()
            .px(px(LIST_PADDING_X))
            .py(px(LIST_PADDING_Y));

        if searching {
            let ranked = self.ranked_items.clone();
            for (index, folder) in ranked.iter().enumerate() {
                results = results.child(self.render_quick_row(
                    folder.item.clone(),
                    &folder.name_matches,
                    index,
                    colors,
                    cx,
                ));
            }
            if ranked.is_empty() {
                results = results.child(empty_label(
                    if self.directory_index.is_scanning() {
                        "Scanning…"
                    } else {
                        "No matches"
                    },
                    colors,
                ));
            }
        } else {
            let recent = self.quick_snapshot.recent.clone();
            let folders = self.quick_snapshot.folders.clone();
            if !recent.is_empty() {
                results = results.child(section_header("Recent", colors));
                for (index, item) in recent.iter().cloned().enumerate() {
                    results = results.child(self.render_quick_row(item, &[], index, colors, cx));
                }
            }
            if !folders.is_empty() {
                results = results.child(section_header("Folders", colors));
                for (offset, item) in folders.iter().cloned().enumerate() {
                    results = results.child(self.render_quick_row(
                        item,
                        &[],
                        recent.len() + offset,
                        colors,
                        cx,
                    ));
                }
            }
            if recent.is_empty() && folders.is_empty() {
                results = results.child(empty_label(
                    if self.directory_index.is_scanning() {
                        "Scanning…"
                    } else {
                        "No folders indexed"
                    },
                    colors,
                ));
            }
        }

        div()
            .w(layout.width)
            .text_color(colors.primary)
            .child(self.render_search("Jump to a project or folder…", colors))
            .child(HairlineDivider::horizontal(colors))
            .child(results)
            .into_any_element()
    }

    fn render_quick_row(
        &mut self,
        item: QuickOpenItem,
        name_matches: &[Range<usize>],
        index: usize,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let path = item.path.clone();
        let parent = relative_parent(&item.path);
        let icon_color = if item.is_git_repo {
            Palette::CLAY
        } else {
            colors.secondary
        };
        let default_name = self
            .store
            .read()
            .expect("session store lock poisoned")
            .preferences()
            .default_agent
            .display_name()
            .to_owned();
        let row = div()
            .id(format!("quick-row-{index}"))
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .h(px(if parent.is_empty() {
                QUICK_ROW_HEIGHT
            } else {
                QUICK_ROW_HEIGHT_WITH_PATH
            }))
            .px(px(ROW_PADDING_X))
            .rounded(px(Radius::ROW))
            .bg(if index == self.highlight {
                colors.primary.alpha(0.10)
            } else {
                colors.primary.alpha(0.0)
            })
            .cursor_pointer()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(9.0))
                    .min_w(px(0.0))
                    .child(
                        div()
                            .w(px(18.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(sf_symbol_weighted(
                                if item.is_git_repo {
                                    "folder.fill"
                                } else {
                                    "folder"
                                },
                                13.0,
                                SymbolWeight::Regular,
                                icon_color,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .min_w(px(0.0))
                            .text_size(px(13.0))
                            .child(highlighted_label(item.name.clone(), name_matches))
                            .when(!parent.is_empty(), |column| {
                                column.child(
                                    div()
                                        .text_size(px(11.0))
                                        .text_color(colors.tertiary)
                                        .child(parent.clone()),
                                )
                            }),
                    ),
            )
            .when(index == self.highlight, |row| {
                row.child(
                    div()
                        .flex()
                        .gap(px(5.0))
                        .child(chip(format!("⏎ {}", default_name.to_lowercase()), colors))
                        .child(chip("⌘⏎ term", colors)),
                )
            })
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                if *hovered {
                    this.highlight = index;
                    cx.notify();
                }
            }))
            .on_click(cx.listener(move |this, _, _, cx| {
                let cwd = path.to_string_lossy().into_owned();
                this.store
                    .write()
                    .expect("session store lock poisoned")
                    .spawn_default(SpawnOptions {
                        cwd: Some(cwd.clone()),
                        ..SpawnOptions::default()
                    });
                this.close_overlay(cx);
            }));
        row.into_any_element()
    }
}

fn section_header(title: &'static str, colors: SemanticColors) -> AnyElement {
    div()
        .h(px(SECTION_HEADER_HEIGHT))
        .flex()
        .flex_none()
        .items_end()
        .px(px(ROW_PADDING_X))
        .pb(px(3.0))
        .text_size(px(11.0))
        .text_color(colors.tertiary)
        .child(title)
        .into_any_element()
}

fn empty_label(text: &'static str, colors: SemanticColors) -> AnyElement {
    div()
        .h(px(ROW_HEIGHT))
        .flex()
        .flex_none()
        .items_center()
        .px(px(ROW_PADDING_X))
        .text_size(px(13.0))
        .text_color(colors.tertiary)
        .child(text)
        .into_any_element()
}

fn palette_row(
    title: AnyElement,
    leading: AnyElement,
    // Owned: agent chips are manifest ids now, not compile-time literals.
    trailing: Option<SharedString>,
    highlighted: bool,
    index: usize,
    colors: SemanticColors,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(format!("palette-row-{index}"))
        .flex()
        // Without this the rows are shrinkable flex children: a list taller
        // than its container squeezes every row toward min-content instead of
        // scrolling, and 40pt rows render as ~21pt of crammed text.
        .flex_none()
        .items_center()
        .justify_between()
        .h(px(ROW_HEIGHT))
        .px(px(ROW_PADDING_X))
        .rounded(px(Radius::ROW))
        .bg(if highlighted {
            colors.primary.alpha(0.10)
        } else {
            colors.primary.alpha(0.0)
        })
        .cursor_pointer()
        .text_size(px(13.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(9.0))
                .min_w_0()
                .child(
                    div()
                        .w(px(18.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(leading),
                )
                .child(
                    div()
                        .min_w_0()
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(title),
                ),
        )
        .when_some(trailing, |row, trailing| row.child(chip(trailing, colors)))
}

fn chip(text: impl Into<gpui::SharedString>, colors: SemanticColors) -> AnyElement {
    div()
        .px(px(5.0))
        .py(px(2.0))
        .rounded(px(Radius::CHIP))
        .bg(colors.primary.alpha(0.06))
        .text_size(px(11.0))
        .text_color(colors.tertiary)
        .child(text.into())
        .into_any_element()
}

fn attention_color(attention: AttentionLevel, colors: SemanticColors) -> gpui::Rgba {
    match attention {
        AttentionLevel::NeedsInput => gpui::rgb(0xf59e0b),
        AttentionLevel::DoneUnseen => gpui::rgb(0x3b82f6),
        AttentionLevel::Working => colors.secondary,
        _ => colors.tertiary,
    }
}

/// Compact label for the navigator's kind column. The manifest id is already a
/// short lowercase word for every agent, so only the two non-agent kinds and
/// Claude's hyphenated id need shortening.
fn kind_label(kind: &AgentKind) -> String {
    match kind.id() {
        AgentKind::CLAUDE_CODE_ID => "claude".to_owned(),
        AgentKind::GENERIC_ID => "term".to_owned(),
        other => other.to_owned(),
    }
}

pub(super) fn relative_parent(path: &Path) -> String {
    let Some(parent) = path.parent() else {
        return String::new();
    };
    let parent = parent.to_string_lossy().into_owned();
    if parent.is_empty() || parent == "/" {
        return parent;
    }
    let Some(home) = std::env::var_os("HOME") else {
        return parent;
    };
    let home = PathBuf::from(home);
    if parent == home.to_string_lossy() {
        return "~".into();
    }
    parent
        .strip_prefix(&format!("{}/", home.to_string_lossy()))
        .map_or(parent.clone(), |suffix| format!("~/{suffix}"))
}
