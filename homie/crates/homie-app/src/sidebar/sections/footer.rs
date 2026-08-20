use super::*;

impl Sidebar {
    pub(crate) fn update_pill(
        &self,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if self.preview || !self.update.is_noteworthy() {
            return None;
        }
        let (symbol, tint, command) = match &self.update.phase {
            UpdatePhase::Available(_) => (
                "arrow.down.circle",
                homie_ui::Ink::FRESH,
                Some(UpdateCommand::Download),
            ),
            UpdatePhase::Downloading { .. } => ("arrow.down.circle", colors.secondary, None),
            UpdatePhase::Ready(_) => (
                "arrow.clockwise.circle",
                homie_ui::Ink::FRESH,
                Some(UpdateCommand::Install),
            ),
            UpdatePhase::Installing => ("arrow.clockwise.circle", colors.secondary, None),
            UpdatePhase::Failed(_) => (
                "exclamationmark.triangle",
                homie_ui::Ink::DANGER,
                Some(UpdateCommand::Dismiss),
            ),
            UpdatePhase::Checking => ("arrow.triangle.2.circlepath", colors.secondary, None),
            UpdatePhase::UpToDate => (
                "checkmark.circle",
                colors.secondary,
                Some(UpdateCommand::Dismiss),
            ),
            UpdatePhase::Idle | UpdatePhase::Unsupported(_) => return None,
        };
        let interactive = command.is_some();
        let hovered = interactive && self.ui.hovered_control == Some("update");
        let mut pill = div()
            .id("update-pill")
            .mb(px(3.0))
            .px(px(Space::ROW_H))
            .h(px(Metrics::ROW_HEIGHT))
            .flex()
            .items_center()
            .gap(px(8.0))
            .rounded(px(Radius::ROW))
            .bg(Fill::hover(colors, hovered))
            .child(
                div()
                    .w(px(16.0))
                    .text_center()
                    .child(sf_symbol(symbol, 12.5, tint)),
            )
            .child(
                div()
                    .min_w(px(0.0))
                    .flex_1()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_size(px(Typo::ROW.size))
                    .text_color(if interactive { tint } else { colors.secondary })
                    .child(self.update.summary()),
            )
            .on_hover(cx.listener(move |this, is_hovered: &bool, _, cx| {
                this.ui.hovered_control = (interactive && *is_hovered).then_some("update");
                cx.notify();
            }));
        if let Some(command) = command {
            pill = pill.cursor_pointer().on_click(cx.listener(
                move |_, _, _, cx: &mut Context<Self>| {
                    cx.emit(SidebarEvent::Update(command.clone()));
                },
            ));
        }
        Some(pill.into_any_element())
    }

    pub(crate) fn account_footer(
        &self,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let hovered = self.ui.hovered_control == Some("account");
        let cost = if self.preview {
            Some(PREVIEW_USAGE)
        } else {
            self.usage
                .map(|snapshot| snapshot.today().cost)
                .filter(|cost| *cost > 0.0)
        };
        div()
            .flex_none()
            .px(px(Space::INSET))
            .pt(px(5.0))
            .pb(px(10.0))
            .border_t_1()
            .border_color(colors.primary.alpha(0.06))
            .children(self.update_pill(colors, cx))
            .child(
                div()
                    .id("account")
                    .px(px(Space::ROW_H))
                    .h(px(Metrics::ROW_HEIGHT))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .rounded(px(Radius::ROW))
                    .bg(Fill::hover(colors, hovered))
                    .cursor_pointer()
                    .on_hover(cx.listener(|this, is_hovered: &bool, _, cx| {
                        this.ui.hovered_control = is_hovered.then_some("account");
                        cx.notify();
                    }))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.ui.popover = Some(Popover::Account);
                        cx.notify();
                    }))
                    .child(
                        div()
                            .w(px(16.0))
                            .text_center()
                            .text_size(px(13.0))
                            .text_color(colors.secondary)
                            .child(sf_symbol("person.crop.circle", 12.5, colors.secondary)),
                    )
                    .child(
                        div()
                            .min_w(px(0.0))
                            .flex_1()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .text_size(px(Typo::ROW.size))
                            .text_color(colors.text(homie_ui::TextTone::Label))
                            .child(if self.preview {
                                "preview@homie.local"
                            } else {
                                "Local agents"
                            }),
                    )
                    .when_some(cost, |row, cost| {
                        row.child(
                            div()
                                .font_family(crate::fonts::mono_family())
                                .text_size(px(Typo::META_MONO.size))
                                .text_color(colors.tertiary)
                                .child(UsageFormat::money(cost)),
                        )
                    })
                    .child(div().text_size(px(9.0)).text_color(colors.tertiary).child(
                        sf_symbol_weighted(
                            "chevron.up.chevron.down",
                            8.5,
                            SymbolWeight::Semibold,
                            colors.tertiary,
                        ),
                    )),
            )
            .into_any_element()
    }
}
