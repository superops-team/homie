//! Inspector tab state machine and review/ask workflow state types.
//!
//! Pure data types only: no `Window`/`Context`/`Entity`/render dependency, so
//! they stay unit-testable in isolation from the GPUI view.

use std::path::PathBuf;
use std::sync::Arc;

use homie_proto::SessionId;

use crate::diff::DiffSnapshot;
use crate::git_review::{PatchMutation, ReviewStatus};
use crate::review_prompt::ReviewEvidence;
use crate::store::InspectorTab;

impl InspectorTab {
    pub(crate) const ALL: [Self; 4] = [Self::Info, Self::Changes, Self::Code, Self::Artifacts];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Changes => "Review",
            Self::Code => "Code",
            Self::Artifacts => "Artifacts",
        }
    }

    pub(crate) const fn index(self) -> i8 {
        match self {
            Self::Info => 0,
            Self::Changes => 1,
            Self::Code => 2,
            Self::Artifacts => 3,
        }
    }

    pub(crate) const fn debug_selector(self) -> &'static str {
        match self {
            Self::Info => "INSPECTOR_TAB_INFO",
            Self::Changes => "INSPECTOR_TAB_CHANGES",
            Self::Code => "INSPECTOR_TAB_CODE",
            Self::Artifacts => "INSPECTOR_TAB_ARTIFACTS",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DiffContext {
    pub(crate) id: SessionId,
    pub(crate) cwd: PathBuf,
    pub(crate) remote: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LoadState {
    NoSession,
    Loading,
    Ready(Arc<DiffSnapshot>),
    Error(String),
}

#[derive(Clone, Debug)]
pub(crate) enum ReviewLoadState {
    NoSession,
    Remote,
    Loading,
    Ready(Arc<ReviewStatus>),
    Error(String),
}

#[derive(Clone, Debug)]
pub(crate) enum ReviewAction {
    Stage(Vec<PathBuf>),
    Unstage(Vec<PathBuf>),
    Discard(Vec<PathBuf>),
    Patch {
        patch: Vec<u8>,
        mutation: PatchMutation,
    },
    Commit(String),
}

#[derive(Clone, Debug)]
pub(crate) struct AskDraft {
    pub(crate) evidence: Vec<ReviewEvidence>,
    pub(crate) label: String,
}
