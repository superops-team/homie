//! The spawn graph read from `SessionRecord.parent`, and how a target session
//! relates to the caller.

use homie_proto::{SessionId, SessionRecord};

pub(crate) fn relation_word(relation: Relation) -> &'static str {
    match relation {
        Relation::Caller => "self",
        Relation::Parent => "parent",
        Relation::Child => "child",
        Relation::Ancestor => "ancestor",
        Relation::Descendant => "descendant",
        Relation::Sibling => "sibling",
        Relation::Unrelated => "unrelated",
    }
}

/// The spawn graph, read from `SessionRecord.parent`. Reads across the graph
/// are open; writes to a session that is not your parent or your own child get
/// a provenance header so the receiving agent can tell it apart from its user.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Relation {
    Caller,
    Parent,
    Child,
    Ancestor,
    Descendant,
    Sibling,
    Unrelated,
}

impl Relation {
    /// Your parent and your direct children are the delegation channel; both
    /// ends already know who the other is, so extra framing only confuses an
    /// agent mid-task.
    pub(crate) fn delivers_verbatim(self) -> bool {
        matches!(self, Relation::Parent | Relation::Child)
    }
}

pub(crate) struct Lineage {
    records: Vec<SessionRecord>,
    caller: Option<SessionId>,
}

impl Lineage {
    pub(crate) fn new(records: Vec<SessionRecord>, caller: Option<SessionId>) -> Self {
        Self { records, caller }
    }

    pub(crate) fn record(&self, id: &SessionId) -> Option<&SessionRecord> {
        self.records.iter().find(|r| &r.id == id)
    }

    pub(crate) fn children_of(&self, id: &SessionId) -> Vec<&SessionRecord> {
        self.records
            .iter()
            .filter(|r| r.parent.as_ref() == Some(id))
            .collect()
    }

    /// Breadth-first descendants with a visited set so a corrupted or
    /// hand-edited state file that describes a cycle degrades to a short
    /// answer instead of hanging the daemon call.
    pub(crate) fn descendants_of(&self, id: &SessionId) -> Vec<&SessionRecord> {
        let mut seen: std::collections::HashSet<&SessionId> = std::collections::HashSet::new();
        seen.insert(id);
        let mut queue = self.children_of(id);
        let mut out = Vec::new();
        while let Some(next) = queue.first().copied() {
            queue.remove(0);
            if !seen.insert(&next.id) {
                continue;
            }
            out.push(next);
            queue.extend(self.children_of(&next.id));
        }
        out
    }

    /// Walk to the root, nearest ancestor first, with the same cycle guard.
    pub(crate) fn ancestors_of(&self, id: &SessionId) -> Vec<&SessionRecord> {
        let mut seen: std::collections::HashSet<&SessionId> = std::collections::HashSet::new();
        seen.insert(id);
        let mut out = Vec::new();
        let mut cursor = self.record(id).and_then(|r| r.parent.as_ref());
        while let Some(current) = cursor {
            if !seen.insert(current) {
                break;
            }
            let Some(record) = self.record(current) else {
                break;
            };
            out.push(record);
            cursor = record.parent.as_ref();
        }
        out
    }

    pub(crate) fn relation_to(&self, target: &SessionId) -> Relation {
        let Some(caller) = &self.caller else {
            return Relation::Unrelated;
        };
        if caller == target {
            return Relation::Caller;
        }
        if self.record(caller).and_then(|r| r.parent.as_ref()) == Some(target) {
            return Relation::Parent;
        }
        if self.record(target).and_then(|r| r.parent.as_ref()) == Some(caller) {
            return Relation::Child;
        }
        if self.ancestors_of(caller).iter().any(|r| &r.id == target) {
            return Relation::Ancestor;
        }
        if self.descendants_of(caller).iter().any(|r| &r.id == target) {
            return Relation::Descendant;
        }
        let mine = self.record(caller).and_then(|r| r.parent.as_ref());
        let theirs = self.record(target).and_then(|r| r.parent.as_ref());
        if mine.is_some() && mine == theirs {
            return Relation::Sibling;
        }
        Relation::Unrelated
    }

    /// Attribution for a cross-session write. Verbatim for the delegation
    /// channel or when there is no caller; otherwise prefixes one line naming
    /// the sender.
    pub(crate) fn frame(&self, text: &str, relation: Relation) -> String {
        if relation.delivers_verbatim() {
            return text.to_string();
        }
        let Some(caller) = &self.caller else {
            return text.to_string();
        };
        let who = self
            .record(caller)
            .map(|r| format!("id:{} ({})", r.id.0, r.title))
            .unwrap_or_else(|| format!("id:{}", caller.0));
        format!(
            "[message from {who}, channel: homie — reply with send_prompt to that id]\n\n{text}"
        )
    }
}
