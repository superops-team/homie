use super::*;

impl WorkbenchInspector {
    pub(super) fn render_ask_composer(
        &self,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let draft = self.ask_draft.as_ref()?;
        let empty = self.ask_query.text().trim().is_empty();
        let busy = self.ask_busy;
        let label = draft.label.clone();

        let mut composer = div()
            .id("inspector-ask-composer")
            .debug_selector(|| "INSPECTOR_ASK_COMPOSER".to_owned())
            .flex_none()
            .px(px(11.0))
            .py(px(9.0))
            .flex()
            .flex_col()
            .gap(px(7.0))
            .border_t_1()
            .border_color(colors.primary.alpha(0.09))
            .bg(rgba(0x17191ef8))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(7.0))
                    .child(sf_symbol_weighted(
                        "sparkles",
                        11.5,
                        SymbolWeight::Semibold,
                        rgba(0xe9a381ff),
                    ))
                    .child(
                        div()
                            .text_size(px(10.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors.primary)
                            .child("Ask active agent"),
                    )
                    .child(
                        div()
                            .min_w(px(0.0))
                            .flex_1()
                            .truncate()
                            .text_size(px(9.5))
                            .text_color(colors.tertiary)
                            .child(label),
                    )
                    .child(
                        div()
                            .id("inspector-ask-close")
                            .size(px(20.0))
                            .flex_none()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(Radius::CHIP))
                            .cursor_pointer()
                            .hover(move |button| button.bg(colors.primary.alpha(0.07)))
                            .child(sf_symbol("xmark", 9.5, colors.tertiary))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.ask_draft = None;
                                this.ask_feedback = None;
                                this.ask_query.clear();
                                cx.notify();
                                cx.stop_propagation();
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(5.0))
                    .child(self.render_ask_preset(
                        "ask-preset-review",
                        "Review",
                        "Review this for correctness, regressions, and missing tests.",
                        colors,
                        cx,
                    ))
                    .child(self.render_ask_preset(
                        "ask-preset-risks",
                        "Find risks",
                        "Find the highest-risk behavior changes and explain why they matter.",
                        colors,
                        cx,
                    ))
                    .child(self.render_ask_preset(
                        "ask-preset-tests",
                        "Suggest tests",
                        "Identify missing tests and propose concrete cases for this context.",
                        colors,
                        cx,
                    )),
            )
            .child(
                div()
                    .h(px(34.0))
                    .flex()
                    .items_center()
                    .gap(px(7.0))
                    .child(
                        div()
                            .id("inspector-ask-input")
                            .min_w(px(0.0))
                            .h_full()
                            .flex_1()
                            .px(px(9.0))
                            .flex()
                            .items_center()
                            .rounded(px(Radius::BADGE))
                            .bg(colors.primary.alpha(0.045))
                            .border_1()
                            .border_color(colors.primary.alpha(0.075))
                            .text_size(px(10.5))
                            .text_color(colors.primary)
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, window, cx| {
                                    window.focus(&this.focus, cx);
                                    cx.stop_propagation();
                                }),
                            )
                            .child(if empty {
                                div()
                                    .text_color(colors.tertiary)
                                    .child("Ask a follow-up…")
                                    .into_any_element()
                            } else {
                                crate::navigation::query_label(&self.ask_query)
                            }),
                    )
                    .child(
                        div()
                            .id("inspector-ask-send")
                            .debug_selector(|| "INSPECTOR_ASK_SEND".to_owned())
                            .h_full()
                            .px(px(11.0))
                            .flex_none()
                            .flex()
                            .items_center()
                            .gap(px(5.0))
                            .rounded(px(Radius::BADGE))
                            .bg(if empty || busy {
                                colors.primary.alpha(0.04)
                            } else {
                                rgba(0xd97757d9)
                            })
                            .text_size(px(10.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(if empty || busy {
                                colors.primary.alpha(0.28)
                            } else {
                                rgba(0xffffffff)
                            })
                            .when(!empty && !busy, |button| {
                                button
                                    .cursor_pointer()
                                    .hover(|button| button.bg(rgba(0xe38563ff)))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.submit_ask(cx);
                                        cx.stop_propagation();
                                    }))
                            })
                            .child(if busy { "Sending…" } else { "Send" })
                            .child(sf_symbol(
                                "arrow.up",
                                9.0,
                                if empty || busy {
                                    colors.primary.alpha(0.28)
                                } else {
                                    rgba(0xffffffff)
                                },
                            )),
                    ),
            );
        if let Some((success, message)) = &self.ask_feedback {
            let accent = if *success { Ink::FRESH } else { Ink::DANGER };
            composer = composer.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(5.0))
                    .text_size(px(9.5))
                    .text_color(accent)
                    .child(sf_symbol(
                        if *success {
                            "checkmark.circle.fill"
                        } else {
                            "exclamationmark.circle.fill"
                        },
                        10.0,
                        accent,
                    ))
                    .child(message.clone()),
            );
        }
        Some(composer.into_any_element())
    }

    pub(super) fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.ask_draft.is_some() {
            match event.keystroke.key.as_str() {
                "escape" => {
                    self.ask_draft = None;
                    self.ask_feedback = None;
                    self.ask_query.clear();
                    cx.notify();
                }
                "enter" => self.submit_ask(cx),
                _ => {
                    let Some(edit) = query_editor::edit_for(&event.keystroke) else {
                        return;
                    };
                    match edit {
                        Edit::Local(local) => {
                            self.ask_query.apply(local);
                        }
                        Edit::Clipboard(ClipboardEdit::Copy) => {
                            query_editor::copy_selection(&self.ask_query, cx);
                        }
                        Edit::Clipboard(ClipboardEdit::Cut) => {
                            query_editor::cut_selection(&mut self.ask_query, cx);
                        }
                        Edit::Clipboard(ClipboardEdit::Paste) => {
                            if let Some(text) =
                                cx.read_from_clipboard().and_then(|item| item.text())
                            {
                                self.ask_query.insert(&text);
                            }
                        }
                    }
                    cx.notify();
                }
            }
            cx.stop_propagation();
            return;
        }
        if !self.commit_open {
            return;
        }
        match event.keystroke.key.as_str() {
            "escape" => {
                self.commit_open = false;
                cx.notify();
            }
            "enter" => self.submit_commit(cx),
            _ => {
                let Some(edit) = query_editor::edit_for(&event.keystroke) else {
                    return;
                };
                match edit {
                    Edit::Local(local) => {
                        self.commit_query.apply(local);
                    }
                    Edit::Clipboard(ClipboardEdit::Copy) => {
                        query_editor::copy_selection(&self.commit_query, cx);
                    }
                    Edit::Clipboard(ClipboardEdit::Cut) => {
                        query_editor::cut_selection(&mut self.commit_query, cx);
                    }
                    Edit::Clipboard(ClipboardEdit::Paste) => {
                        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                            self.commit_query.insert(&text);
                        }
                    }
                }
                cx.notify();
            }
        }
        cx.stop_propagation();
    }

    fn render_ask_preset(
        &self,
        id: &'static str,
        label: &'static str,
        question: &'static str,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .id(id)
            .h(px(21.0))
            .px(px(7.0))
            .flex()
            .items_center()
            .rounded_full()
            .bg(colors.primary.alpha(0.045))
            .border_1()
            .border_color(colors.primary.alpha(0.065))
            .cursor_pointer()
            .hover(move |button| button.bg(colors.primary.alpha(0.085)))
            .text_size(px(9.5))
            .font_weight(FontWeight::MEDIUM)
            .text_color(colors.secondary)
            .child(label)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.set_ask_question(question, cx);
                cx.stop_propagation();
            }))
            .into_any_element()
    }
}
