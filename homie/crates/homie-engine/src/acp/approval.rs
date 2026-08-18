//! Permission approval four-state semantics.
//!
//! ACP agents can emit permission/tool requests that need a decision from the
//! user. Homie supports four states: allow/deny for the single request, or
//! always-allow/always-deny remembered for the rest of the session (keyed by
//! permission kind).

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PermissionDecision {
    /// Allow this one request only.
    AllowOnce,
    /// Deny this one request only.
    DenyOnce,
    /// Allow this and all subsequent requests of the same kind this session.
    AlwaysAllow,
    /// Deny this and all subsequent requests of the same kind this session.
    AlwaysDeny,
}

impl PermissionDecision {
    pub fn is_always(&self) -> bool {
        matches!(self, Self::AlwaysAllow | Self::AlwaysDeny)
    }

    pub fn allows(&self) -> bool {
        matches!(self, Self::AllowOnce | Self::AlwaysAllow)
    }

    /// The stable option id reported back to an ACP agent.
    pub fn option_id(&self) -> &'static str {
        match self {
            Self::AllowOnce => "allow",
            Self::DenyOnce => "deny",
            Self::AlwaysAllow => "allow_always",
            Self::AlwaysDeny => "deny_always",
        }
    }
}

/// Remembers `always` decisions per permission kind; `once` decisions are
/// applied immediately and never stored.
#[derive(Clone, Debug, Default)]
pub struct ApprovalMemory {
    always: HashMap<String, PermissionDecision>,
}

impl ApprovalMemory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a decision. Returns the effective decision: `once` decisions are
    /// returned unchanged and not remembered; `always` decisions are stored.
    pub fn record(&mut self, kind: &str, decision: PermissionDecision) -> PermissionDecision {
        if decision.is_always() {
            self.always.insert(kind.to_owned(), decision);
        }
        decision
    }

    /// Return a previously-remembered `always` decision for `kind`, if any.
    pub fn recall(&self, kind: &str) -> Option<PermissionDecision> {
        self.always.get(kind).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn once_decisions_apply_but_are_not_remembered() {
        let mut memory = ApprovalMemory::new();
        assert_eq!(
            memory.record("shell", PermissionDecision::AllowOnce),
            PermissionDecision::AllowOnce
        );
        assert_eq!(memory.recall("shell"), None);
    }

    #[test]
    fn always_decisions_are_recalled_by_kind() {
        let mut memory = ApprovalMemory::new();
        memory.record("file_write", PermissionDecision::AlwaysAllow);
        assert_eq!(
            memory.recall("file_write"),
            Some(PermissionDecision::AlwaysAllow)
        );
        // Different kind is unaffected.
        assert_eq!(memory.recall("shell"), None);
    }

    #[test]
    fn later_always_overrides_earlier() {
        let mut memory = ApprovalMemory::new();
        memory.record("shell", PermissionDecision::AlwaysAllow);
        memory.record("shell", PermissionDecision::AlwaysDeny);
        assert_eq!(memory.recall("shell"), Some(PermissionDecision::AlwaysDeny));
    }

    #[test]
    fn option_ids_are_stable() {
        assert_eq!(PermissionDecision::AllowOnce.option_id(), "allow");
        assert_eq!(PermissionDecision::DenyOnce.option_id(), "deny");
        assert_eq!(PermissionDecision::AlwaysAllow.option_id(), "allow_always");
        assert_eq!(PermissionDecision::AlwaysDeny.option_id(), "deny_always");
    }
}
