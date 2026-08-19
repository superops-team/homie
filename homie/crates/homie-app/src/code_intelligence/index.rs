use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

use super::text::excerpt;
use super::{
    CodeIntelligenceError, IGNORED_DIRECTORIES, MAX_INDEX_FILES, MAX_SYMBOL_BYTES,
    MAX_SYMBOL_INDEX_BYTES, MAX_SYMBOLS_PER_FILE, SourceLanguage,
};

#[derive(Clone, Debug)]
pub(super) struct WorkspaceIndex {
    pub(super) files: Vec<IndexedFile>,
    pub(super) symbols: Vec<IndexedSymbol>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct IndexedFile {
    pub(super) relative_path: PathBuf,
    pub(super) display_path: String,
}

#[derive(Clone, Debug)]
pub(super) struct IndexedSymbol {
    pub(super) relative_path: PathBuf,
    pub(super) name: String,
    pub(super) line: usize,
    pub(super) preview: String,
}

pub(super) fn discover_workspace_root(
    session_cwd: &Path,
) -> Result<PathBuf, CodeIntelligenceError> {
    if let Ok(output) = Command::new("git")
        .current_dir(session_cwd)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        && output.status.success()
    {
        let root = String::from_utf8_lossy(&output.stdout);
        let root = root.trim();
        if !root.is_empty() {
            return fs::canonicalize(root).map_err(|error| {
                CodeIntelligenceError::WorkspaceUnavailable {
                    path: PathBuf::from(root),
                    message: error.to_string(),
                }
            });
        }
    }

    for ancestor in session_cwd.ancestors() {
        if ancestor.join(".git").exists() {
            return Ok(ancestor.to_path_buf());
        }
    }
    Ok(session_cwd.to_path_buf())
}

pub(super) fn build_index(workspace_root: &Path) -> WorkspaceIndex {
    let paths = git_paths(workspace_root).unwrap_or_else(|| filesystem_paths(workspace_root));
    let mut files = Vec::with_capacity(paths.len());
    let mut symbols = Vec::new();
    let mut symbol_bytes = 0u64;

    for relative_path in paths {
        if files.len() >= MAX_INDEX_FILES || ignored_path(&relative_path) {
            continue;
        }
        let absolute_path = workspace_root.join(&relative_path);
        let Ok(metadata) = fs::symlink_metadata(&absolute_path) else {
            continue;
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            continue;
        }

        let display_path = relative_path.to_string_lossy().replace('\\', "/");
        let language = SourceLanguage::from_path(&relative_path);
        files.push(IndexedFile {
            relative_path: relative_path.clone(),
            display_path,
        });
        let symbol_candidate =
            language != SourceLanguage::PlainText || is_symbol_text_file(&relative_path);
        if symbol_candidate
            && metadata.len() <= MAX_SYMBOL_BYTES
            && symbol_bytes.saturating_add(metadata.len()) <= MAX_SYMBOL_INDEX_BYTES
        {
            symbol_bytes = symbol_bytes.saturating_add(metadata.len());
            symbols.extend(index_symbols(&absolute_path, &relative_path, language));
        }
    }

    files.sort_by(|left, right| left.display_path.cmp(&right.display_path));
    symbols.sort_by(|left, right| {
        left.relative_path
            .cmp(&right.relative_path)
            .then_with(|| left.line.cmp(&right.line))
    });
    WorkspaceIndex { files, symbols }
}

fn git_paths(workspace_root: &Path) -> Option<Vec<PathBuf>> {
    let output = Command::new("git")
        .current_dir(workspace_root)
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    Some(
        output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .take(MAX_INDEX_FILES)
            .map(bytes_to_path)
            .collect(),
    )
}

#[cfg(unix)]
fn bytes_to_path(bytes: &[u8]) -> PathBuf {
    PathBuf::from(OsString::from_vec(bytes.to_vec()))
}

#[cfg(not(unix))]
fn bytes_to_path(bytes: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(bytes).into_owned())
}

fn filesystem_paths(workspace_root: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut pending = vec![workspace_root.to_path_buf()];

    while let Some(directory) = pending.pop() {
        if result.len() >= MAX_INDEX_FILES {
            break;
        }
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries.into_iter().rev() {
            let path = entry.path();
            let Ok(relative) = path.strip_prefix(workspace_root) else {
                continue;
            };
            if ignored_path(relative) {
                continue;
            }
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                pending.push(path);
            } else if file_type.is_file() {
                result.push(relative.to_path_buf());
                if result.len() >= MAX_INDEX_FILES {
                    break;
                }
            }
        }
    }
    result
}

fn ignored_path(path: &Path) -> bool {
    path.components().any(|component| {
        let Component::Normal(name) = component else {
            return false;
        };
        let name = name.to_string_lossy();
        IGNORED_DIRECTORIES
            .iter()
            .any(|ignored| name.eq_ignore_ascii_case(ignored))
    })
}

fn index_symbols(
    absolute_path: &Path,
    relative_path: &Path,
    language: SourceLanguage,
) -> Vec<IndexedSymbol> {
    let Ok(file) = File::open(absolute_path) else {
        return Vec::new();
    };
    let mut bytes = Vec::new();
    if file.take(MAX_SYMBOL_BYTES).read_to_end(&mut bytes).is_err() || bytes.contains(&0) {
        return Vec::new();
    }
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return Vec::new();
    };

    text.lines()
        .enumerate()
        .filter_map(|(line, source)| {
            let name = symbol_name(source, language)?;
            Some(IndexedSymbol {
                relative_path: relative_path.to_path_buf(),
                name,
                line: line + 1,
                preview: excerpt(source.trim(), 180),
            })
        })
        .take(MAX_SYMBOLS_PER_FILE)
        .collect()
}

fn symbol_name(line: &str, language: SourceLanguage) -> Option<String> {
    let mut line = line.trim_start();
    if line.is_empty()
        || line
            .chars()
            .next()
            .is_some_and(|character| matches!(character, '/' | '#' | '*'))
    {
        return None;
    }

    // Remove common access and declaration modifiers without exposing a full
    // parser at the module seam. The language-specific declaration token below
    // keeps ordinary prose and expressions out of the symbol index.
    loop {
        let next = [
            "pub(crate) ",
            "pub(super) ",
            "pub ",
            "public ",
            "private ",
            "protected ",
            "internal ",
            "open ",
            "final ",
            "static ",
            "async ",
            "export default ",
            "export ",
            "default ",
        ]
        .into_iter()
        .find_map(|prefix| line.strip_prefix(prefix));
        let Some(next) = next else { break };
        line = next.trim_start();
    }

    let prefixes: &[&str] = match language {
        SourceLanguage::Rust => &[
            "fn ", "struct ", "enum ", "trait ", "type ", "const ", "static ", "mod ",
        ],
        SourceLanguage::Swift => &[
            "func ",
            "struct ",
            "class ",
            "enum ",
            "protocol ",
            "actor ",
            "typealias ",
            "let ",
            "var ",
        ],
        SourceLanguage::TypeScript
        | SourceLanguage::Tsx
        | SourceLanguage::JavaScript
        | SourceLanguage::Jsx => &[
            "function ",
            "class ",
            "interface ",
            "type ",
            "enum ",
            "const ",
            "let ",
            "var ",
        ],
        SourceLanguage::Python => &["def ", "class ", "async def "],
        SourceLanguage::Go => &["func ", "type ", "const ", "var "],
        SourceLanguage::Java | SourceLanguage::Kotlin | SourceLanguage::CSharp => &[
            "class ",
            "interface ",
            "enum ",
            "record ",
            "fun ",
            "object ",
        ],
        SourceLanguage::C | SourceLanguage::Cpp => {
            &["class ", "struct ", "enum ", "namespace ", "typedef "]
        }
        SourceLanguage::Ruby => &["def ", "class ", "module "],
        SourceLanguage::Shell => &["function "],
        SourceLanguage::Css => &["@keyframes ", "@mixin ", "@function "],
        SourceLanguage::Sql => &["CREATE TABLE ", "CREATE VIEW ", "CREATE FUNCTION "],
        SourceLanguage::Markdown => &["# ", "## ", "### ", "#### ", "##### ", "###### "],
        _ => &[],
    };

    let declaration = prefixes.iter().find_map(|prefix| {
        if language == SourceLanguage::Sql {
            line.to_ascii_uppercase()
                .starts_with(prefix)
                .then(|| &line[prefix.len()..])
        } else {
            line.strip_prefix(prefix)
        }
    })?;
    let name: String = declaration
        .trim_start_matches(['&', '*'])
        .chars()
        .take_while(|character| {
            character.is_alphanumeric()
                || matches!(character, '_' | '-' | '.' | ':' | '<' | '>' | '?')
        })
        .collect();
    (!name.is_empty()).then_some(name)
}

fn is_symbol_text_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                "Dockerfile" | "Makefile" | "CMakeLists.txt" | "Justfile"
            )
        })
}
