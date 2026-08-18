//! Newline-delimited JSON framing.
//!
//! ACP frames are one JSON value per line, terminated by `\n`. Each message is
//! a standalone JSON-RPC request, response or notification. This module owns
//! the encode/decode boundary so callers never hand-roll `\n` appends or
//! partial-line parsing.

use std::io::{self, BufRead};

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Serialize a value into a single newline-terminated frame.
pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Parse a single frame (without the trailing newline) into `T`.
pub fn decode<T: DeserializeOwned>(line: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(line)
}

/// Read the next frame line from a buffered reader.
///
/// Returns `Ok(None)` at end-of-stream. Empty lines (a stray blank between
/// frames) are skipped rather than surfaced as parse errors; any other line is
/// returned verbatim so the caller can classify it.
pub fn read_line(reader: &mut impl BufRead) -> io::Result<Option<String>> {
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end_matches(['\n', '\r']);
        if trimmed.trim().is_empty() {
            continue;
        }
        return Ok(Some(trimmed.to_owned()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn encode_appends_exactly_one_newline() {
        let bytes = encode(&serde_json::json!({"a": 1})).expect("encode");
        let text = String::from_utf8(bytes).expect("utf8");
        assert_eq!(text, "{\"a\":1}\n");
        assert!(!text.ends_with("\n\n"));
    }

    #[test]
    fn decode_round_trips() {
        let value: serde_json::Value = decode::<serde_json::Value>(r#"{"a":1}"#).expect("decode");
        assert_eq!(value["a"], 1);
    }

    #[test]
    fn read_line_skips_blank_and_returns_eof() {
        let mut input = Cursor::new(b"first\n\n  \nsecond\n".to_vec());
        assert_eq!(
            read_line(&mut input).expect("read").as_deref(),
            Some("first")
        );
        assert_eq!(
            read_line(&mut input).expect("read").as_deref(),
            Some("second")
        );
        assert!(read_line(&mut input).expect("read").is_none());
    }

    #[test]
    fn read_line_tolerates_crlf() {
        let mut input = Cursor::new(b"a\r\nb\r\n".to_vec());
        assert_eq!(read_line(&mut input).expect("read").as_deref(), Some("a"));
        assert_eq!(read_line(&mut input).expect("read").as_deref(), Some("b"));
    }
}
