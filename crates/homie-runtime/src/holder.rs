use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "op")]
pub enum HolderRequest {
    Write { text: String, submit: bool },
    WriteBytes { bytes: Vec<u8> },
    Resize { cols: u16, rows: u16 },
    Terminate,
    Stat,
    KillTree,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HolderResponse {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree_size: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cols: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_offset: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epoch_offset: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct HolderPaths {
    pub socket: PathBuf,
    pub pid_file: PathBuf,
    pub status_file: PathBuf,
}

impl HolderPaths {
    pub fn new(data_dir: &Path, session_id: &str) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        data_dir.hash(&mut hasher);
        session_id.hash(&mut hasher);
        let token = format!("{:016x}", hasher.finish());
        let dir = std::env::temp_dir().join("homie-runtime-holders");
        Self {
            socket: dir.join(format!("{token}.sock")),
            pid_file: dir.join(format!("{token}.pid")),
            status_file: dir.join(format!("{token}.status")),
        }
    }
}

pub fn request(socket: &Path, request: &HolderRequest) -> std::io::Result<HolderResponse> {
    let mut stream = UnixStream::connect(socket)?;
    stream.set_read_timeout(Some(Duration::from_millis(350)))?;
    stream.set_write_timeout(Some(Duration::from_millis(350)))?;
    let mut encoded = serde_json::to_vec(request)?;
    encoded.push(b'\n');
    stream.write_all(&encoded)?;
    stream.flush()?;
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line)?;
    serde_json::from_str(&line).map_err(std::io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::HolderRequest;

    #[test]
    fn write_bytes_round_trips_arbitrary_bytes_without_text_conversion() {
        let bytes = vec![0x00, 0xff, 0x80, b'\n'];
        let encoded = serde_json::to_vec(&HolderRequest::WriteBytes {
            bytes: bytes.clone(),
        })
        .expect("encode");
        let decoded: HolderRequest = serde_json::from_slice(&encoded).expect("decode");

        let HolderRequest::WriteBytes {
            bytes: decoded_bytes,
        } = decoded
        else {
            panic!("unexpected holder request");
        };
        assert_eq!(decoded_bytes, bytes);
    }
}
