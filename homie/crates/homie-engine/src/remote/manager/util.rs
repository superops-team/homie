use std::io;

use homie_proto::HostEntry;
use homie_proto::remote_pty::PersistenceCapability;

pub(crate) fn parse_json_line<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> io::Result<T> {
    for line in bytes.split(|byte| *byte == b'\n').rev() {
        let line = line
            .iter()
            .copied()
            .skip_while(u8::is_ascii_whitespace)
            .collect::<Vec<_>>();
        if line.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_slice(&line) {
            return Ok(value);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Helper JSON response is missing or invalid",
    ))
}

pub(crate) fn random_hex(bytes: usize) -> io::Result<String> {
    let mut random = vec![0_u8; bytes];
    getrandom::fill(&mut random)
        .map_err(|error| io::Error::other(format!("secure random source failed: {error}")))?;
    Ok(random.iter().map(|byte| format!("{byte:02x}")).collect())
}

pub(crate) fn classify_persistence(native: bool, supervised: bool) -> PersistenceCapability {
    if native {
        PersistenceCapability::NativeDetach
    } else if supervised {
        PersistenceCapability::UserSupervisor
    } else {
        PersistenceCapability::NonPersistent
    }
}

pub(crate) fn persistence_key(host: &HostEntry) -> String {
    format!("{}\0{}", host.id, host.ssh)
}
