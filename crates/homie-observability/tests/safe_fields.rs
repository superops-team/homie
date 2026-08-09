use homie_observability::{SafeFieldError, SafeFields};
use serde_json::json;

#[test]
fn safe_field_projection_keeps_allowlisted_fields_and_drops_unknowns() {
    let input = json!({
        "event.name": "session.status",
        "event.seq": 7,
        "session.id": "s_123",
        "usage.input_tokens": 42,
        "debug.unowned": "drop me"
    });

    let projected = SafeFields::project(&input).expect("safe projection");

    assert_eq!(projected.get("event.name"), Some(&json!("session.status")));
    assert_eq!(projected.get("event.seq"), Some(&json!(7)));
    assert_eq!(projected.get("session.id"), Some(&json!("s_123")));
    assert_eq!(projected.get("usage.input_tokens"), Some(&json!(42)));
    assert!(projected.get("debug.unowned").is_none());
}

#[test]
fn safe_field_projection_blocks_dangerous_fields_even_when_other_fields_are_safe() {
    let input = json!({
        "event.name": "session.output",
        "authorization": "example-authorization-value",
        "raw_prompt": "example prompt body",
        "tool_args": {"path": "/private/file"}
    });

    let error = SafeFields::project(&input).expect_err("dangerous fields fail closed");

    assert_eq!(
        error,
        SafeFieldError::DangerousField {
            field: "authorization".to_string()
        }
    );
}

#[test]
fn safe_field_projection_blocks_dangerous_fields_inside_allowed_objects() {
    let input = json!({
        "evidence.output_summary": {
            "safe": "short summary",
            "headers": {
                "authorization": "example-authorization-value"
            }
        }
    });

    let error = SafeFields::project(&input).expect_err("nested dangerous fields fail closed");

    assert_eq!(
        error,
        SafeFieldError::DangerousField {
            field: "evidence.output_summary.headers".to_string()
        }
    );
}
