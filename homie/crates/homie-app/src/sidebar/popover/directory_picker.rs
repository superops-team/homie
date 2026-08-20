use super::*;

impl Sidebar {
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
}
