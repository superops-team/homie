use std::collections::{HashSet, VecDeque};
use std::io;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProcessSample {
    pub pid: i32,
    pub start_time: u64,
}

pub fn process_tree(root: i32) -> Vec<ProcessSample> {
    if root <= 1 {
        return Vec::new();
    }
    enumerate(root)
}

pub fn kill_process_tree(root: i32, grace: Duration) -> io::Result<()> {
    if root <= 1 {
        return Ok(());
    }
    let mut samples = enumerate(root);
    signal_group(root, libc::SIGTERM);
    signal_samples(&samples, libc::SIGTERM);
    signal_group(root, libc::SIGCONT);

    let deadline = Instant::now() + grace;
    while Instant::now() < deadline {
        if samples.iter().all(|sample| !is_same_process(*sample)) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    samples.extend(enumerate(root));
    samples.sort_by_key(|sample| sample.pid);
    samples.dedup();
    signal_group(root, libc::SIGKILL);
    signal_samples(&samples, libc::SIGKILL);
    signal_samples(&samples, libc::SIGCONT);
    Ok(())
}

fn enumerate(root: i32) -> Vec<ProcessSample> {
    let holder_pid = std::process::id() as i32;
    let mut seen = HashSet::new();
    let mut queue = VecDeque::from([root]);
    let mut samples = Vec::new();
    while let Some(pid) = queue.pop_front() {
        if pid <= 1 || pid == holder_pid || !seen.insert(pid) {
            continue;
        }
        if let Some(sample) = sample(pid) {
            samples.push(sample);
        }
        for child in child_pids(pid) {
            queue.push_back(child);
        }
        for peer in process_group_pids(pid) {
            queue.push_back(peer);
        }
    }
    samples
}

fn signal_group(root: i32, signal: i32) {
    // SAFETY: kill(2) with a negative pid targets the process group.
    let rc = unsafe { libc::kill(-root, signal) };
    if rc < 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
        // SAFETY: fallback to the root process when the process group no
        // longer exists or was not the root pid.
        unsafe {
            libc::kill(root, signal);
        }
    }
}

fn signal_samples(samples: &[ProcessSample], signal: i32) {
    for sample in samples {
        if is_same_process(*sample) {
            // SAFETY: pid was sampled and checked immediately before signal.
            unsafe {
                libc::kill(sample.pid, signal);
            }
        }
    }
}

fn is_same_process(sample: ProcessSample) -> bool {
    start_time(sample.pid) == Some(sample.start_time)
}

fn sample(pid: i32) -> Option<ProcessSample> {
    Some(ProcessSample {
        pid,
        start_time: start_time(pid)?,
    })
}

#[cfg(target_os = "macos")]
fn child_pids(pid: i32) -> Vec<i32> {
    let count = unsafe { libc::proc_listchildpids(pid, std::ptr::null_mut(), 0) };
    if count <= 0 {
        return Vec::new();
    }
    let mut buffer = vec![0_i32; count as usize + 8];
    let bytes = (buffer.len() * std::mem::size_of::<i32>()) as i32;
    let filled = unsafe { libc::proc_listchildpids(pid, buffer.as_mut_ptr().cast(), bytes) };
    if filled <= 0 {
        return Vec::new();
    }
    buffer
        .into_iter()
        .take(filled as usize)
        .filter(|pid| *pid > 1)
        .collect()
}

#[cfg(target_os = "macos")]
fn process_group_pids(_pid: i32) -> Vec<i32> {
    Vec::new()
}

#[cfg(target_os = "macos")]
fn start_time(pid: i32) -> Option<u64> {
    let mut info = unsafe { std::mem::zeroed::<libc::proc_taskallinfo>() };
    let size = std::mem::size_of::<libc::proc_taskallinfo>() as i32;
    let filled = unsafe {
        libc::proc_pidinfo(
            pid,
            libc::PROC_PIDTASKALLINFO,
            0,
            (&mut info as *mut libc::proc_taskallinfo).cast(),
            size,
        )
    };
    (filled == size).then_some(info.pbsd.pbi_start_tvsec)
}

#[cfg(target_os = "linux")]
fn child_pids(pid: i32) -> Vec<i32> {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().to_string_lossy().parse::<i32>().ok())
        .filter(|candidate| parent_pid(*candidate) == Some(pid))
        .collect()
}

#[cfg(target_os = "linux")]
fn process_group_pids(pid: i32) -> Vec<i32> {
    let Some(group) = process_group(pid) else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().to_string_lossy().parse::<i32>().ok())
        .filter(|candidate| process_group(*candidate) == Some(group))
        .collect()
}

#[cfg(target_os = "linux")]
fn parent_pid(pid: i32) -> Option<i32> {
    stat_fields(pid).and_then(|fields| fields.get(1).and_then(|value| value.parse().ok()))
}

#[cfg(target_os = "linux")]
fn process_group(pid: i32) -> Option<i32> {
    stat_fields(pid).and_then(|fields| fields.get(2).and_then(|value| value.parse().ok()))
}

#[cfg(target_os = "linux")]
fn start_time(pid: i32) -> Option<u64> {
    stat_fields(pid).and_then(|fields| fields.get(19).and_then(|value| value.parse().ok()))
}

#[cfg(target_os = "linux")]
fn stat_fields(pid: i32) -> Option<Vec<String>> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let end = stat.rfind(") ")?;
    Some(
        stat[end + 2..]
            .split_whitespace()
            .map(str::to_string)
            .collect(),
    )
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn child_pids(_pid: i32) -> Vec<i32> {
    Vec::new()
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn process_group_pids(_pid: i32) -> Vec<i32> {
    Vec::new()
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn start_time(pid: i32) -> Option<u64> {
    let rc = unsafe { libc::kill(pid, 0) };
    (rc == 0).then_some(1)
}
