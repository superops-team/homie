use super::*;

impl Sidebar {
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
}
