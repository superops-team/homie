use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

use super::{BranchInfo, ChangeKind, FileChange, GitReviewError, ReviewStatus};

const MAX_STATUS_ENTRIES: usize = 20_000;

pub(super) fn parse_status(root: &Path, bytes: &[u8]) -> Result<ReviewStatus, GitReviewError> {
    let records: Vec<&[u8]> = bytes
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
        .collect();
    let mut status = ReviewStatus {
        repo_root: root.to_path_buf(),
        ..ReviewStatus::default()
    };
    let mut index = 0;
    let mut entries = 0;

    while index < records.len() {
        let record = records[index];
        index += 1;

        if let Some(header) = record.strip_prefix(b"# ") {
            parse_branch_header(&mut status.branch, header)?;
            continue;
        }

        entries += 1;
        if entries > MAX_STATUS_ENTRIES {
            return Err(GitReviewError::OutputTooLarge {
                operation: "reading status",
                limit: MAX_STATUS_ENTRIES,
            });
        }

        match record.first().copied() {
            Some(b'1') => {
                let fields = split_fields(record, 9);
                require_fields(&fields, 9, "ordinary entry")?;
                add_tracked_change(&mut status, fields[1], fields[8], None, false)?;
            }
            Some(b'2') => {
                let fields = split_fields(record, 10);
                require_fields(&fields, 10, "rename/copy entry")?;
                let original = records.get(index).copied().ok_or_else(|| {
                    GitReviewError::MalformedStatus(
                        "rename/copy entry has no original path".to_owned(),
                    )
                })?;
                index += 1;
                add_tracked_change(
                    &mut status,
                    fields[1],
                    fields[9],
                    Some(path_from_bytes(original)),
                    false,
                )?;
            }
            Some(b'u') => {
                let fields = split_fields(record, 11);
                require_fields(&fields, 11, "unmerged entry")?;
                add_tracked_change(&mut status, fields[1], fields[10], None, true)?;
            }
            Some(b'?') if record.get(1) == Some(&b' ') => {
                status.untracked.push(FileChange {
                    path: path_from_bytes(&record[2..]),
                    original_path: None,
                    kind: ChangeKind::Added,
                });
            }
            Some(other) => {
                return Err(GitReviewError::MalformedStatus(format!(
                    "unknown entry kind {:?}",
                    char::from(other)
                )));
            }
            None => {}
        }
    }

    Ok(status)
}

fn parse_branch_header(branch: &mut BranchInfo, header: &[u8]) -> Result<(), GitReviewError> {
    if let Some(value) = header.strip_prefix(b"branch.oid ") {
        if value == b"(initial)" {
            branch.oid = None;
        } else {
            branch.oid = Some(String::from_utf8_lossy(value).into_owned());
        }
    } else if let Some(value) = header.strip_prefix(b"branch.head ") {
        if value == b"(detached)" {
            branch.name = None;
        } else {
            branch.name = Some(String::from_utf8_lossy(value).into_owned());
        }
    } else if let Some(value) = header.strip_prefix(b"branch.upstream ") {
        branch.upstream = Some(String::from_utf8_lossy(value).into_owned());
    } else if let Some(value) = header.strip_prefix(b"branch.ab ") {
        let fields = split_fields(value, 2);
        require_fields(&fields, 2, "branch ahead/behind header")?;
        branch.ahead = parse_prefixed_count(fields[0], b'+')?;
        branch.behind = parse_prefixed_count(fields[1], b'-')?;
    }
    Ok(())
}

fn parse_prefixed_count(value: &[u8], prefix: u8) -> Result<u64, GitReviewError> {
    let Some(number) = value.strip_prefix(&[prefix]) else {
        return Err(GitReviewError::MalformedStatus(format!(
            "branch count {:?} has the wrong prefix",
            String::from_utf8_lossy(value)
        )));
    };
    String::from_utf8_lossy(number).parse().map_err(|_| {
        GitReviewError::MalformedStatus(format!(
            "branch count {:?} is not a number",
            String::from_utf8_lossy(value)
        ))
    })
}

fn add_tracked_change(
    status: &mut ReviewStatus,
    xy: &[u8],
    path: &[u8],
    original_path: Option<PathBuf>,
    explicitly_unmerged: bool,
) -> Result<(), GitReviewError> {
    if xy.len() != 2 {
        return Err(GitReviewError::MalformedStatus(format!(
            "XY status {:?} is not two bytes",
            String::from_utf8_lossy(xy)
        )));
    }
    let index_kind = xy[0];
    let worktree_kind = xy[1];
    let path = path_from_bytes(path);

    if explicitly_unmerged || is_unmerged(index_kind, worktree_kind) {
        status.conflicted.push(FileChange {
            path,
            original_path,
            kind: ChangeKind::Unmerged,
        });
        return Ok(());
    }

    if index_kind != b'.' {
        status.staged.push(FileChange {
            path: path.clone(),
            original_path: original_path.clone(),
            kind: change_kind(index_kind),
        });
    }
    if worktree_kind != b'.' {
        status.unstaged.push(FileChange {
            path,
            original_path,
            kind: change_kind(worktree_kind),
        });
    }
    Ok(())
}

fn is_unmerged(index: u8, worktree: u8) -> bool {
    index == b'U' || worktree == b'U' || matches!((index, worktree), (b'D', b'D') | (b'A', b'A'))
}

fn change_kind(value: u8) -> ChangeKind {
    match value {
        b'A' => ChangeKind::Added,
        b'M' => ChangeKind::Modified,
        b'D' => ChangeKind::Deleted,
        b'R' => ChangeKind::Renamed,
        b'C' => ChangeKind::Copied,
        b'T' => ChangeKind::TypeChanged,
        b'U' => ChangeKind::Unmerged,
        other => ChangeKind::Unknown(char::from(other)),
    }
}

fn split_fields(bytes: &[u8], count: usize) -> Vec<&[u8]> {
    bytes.splitn(count, |byte| *byte == b' ').collect()
}

fn require_fields(
    fields: &[&[u8]],
    expected: usize,
    description: &str,
) -> Result<(), GitReviewError> {
    if fields.len() == expected {
        Ok(())
    } else {
        Err(GitReviewError::MalformedStatus(format!(
            "{description} has {} fields, expected {expected}",
            fields.len()
        )))
    }
}

#[cfg(unix)]
fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    PathBuf::from(OsString::from_vec(bytes.to_vec()))
}

#[cfg(not(unix))]
fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(bytes).into_owned())
}

pub(super) fn path_from_output_line(bytes: &[u8]) -> PathBuf {
    path_from_bytes(trim_line_ending(bytes))
}

pub(super) fn trim_line_ending(mut bytes: &[u8]) -> &[u8] {
    if bytes.ends_with(b"\n") {
        bytes = &bytes[..bytes.len() - 1];
    }
    if bytes.ends_with(b"\r") {
        bytes = &bytes[..bytes.len() - 1];
    }
    bytes
}
