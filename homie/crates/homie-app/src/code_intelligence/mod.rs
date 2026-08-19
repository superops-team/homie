//! Lightweight workspace intelligence for native file browsing.
//!
//! Callers cross one seam: create a [`CodeIntelligence`] value for a session,
//! then resolve/open terminal references or search its cached workspace index.
//! Git invocation, traversal safety, file bounds, language classification,
//! symbol heuristics, and ranking remain implementation details. All methods
//! are blocking by design so GPUI can dispatch them to its background executor.

use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

mod index;
mod reference;
#[cfg(test)]
mod tests;
mod text;

use index::{WorkspaceIndex, build_index, discover_workspace_root};
use reference::parse_reference_candidates;
use text::{clamp_target, excerpt, fuzzy_score, lexical_normalize, path_score, source_lines};

/// Source files larger than this are not loaded into the native viewer.
pub const MAX_SOURCE_BYTES: u64 = 2 * 1024 * 1024;

const MAX_INDEX_FILES: usize = 50_000;
const MAX_SYMBOL_BYTES: u64 = 256 * 1024;
const MAX_SYMBOL_INDEX_BYTES: u64 = 32 * 1024 * 1024;
const MAX_SYMBOLS_PER_FILE: usize = 256;

const IGNORED_DIRECTORIES: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    ".build",
    ".next",
    "build",
    "coverage",
    "DerivedData",
    "dist",
    "node_modules",
    "Pods",
    "target",
    "vendor",
];

/// The deep module exposed to the workbench and terminal adapters.
///
/// Construction and lazy indexing perform blocking filesystem work. The value
/// is owned, `Clone`, `Send`, and `Sync`, so callers can move it through a
/// background task and retain the resulting index for subsequent searches.
#[derive(Clone, Debug)]
pub struct CodeIntelligence {
    workspace_root: PathBuf,
    session_cwd: PathBuf,
    index: OnceLock<WorkspaceIndex>,
}

impl CodeIntelligence {
    /// Resolve the session directory to a canonical Git root when possible,
    /// otherwise use the canonical session directory as the workspace root.
    /// Indexing is lazy: terminal references can open immediately, while the
    /// file tree and search index are built on their first use.
    pub fn for_session(cwd: impl AsRef<Path>) -> Result<Self, CodeIntelligenceError> {
        let requested = cwd.as_ref();
        let canonical = fs::canonicalize(requested).map_err(|error| {
            CodeIntelligenceError::WorkspaceUnavailable {
                path: requested.to_path_buf(),
                message: error.to_string(),
            }
        })?;
        let session_cwd = if canonical.is_file() {
            canonical.parent().map(Path::to_path_buf).ok_or_else(|| {
                CodeIntelligenceError::WorkspaceUnavailable {
                    path: canonical.clone(),
                    message: "the session path has no parent directory".to_string(),
                }
            })?
        } else if canonical.is_dir() {
            canonical
        } else {
            return Err(CodeIntelligenceError::WorkspaceUnavailable {
                path: canonical,
                message: "the session path is not a directory".to_string(),
            });
        };

        let workspace_root = discover_workspace_root(&session_cwd)?;
        Ok(Self {
            workspace_root,
            session_cwd,
            index: OnceLock::new(),
        })
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    fn index(&self) -> &WorkspaceIndex {
        self.index.get_or_init(|| build_index(&self.workspace_root))
    }

    /// Resolve a terminal-shaped reference without loading its contents.
    /// Relative paths are attempted from the session cwd first and then from
    /// the workspace root. A successful result is always a canonical file
    /// contained by the workspace root.
    pub fn resolve_reference(
        &self,
        terminal_text: &str,
    ) -> Result<ResolvedReference, CodeIntelligenceError> {
        let candidates = parse_reference_candidates(terminal_text);
        if candidates.is_empty() {
            return Err(CodeIntelligenceError::NoFileReference {
                text: excerpt(terminal_text, 160),
            });
        }

        let mut first_error = None;
        for candidate in candidates {
            let bases = if candidate.path.is_absolute() || self.session_cwd == self.workspace_root {
                [Some(self.workspace_root.as_path()), None]
            } else {
                [
                    Some(self.session_cwd.as_path()),
                    Some(self.workspace_root.as_path()),
                ]
            };

            for base in bases.into_iter().flatten() {
                let requested = if candidate.path.is_absolute() {
                    candidate.path.clone()
                } else {
                    base.join(&candidate.path)
                };
                match self.resolve_workspace_file(&requested) {
                    Ok((absolute_path, relative_path)) => {
                        return Ok(ResolvedReference {
                            absolute_path,
                            relative_path,
                            target: candidate.target,
                        });
                    }
                    Err(error) => {
                        if first_error.is_none()
                            || matches!(error, CodeIntelligenceError::OutsideWorkspace { .. })
                        {
                            first_error = Some(error);
                        }
                    }
                }
            }
        }

        Err(
            first_error.unwrap_or_else(|| CodeIntelligenceError::NoFileReference {
                text: excerpt(terminal_text, 160),
            }),
        )
    }

    /// Resolve a terminal reference and return a bounded, viewer-ready source
    /// snapshot in one call.
    pub fn open_reference(
        &self,
        terminal_text: &str,
    ) -> Result<SourceSnapshot, CodeIntelligenceError> {
        let reference = self.resolve_reference(terminal_text)?;
        self.load_resolved(reference)
    }

    /// Rank cached file paths and symbol-like declarations. Results are stable
    /// for equal scores and bounded by `limit`; this performs no filesystem I/O.
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchHit> {
        if limit == 0 {
            return Vec::new();
        }

        let query = query.trim();
        let mut results = Vec::new();
        let index = self.index();
        for file in &index.files {
            let score = if query.is_empty() {
                Some(0)
            } else {
                path_score(query, &file.display_path)
            };
            if let Some(score) = score {
                results.push(SearchHit {
                    relative_path: file.relative_path.clone(),
                    kind: SearchHitKind::File,
                    line: None,
                    preview: file.display_path.clone(),
                    score,
                });
            }
        }

        if !query.is_empty() {
            for symbol in &index.symbols {
                let Some(mut score) = fuzzy_score(query, &symbol.name) else {
                    continue;
                };
                if symbol.name.eq_ignore_ascii_case(query) {
                    score = score.saturating_add(6_000);
                } else if symbol
                    .name
                    .to_lowercase()
                    .starts_with(&query.to_lowercase())
                {
                    score = score.saturating_add(2_500);
                }
                results.push(SearchHit {
                    relative_path: symbol.relative_path.clone(),
                    kind: SearchHitKind::Symbol,
                    line: Some(symbol.line),
                    preview: symbol.preview.clone(),
                    score: score.saturating_add(1_000),
                });
            }
        }

        results.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.relative_path.cmp(&right.relative_path))
                .then_with(|| left.line.cmp(&right.line))
                .then_with(|| left.kind.cmp(&right.kind))
        });
        results.truncate(limit);
        results
    }

    fn resolve_workspace_file(
        &self,
        requested: &Path,
    ) -> Result<(PathBuf, PathBuf), CodeIntelligenceError> {
        let canonical = fs::canonicalize(requested).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                let lexical = lexical_normalize(requested);
                if lexical.starts_with(&self.workspace_root) {
                    CodeIntelligenceError::NotFound {
                        path: requested.to_path_buf(),
                    }
                } else {
                    CodeIntelligenceError::OutsideWorkspace { path: lexical }
                }
            } else {
                CodeIntelligenceError::Io {
                    path: requested.to_path_buf(),
                    operation: "resolve",
                    message: error.to_string(),
                }
            }
        })?;
        if !canonical.starts_with(&self.workspace_root) {
            return Err(CodeIntelligenceError::OutsideWorkspace { path: canonical });
        }
        if !canonical.is_file() {
            return Err(CodeIntelligenceError::NotAFile { path: canonical });
        }
        let relative = canonical
            .strip_prefix(&self.workspace_root)
            .expect("workspace containment was checked")
            .to_path_buf();
        Ok((canonical, relative))
    }

    fn load_resolved(
        &self,
        reference: ResolvedReference,
    ) -> Result<SourceSnapshot, CodeIntelligenceError> {
        let metadata =
            fs::metadata(&reference.absolute_path).map_err(|error| CodeIntelligenceError::Io {
                path: reference.absolute_path.clone(),
                operation: "inspect",
                message: error.to_string(),
            })?;
        if metadata.len() > MAX_SOURCE_BYTES {
            return Err(CodeIntelligenceError::TooLarge {
                path: reference.absolute_path,
                size: metadata.len(),
                limit: MAX_SOURCE_BYTES,
            });
        }

        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        File::open(&reference.absolute_path)
            .and_then(|file| {
                file.take(MAX_SOURCE_BYTES + 1)
                    .read_to_end(&mut bytes)
                    .map(|_| ())
            })
            .map_err(|error| CodeIntelligenceError::Io {
                path: reference.absolute_path.clone(),
                operation: "read",
                message: error.to_string(),
            })?;
        if bytes.len() as u64 > MAX_SOURCE_BYTES {
            return Err(CodeIntelligenceError::TooLarge {
                path: reference.absolute_path,
                size: bytes.len() as u64,
                limit: MAX_SOURCE_BYTES,
            });
        }
        if bytes.contains(&0) {
            return Err(CodeIntelligenceError::BinaryFile {
                path: reference.absolute_path,
            });
        }
        let text = String::from_utf8(bytes).map_err(|_| CodeIntelligenceError::NotUtf8 {
            path: reference.absolute_path.clone(),
        })?;
        let lines = source_lines(&text);
        let target = reference
            .target
            .map(|target| clamp_target(target, &text, &lines));
        let language = SourceLanguage::from_path(&reference.relative_path);

        Ok(SourceSnapshot {
            absolute_path: reference.absolute_path,
            relative_path: reference.relative_path,
            language,
            text,
            lines,
            target,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedReference {
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
    /// One-based line and column parsed from the terminal, if present.
    pub target: Option<SourceTarget>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SourceTarget {
    /// One-based line number. Loaded snapshots clamp it to the document.
    pub line: usize,
    /// One-based character column, not a UTF-8 byte offset. A loaded snapshot
    /// clamps it to a valid caret position on the target line.
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceSnapshot {
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
    pub language: SourceLanguage,
    pub text: String,
    /// One entry per visual source line, including a final empty line when the
    /// file ends in a newline. Ranges exclude line terminators.
    pub lines: Vec<SourceLine>,
    pub target: Option<SourceTarget>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceLine {
    pub number: usize,
    /// Byte range into [`SourceSnapshot::text`], excluding `\r`/`\n`.
    pub range: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchHit {
    pub relative_path: PathBuf,
    pub kind: SearchHitKind,
    /// A one-based declaration line for symbol results.
    pub line: Option<usize>,
    pub preview: String,
    /// Larger is a better match. The magnitude is intentionally private to
    /// this implementation; callers should only rely on returned ordering.
    pub score: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SearchHitKind {
    Symbol,
    File,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceLanguage {
    Rust,
    Swift,
    TypeScript,
    Tsx,
    JavaScript,
    Jsx,
    Python,
    Go,
    Java,
    Kotlin,
    C,
    Cpp,
    CSharp,
    Ruby,
    Shell,
    Markdown,
    Json,
    Toml,
    Yaml,
    Html,
    Css,
    Sql,
    PlainText,
}

impl SourceLanguage {
    pub fn from_path(path: &Path) -> Self {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        match extension.as_str() {
            "rs" => Self::Rust,
            "swift" => Self::Swift,
            "ts" | "mts" | "cts" => Self::TypeScript,
            "tsx" => Self::Tsx,
            "js" | "mjs" | "cjs" => Self::JavaScript,
            "jsx" => Self::Jsx,
            "py" | "pyi" => Self::Python,
            "go" => Self::Go,
            "java" => Self::Java,
            "kt" | "kts" => Self::Kotlin,
            "c" | "h" => Self::C,
            "cc" | "cpp" | "cxx" | "hh" | "hpp" | "hxx" => Self::Cpp,
            "cs" => Self::CSharp,
            "rb" => Self::Ruby,
            "sh" | "bash" | "zsh" | "fish" => Self::Shell,
            "md" | "mdx" | "markdown" => Self::Markdown,
            "json" | "jsonc" => Self::Json,
            "toml" => Self::Toml,
            "yaml" | "yml" => Self::Yaml,
            "html" | "htm" => Self::Html,
            "css" | "scss" | "sass" | "less" => Self::Css,
            "sql" => Self::Sql,
            _ if matches!(
                file_name.as_str(),
                "bashrc" | "zshrc" | "profile" | ".bashrc" | ".zshrc"
            ) =>
            {
                Self::Shell
            }
            _ => Self::PlainText,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodeIntelligenceError {
    WorkspaceUnavailable {
        path: PathBuf,
        message: String,
    },
    NoFileReference {
        text: String,
    },
    OutsideWorkspace {
        path: PathBuf,
    },
    NotFound {
        path: PathBuf,
    },
    NotAFile {
        path: PathBuf,
    },
    TooLarge {
        path: PathBuf,
        size: u64,
        limit: u64,
    },
    BinaryFile {
        path: PathBuf,
    },
    NotUtf8 {
        path: PathBuf,
    },
    Io {
        path: PathBuf,
        operation: &'static str,
        message: String,
    },
}

impl fmt::Display for CodeIntelligenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkspaceUnavailable { path, message } => {
                write!(
                    formatter,
                    "Cannot use workspace {}: {message}",
                    path.display()
                )
            }
            Self::NoFileReference { text } => {
                write!(formatter, "No file reference found in {text:?}")
            }
            Self::OutsideWorkspace { path } => write!(
                formatter,
                "Refusing to open a path outside the workspace: {}",
                path.display()
            ),
            Self::NotFound { path } => write!(formatter, "File not found: {}", path.display()),
            Self::NotAFile { path } => {
                write!(formatter, "Path is not a file: {}", path.display())
            }
            Self::TooLarge { path, size, limit } => write!(
                formatter,
                "{} is too large for the source viewer ({size} bytes; limit {limit})",
                path.display()
            ),
            Self::BinaryFile { path } => {
                write!(formatter, "{} appears to be a binary file", path.display())
            }
            Self::NotUtf8 { path } => {
                write!(formatter, "{} is not valid UTF-8 text", path.display())
            }
            Self::Io {
                path,
                operation,
                message,
            } => write!(
                formatter,
                "Could not {operation} {}: {message}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for CodeIntelligenceError {}
