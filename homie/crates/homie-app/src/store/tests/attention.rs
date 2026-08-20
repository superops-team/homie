use super::*;

#[test]
fn attention_rollup_and_needs_input_sort_use_proto_derivation() {
    let mut done = session("done", "p", 1.0);
    done.last_turn_completed_at = Some(DateMillis(50.0));
    done.last_seen_at = Some(DateMillis(40.0));
    let mut older_input = session("older-input", "p", 2.0);
    older_input.status = SessionStatus::NeedsInput(homie_proto::NeedsInputKind::Question);
    older_input.updated_at = DateMillis(100.0);
    let mut newer_input = session("newer-input", "p", 3.0);
    newer_input.status = SessionStatus::NeedsInput(homie_proto::NeedsInputKind::Permission);
    newer_input.updated_at = DateMillis(200.0);
    let (store, _) = hydrated(
        vec![done, older_input, newer_input],
        vec![project("p", "P")],
        Prefs::default(),
    );

    assert_eq!(store.global_attention(), AttentionLevel::NeedsInput);
    assert_eq!(
        store
            .needs_input_sessions()
            .iter()
            .map(|session| session.id.clone())
            .collect::<Vec<_>>(),
        vec![id("newer-input"), id("older-input")]
    );
}

#[test]
fn hidden_needs_input_update_emits_chime_and_notification_effect() {
    let (mut store, mut effects) = hydrated(
        vec![session("visible", "p", 2.0)],
        vec![project("p", "P")],
        Prefs::default(),
    );
    drain(&mut effects);

    let mut hidden = session("hidden", "p", 1.0);
    hidden.status = SessionStatus::NeedsInput(homie_proto::NeedsInputKind::Permission);
    store.upsert_session(hidden);

    let transition = drain(&mut effects)
        .into_iter()
        .find_map(|effect| match effect {
            StoreEffect::StatusTransition(transition) => Some(transition),
            _ => None,
        })
        .expect("needs-input update should emit a status transition");
    assert_eq!(transition.sound, Some(NotificationSound::NeedsInput));
    assert!(transition.notification.is_some());
}
