//! Control-channel wire codec.
//!
//! Pure newline-delimited JSON encode/decode plus error mapping for the
//! daemon's front door. These functions have no registry, session, or
//! socket-loop dependency beyond the transport writer itself, so they stay
//! unit-testable without a running daemon.

use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::{Arc, Mutex};

use homie_proto::{ControlError, ControlMessage, JsonValue};
use serde_json::json;

use crate::migrate::MigrateError;

/// Serializes one control message and writes it newline-terminated.
pub(super) fn write_message(
    writer: &Arc<Mutex<UnixStream>>,
    message: &ControlMessage,
) -> std::io::Result<()> {
    let mut bytes = serde_json::to_vec(message)?;
    bytes.push(b'\n');
    let mut stream = writer
        .lock()
        .map_err(|_| std::io::Error::other("writer poisoned"))?;
    stream.write_all(&bytes)?;
    stream.flush()
}

/// Maps a poisoned mutex guard to a stable internal control error.
pub(super) fn poisoned<T>(_: T) -> ControlError {
    ControlError::internal("engine state is poisoned")
}

/// Decodes params into the shared `homie-proto` type for the method — the same
/// types the app itself serializes, so a shape drift is a compile error, not
/// a wire bug.
pub(super) fn decode<T: serde::de::DeserializeOwned>(
    params: Option<JsonValue>,
) -> Result<T, ControlError> {
    serde_json::from_value(params.unwrap_or_else(|| json!({})))
        .map_err(|error| ControlError::bad_request(error.to_string()))
}

/// Serializes a value into the control channel's JSON payload.
pub(super) fn encode<T: serde::Serialize>(value: &T) -> Result<JsonValue, ControlError> {
    serde_json::to_value(value).map_err(|error| ControlError::internal(error.to_string()))
}

/// Resolves a binary on the daemon's PATH, as the readiness check needs.
pub(super) fn resolve_on_path(binary: &str) -> Option<String> {
    if binary.contains('/') {
        return Path::new(binary).exists().then(|| binary.to_string());
    }
    let path = std::env::var("PATH").ok()?;
    for dir in path.split(':') {
        let candidate = Path::new(dir).join(binary);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if std::fs::metadata(&candidate)
                .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
            {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
        #[cfg(not(unix))]
        {
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }
    None
}

/// Maps a migration error onto the control channel's error vocabulary.
pub(super) fn migrate_control_error(error: MigrateError) -> ControlError {
    match error {
        MigrateError::BadRequest(message) => ControlError::bad_request(message),
        MigrateError::Internal(message) => ControlError::internal(message),
    }
}

/// Maps an I/O error onto the control channel's error vocabulary.
pub(super) fn io_control_error(error: std::io::Error) -> ControlError {
    match error.kind() {
        std::io::ErrorKind::NotFound => ControlError::not_found(error.to_string()),
        _ => ControlError::internal(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, PartialEq, Debug)]
    struct Probe {
        id: String,
        count: u32,
    }

    #[test]
    fn decode_round_trips_params() {
        let v = json!({ "id": "s1", "count": 3 });
        let p: Probe = decode(Some(v)).expect("decode");
        assert_eq!(
            p,
            Probe {
                id: "s1".into(),
                count: 3
            }
        );
    }

    #[test]
    fn decode_defaults_missing_params_to_empty_object() {
        let v: JsonValue = decode(None).expect("decode default");
        assert_eq!(v, json!({}));
    }

    #[test]
    fn decode_reports_bad_request_on_shape_mismatch() {
        let err = decode::<Probe>(Some(json!({ "id": 123 }))).unwrap_err();
        assert_eq!(err.code, "bad_request");
    }

    #[test]
    fn encode_round_trips_values() {
        let v = encode(&Probe {
            id: "x".into(),
            count: 7,
        })
        .expect("encode");
        assert_eq!(v["id"], json!("x"));
        assert_eq!(v["count"], json!(7));
    }

    #[test]
    fn io_error_maps_not_found_and_others() {
        let nf = io_control_error(std::io::Error::from(std::io::ErrorKind::NotFound));
        assert_eq!(nf.code, "not_found");
        let other = io_control_error(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
        assert_eq!(other.code, "internal");
    }

    #[test]
    fn migrate_error_maps_bad_request_and_internal() {
        let br = migrate_control_error(MigrateError::BadRequest("b".into()));
        assert_eq!(br.code, "bad_request");
        let internal = migrate_control_error(MigrateError::Internal("i".into()));
        assert_eq!(internal.code, "internal");
    }

    #[cfg(unix)]
    #[test]
    fn resolve_on_path_accepts_existing_absolute_binary() {
        let found = resolve_on_path("/bin/sh");
        assert_eq!(found.as_deref(), Some("/bin/sh"));
    }

    #[test]
    fn resolve_on_path_rejects_missing_absolute_binary() {
        assert_eq!(resolve_on_path("/no/such/binary/here"), None);
    }
}
