use std::fs;
use std::io;
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::MAX_CONTROL_DIRECTORY_BYTES;

pub(crate) fn validate_control_dir_if_present(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata)
            if metadata.file_type().is_symlink()
                || !metadata.is_dir()
                || metadata.uid() != effective_uid() =>
        {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "SSH control directory must be a real directory owned by the current user",
            ))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub(crate) fn normalized_control_dir(requested: &Path) -> PathBuf {
    if requested.as_os_str().as_bytes().len() <= MAX_CONTROL_DIRECTORY_BYTES {
        return requested.to_path_buf();
    }
    let digest = Sha256::digest(requested.as_os_str().as_bytes());
    let suffix = digest[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    // `/tmp` is intentionally explicit: macOS's TMPDIR path is commonly long
    // enough to reproduce the same sockaddr_un failure. The sticky parent and
    // the owner/type checks in `new` protect this private child directory.
    PathBuf::from(format!("/tmp/homie-ssh-{}-{suffix}", effective_uid()))
}

fn effective_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and does not access caller-owned
    // memory; it returns the kernel credential for this process.
    unsafe { libc::geteuid() }
}
