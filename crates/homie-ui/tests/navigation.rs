use homie_ui::{HistoryEntry, fuzzy_score, rank_items};

#[test]
fn fuzzy_score_rewards_prefix_and_word_boundaries() {
    assert!(fuzzy_score("co", "codex").unwrap() > fuzzy_score("co", "my codex").unwrap());
    assert!(fuzzy_score("cc", "Claude Code").unwrap() > fuzzy_score("cc", "acclaim").unwrap());
    assert!(fuzzy_score("zz", "codex").is_none());
}

#[test]
fn rank_items_orders_by_score_then_label() {
    let ranked = rank_items("co", ["codex", "my codex", "cursor"]);
    assert_eq!(ranked[0].label, "codex");
    assert_eq!(ranked[1].label, "my codex");
}

#[test]
fn history_entry_resume_requires_existing_cwd_and_transcript() {
    let resumable = HistoryEntry {
        id: "thread_1".to_string(),
        agent_kind: "codex".to_string(),
        cwd: "/repo".to_string(),
        title: Some("Work".to_string()),
        transcript_path: "/tmp/transcript.jsonl".to_string(),
        cwd_exists: true,
    };
    assert!(resumable.can_resume());

    let dead_cwd = HistoryEntry {
        cwd_exists: false,
        ..resumable.clone()
    };
    assert!(!dead_cwd.can_resume());

    let missing_transcript = HistoryEntry {
        transcript_path: String::new(),
        ..resumable
    };
    assert!(!missing_transcript.can_resume());
}
