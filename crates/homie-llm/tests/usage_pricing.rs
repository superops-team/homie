use homie_llm::{claude_estimate, match_claude, match_openai, openai_estimate};

#[test]
fn claude_estimate_matches_diri_cache_rates() {
    let base = claude_estimate("claude-sonnet", 1_000_000, 0, 0, 0, 0).expect("sonnet price");
    assert_eq!(base, 3.0);

    let cache_read = claude_estimate("claude-sonnet", 0, 0, 1_000_000, 0, 0).expect("sonnet price");
    assert!((cache_read - 0.3).abs() < 1e-12);

    let write_5m = claude_estimate("claude-sonnet", 0, 0, 0, 1_000_000, 0).expect("sonnet price");
    assert!((write_5m - 3.75).abs() < 1e-12);

    let write_1h = claude_estimate("claude-sonnet", 0, 0, 0, 0, 1_000_000).expect("sonnet price");
    assert!((write_1h - 6.0).abs() < 1e-12);

    let opus_4 = match_claude("claude-opus-4-1").expect("opus 4.1 price");
    assert_eq!(opus_4.input, 15.0);
    assert_eq!(opus_4.output, 75.0);
}

#[test]
fn openai_estimate_matches_diri_model_rules() {
    let cached = openai_estimate("codex", 0, 0, 1_000_000).expect("codex price");
    assert!((cached - 0.175).abs() < 1e-12);

    let gpt_54_mini = match_openai("gpt-5.4-mini").expect("gpt-5.4-mini price");
    assert_eq!(gpt_54_mini.input, 0.75);
    assert_eq!(gpt_54_mini.output, 4.5);

    let output = openai_estimate("gpt-5.5", 0, 1_000_000, 0).expect("gpt-5.5 price");
    assert_eq!(output, 30.0);
}

#[test]
fn unknown_and_negative_inputs_are_safe() {
    assert!(match_claude("unknown-model").is_none());
    assert!(match_openai("unknown-model").is_none());
    assert!(claude_estimate("unknown-model", 1, 1, 1, 1, 1).is_none());
    assert!(openai_estimate("unknown-model", 1, 1, 1).is_none());

    let clamped = openai_estimate("codex", -1_000_000, 1_000_000, -1).expect("codex price");
    assert_eq!(clamped, 14.0);
}
