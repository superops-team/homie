use std::collections::HashSet;
use std::sync::Arc;

use homie_proto::{
    AgentKind, AttentionLevel, DateMillis, ExitInfo, ExitReason, Project, ProjectId, Resumability,
    SessionId, SessionListResult, SessionRecord, SessionStatus, TitleSource,
};
use tempfile::tempdir;
use tokio::sync::mpsc;

use crate::notifications::NotificationSound;

use super::{
    ClickModifiers, DefaultAgent, EventEnvelope, InspectorTab, Prefs, SessionStore,
    SidebarProjection, StoreEffect, StoreEventChange, TerminalResidency, WindowMode,
    WindowPlacement, event_publication_policy,
};
use crate::switcher::{OverviewFilter, OverviewLane, SwitcherKey};

fn id(value: &str) -> SessionId {
    SessionId::new(value)
}

fn pid(value: &str) -> ProjectId {
    ProjectId::new(value)
}

fn session(value: &str, project: &str, created: f64) -> SessionRecord {
    SessionRecord {
        id: id(value),
        kind: AgentKind::CLAUDE_CODE,
        cwd: format!("/work/{project}"),
        project_id: pid(project),
        worktree_path: None,
        git_branch: None,
        title: value.to_owned(),
        title_source: TitleSource::Placeholder,
        agent_session_id: None,
        transcript_path: None,
        status: SessionStatus::Idle,
        needs_input: None,
        resumability: Resumability::Live,
        parent: None,
        created_at: DateMillis(created),
        updated_at: DateMillis(created),
        last_turn_completed_at: None,
        last_seen_at: None,
        pinned: false,
        archived_at: None,
        host: None,
        remote_persistence: None,
        hibernation: None,
        memory_bytes: None,
        artifacts: None,
        pull_requests: None,
        listening_ports: None,
        foreground_agent: None,
    }
}

fn project(value: &str, name: &str) -> Project {
    Project {
        id: pid(value),
        root: format!("/work/{value}"),
        name: name.to_owned(),
        pinned_order: None,
    }
}

fn hydrated(
    sessions: Vec<SessionRecord>,
    projects: Vec<Project>,
    prefs: Prefs,
) -> (SessionStore, mpsc::UnboundedReceiver<StoreEffect>) {
    let (mut store, effects) = SessionStore::headless(prefs);
    store.hydrate(SessionListResult { sessions, projects });
    (store, effects)
}

fn drain(effects: &mut mpsc::UnboundedReceiver<StoreEffect>) -> Vec<StoreEffect> {
    let mut drained = Vec::new();
    while let Ok(effect) = effects.try_recv() {
        drained.push(effect);
    }
    drained
}

mod attention;
mod events;
mod hosts;
mod ordering;
mod runtime;
mod sessions;
mod switcher;
