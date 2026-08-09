//! Masks obvious secrets and strips control sequences from captured text.
//!
//! Ported from diri-engine. Prompt excerpts are shown in the UI and sent to
//! the phone, so anything captured off a screen passes through here first.

use std::sync::LazyLock;

use regex::Regex;

static SECRET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)\b(token|secret|password|api[_-]?key|authorization|cookie)\b\s*[=:]\s*(bearer\s+)?[^\s&"'}\]]+"#,
    )
        .expect("secret pattern compiles")
});

static URL_SECRET_QUERY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)([?&](?:token|secret|password|api[_-]?key|authorization|cookie)=)[^&\s"'}\]]+"#,
    )
    .expect("url secret query pattern compiles")
});

static ANSI: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\x1b\[[0-9;?]*[a-zA-Z]").expect("ansi pattern compiles"));

pub fn redact(text: &str) -> String {
    let masked = URL_SECRET_QUERY.replace_all(text, "$1•••");
    let masked = SECRET.replace_all(&masked, "$1=•••");
    ANSI.replace_all(&masked, "").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_secret_assignments_case_insensitively() {
        assert_eq!(redact("API_KEY: sk-abc123"), "API_KEY=•••");
        assert_eq!(redact("password=hunter2"), "password=•••");
        assert_eq!(redact("Authorization: Bearer abc123"), "Authorization=•••");
        assert_eq!(redact("Cookie: sid=abc123"), "Cookie=•••");
    }

    #[test]
    fn masks_url_query_secret_values() {
        assert_eq!(
            redact("https://x.test?a=1&token=sk-abc&safe=ok&api_key=sk-key"),
            "https://x.test?a=1&token=•••&safe=ok&api_key=•••"
        );
    }

    #[test]
    fn strips_ansi_control_sequences() {
        assert_eq!(redact("\x1b[31mred\x1b[0m"), "red");
    }

    #[test]
    fn leaves_ordinary_text_alone() {
        assert_eq!(redact("do you want to proceed?"), "do you want to proceed?");
    }
}
