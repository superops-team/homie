use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HostPreparationKind {
    Initialize,
    Reinstall,
}

#[derive(Clone, Debug)]
pub(crate) enum HostInitialization {
    Running {
        id: String,
        name: String,
        kind: HostPreparationKind,
        operation: u64,
    },
    Ready {
        id: String,
        name: String,
        kind: HostPreparationKind,
        operation: u64,
        result: homie_proto::HostInitializeResult,
    },
    Failed {
        id: String,
        name: String,
        kind: HostPreparationKind,
        operation: u64,
        message: String,
    },
}

pub(crate) struct HostInitializationCardModel {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) symbol: Option<&'static str>,
    pub(crate) title: String,
    pub(crate) detail: String,
    pub(crate) tone: Rgba,
    pub(crate) action: Option<&'static str>,
    pub(crate) retry_kind: Option<HostPreparationKind>,
}

impl HostInitialization {
    pub(crate) fn id(&self) -> &str {
        match self {
            Self::Running { id, .. } | Self::Ready { id, .. } | Self::Failed { id, .. } => id,
        }
    }

    pub(crate) fn operation(&self) -> u64 {
        match self {
            Self::Running { operation, .. }
            | Self::Ready { operation, .. }
            | Self::Failed { operation, .. } => *operation,
        }
    }
}

pub(crate) fn expire_completed_reinstall(
    state: &mut Option<HostInitialization>,
    id: &str,
    operation: u64,
) -> bool {
    let should_expire = matches!(
        state.as_ref(),
        Some(HostInitialization::Ready {
            id: state_id,
            kind: HostPreparationKind::Reinstall,
            operation: state_operation,
            ..
        }) if state_id == id && *state_operation == operation
    );
    if should_expire {
        *state = None;
    }
    should_expire
}
