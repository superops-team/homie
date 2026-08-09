use std::ffi::{CStr, OsString};
use std::io;
use std::os::unix::ffi::OsStringExt as _;
use std::path::PathBuf;
use std::ptr;

const DEFAULT_PASSWD_BUFFER: usize = 16 * 1024;
const MAX_PASSWD_BUFFER: usize = 1024 * 1024;

pub fn default_data_dir() -> io::Result<PathBuf> {
    Ok(current_user_home()?
        .join("Library")
        .join("Application Support")
        .join("Homie"))
}

fn current_user_home() -> io::Result<PathBuf> {
    let configured_size = unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) };
    let mut size = if configured_size > 0 {
        usize::try_from(configured_size)
            .unwrap_or(DEFAULT_PASSWD_BUFFER)
            .clamp(DEFAULT_PASSWD_BUFFER, MAX_PASSWD_BUFFER)
    } else {
        DEFAULT_PASSWD_BUFFER
    };

    loop {
        let mut buffer = vec![0_u8; size];
        let mut passwd = unsafe { std::mem::zeroed::<libc::passwd>() };
        let mut result = ptr::null_mut();
        let status = unsafe {
            libc::getpwuid_r(
                libc::geteuid(),
                &mut passwd,
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                &mut result,
            )
        };
        if status == libc::ERANGE && size < MAX_PASSWD_BUFFER {
            size = size.saturating_mul(2).min(MAX_PASSWD_BUFFER);
            continue;
        }
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status));
        }
        if result.is_null() || passwd.pw_dir.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "current user home directory is unavailable",
            ));
        }

        let home = unsafe { CStr::from_ptr(passwd.pw_dir) }.to_bytes();
        let home = PathBuf::from(OsString::from_vec(home.to_vec()));
        if home.is_absolute() {
            return Ok(home);
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "current user home directory is not absolute",
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_data_directory_is_absolute_and_account_owned() {
        let home = current_user_home().expect("current user home");
        let data_dir = default_data_dir().expect("default data dir");

        assert!(home.is_absolute());
        assert_eq!(
            data_dir,
            home.join("Library")
                .join("Application Support")
                .join("Homie")
        );
    }
}
