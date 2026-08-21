//! Idempotent remote Helper bootstrap and management RPCs.

use std::time::Duration;

mod catalog;
mod control_dir;
mod runtime;
mod util;

pub use catalog::ArtifactCatalog;
pub use runtime::{InstalledHelper, RemoteManager};

#[cfg(test)]
pub(crate) use catalog::verify_required_helper_probe;
#[cfg(test)]
pub(crate) use control_dir::normalized_control_dir;
#[cfg(test)]
pub(crate) use util::{classify_persistence, parse_json_line};

// These wall-clock bounds include time spent in Homie's native OpenSSH
// authentication UI. Network establishment is independently bounded by
// `ConnectTimeout`, and Helper-side environment capture has its own shorter
// deadline, so allowing a human to unlock a key cannot create an unbounded
// remote command.
const PROBE_TIMEOUT: Duration = Duration::from_secs(120);
const UPLOAD_TIMEOUT: Duration = Duration::from_secs(180);
const RPC_TIMEOUT: Duration = Duration::from_secs(120);
const MAX_RPC_OUTPUT: usize = 1024 * 1024;
const MAX_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;
// macOS sockaddr_un.sun_path is 104 bytes. OpenSSH briefly appends a dot and
// 16-byte nonce while creating a multiplex socket, so leave room for the
// per-host digest plus that suffix.
const MAX_CONTROL_DIRECTORY_BYTES: usize = 56;

#[cfg(test)]
mod tests;
