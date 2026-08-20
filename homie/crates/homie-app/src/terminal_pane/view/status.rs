use super::buttons::*;
use super::*;

impl TerminalPane {
    pub(crate) fn render_exit_pill(
        &self,
        session: &SessionRecord,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = session.id.clone();
        let resumable = session.resumability == Resumability::Resumable;
        let mut pill = div()
            .id("exit-pill")
            .rounded(px(999.0))
            .pl(px(12.0))
            .pr(if resumable { px(4.0) } else { px(12.0) })
            .py(px(4.0))
            .bg(rgba(0x303238e8))
            .flex()
            .items_center()
            .gap(px(8.0))
            .text_size(px(11.5))
            .text_color(rgba(0xffffff99))
            .child(sf_symbol("power", 11.0, rgba(0xffffff66)))
            .child(exit_description(session));
        if resumable {
            pill = pill.child(
                div()
                    .id("exit-pill-resume")
                    .rounded(px(999.0))
                    .px(px(9.0))
                    .py(px(3.0))
                    .bg(rgba(0xffffff1a))
                    .hover(|style| style.bg(rgba(0xffffff2e)))
                    .cursor_pointer()
                    .text_color(rgba(0xffffffe6))
                    .child("Resume")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.runtime
                            .store
                            .read()
                            .expect("session store lock poisoned")
                            .resume(id.clone());
                        cx.notify();
                    })),
            );
        } else if session.resumability == Resumability::TranscriptMissing {
            pill = pill.child(
                div()
                    .text_color(rgba(0xffffff4d))
                    .child("· transcript gone"),
            );
        }
        // Centered by a full-width row rather than a guessed half-width offset,
        // since the description's length varies with the exit reason.
        div()
            .absolute()
            .bottom(px(18.0))
            .left_0()
            .right_0()
            .flex()
            .justify_center()
            .child(pill)
            .into_any_element()
    }

    pub(crate) fn render_exited_takeover(
        &self,
        session: &SessionRecord,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let (auto_resuming, migrating) = {
            let store = self
                .runtime
                .store
                .read()
                .expect("session store lock poisoned");
            (
                store.auto_resuming().contains(&session.id),
                store.migrating().contains(&session.id),
            )
        };
        // Mid-migration the source agent is briefly down; show the busy state
        // instead of an exit card with a doomed Resume button.
        if migrating {
            return Some(centered_message("◌", "Moving session…").into_any_element());
        }
        if auto_resuming {
            return Some(centered_message("◌", "Resuming conversation…").into_any_element());
        }
        if self
            .residents
            .get(&session.id)
            .is_some_and(|resident| resident.element.has_content())
        {
            return None;
        }
        Some(self.render_exited_card(session, cx))
    }

    pub(crate) fn render_exited_card(
        &self,
        session: &SessionRecord,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = session.id.clone();
        let content = centered_message("", &exit_description(session));
        if session.resumability == Resumability::Resumable {
            content
                .child(primary_button(
                    "resume-conversation",
                    "Resume Conversation",
                    cx,
                    move |this, cx| {
                        this.runtime
                            .store
                            .read()
                            .expect("session store lock poisoned")
                            .resume(id.clone());
                        cx.notify();
                    },
                ))
                .into_any_element()
        } else if session.resumability == Resumability::TranscriptMissing {
            content
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(rgba(0xffffff4d))
                        .child("Transcript is gone — start a fresh session in the same folder."),
                )
                .into_any_element()
        } else {
            content.into_any_element()
        }
    }

    pub(crate) fn render_archived_overlay(
        &self,
        session: &SessionRecord,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = session.id.clone();
        let mut content = centered_symbol_message("archivebox", 30.0, &session.title).child(
            div()
                .text_size(px(13.0))
                .text_color(rgba(0xffffff99))
                .child("Archived"),
        );
        if session.resumability == Resumability::NotResumable {
            content = content.child(
                div()
                    .max_w(px(320.0))
                    .text_size(px(11.5))
                    .text_color(rgba(0xffffff4d))
                    .child(
                        "This session can't resume its conversation; revive restores it as ended.",
                    ),
            );
        }
        content
            .child(primary_button(
                "revive-session",
                "Revive Session",
                cx,
                move |this, cx| {
                    this.runtime
                        .store
                        .write()
                        .expect("session store lock poisoned")
                        .revive_sessions(vec![id.clone()]);
                    this.reconcile_residency();
                    cx.notify();
                },
            ))
            .into_any_element()
    }
}
