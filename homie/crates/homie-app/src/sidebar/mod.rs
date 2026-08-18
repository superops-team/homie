//! Window-sidebar state, deterministic preview data, and GPUI rendering.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use gpui::{
    Anchor, AnyElement, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle,
    Focusable, FontWeight, Hsla, IntoElement, MouseButton, Pixels, Point, Render, Rgba,
    ScrollHandle, SharedString, Task, Window, anchored, deferred, div, linear_color_stop,
    linear_gradient, point, prelude::*, px,
};
use homie_proto::remote_pty::PersistenceCapability;
use homie_proto::{
    AgentKind as ProtoAgentKind, AttentionLevel as ProtoAttentionLevel, ProjectId, SessionId,
    SessionRecord,
};
use homie_ui::{
    AgentKind, AgentLogo, AttentionDot, AttentionLevel, Fill, FloatingSurface, HairlineDivider,
    HoverMarquee, Ink, LoadingIndicator, Metrics, Radius, RowFill, SemanticColors, Space,
    StatusGlyph, StatusState, Typo,
};
use tokio::sync::mpsc;

use crate::macos::sf_symbols::{SymbolWeight, sf_symbol, sf_symbol_weighted};
use crate::navigation::query_label;
use crate::query_editor::{self, ClipboardEdit, Edit};
use crate::seam::toggle_has_settled;
use crate::store::{
    ClickModifiers, DirectoryListingState, SessionStore, SpawnOptions, StoreEffect, StoreRuntime,
};
use crate::updates::{UpdateCommand, UpdatePhase, UpdateState};
use crate::usage::{UsageFormat, UsageSnapshot};

use picker_logic::{agent_picker_shortcut, remote_picker_target, should_resolve_active_repo};
use projection::*;
use render_helpers::*;

mod commands;
mod fixture;
mod picker_logic;
mod popover;
mod projection;
mod render_helpers;
mod sections;
mod state;
mod view;

pub use fixture::{PreviewScenario, SidebarPreviewFixture};
pub use state::{DragItem, Popover, SidebarUiState, move_before, move_to_end};
pub(crate) use view::{DragPreview, DraggedSidebarItem};
pub use view::{Sidebar, SidebarEvent};

#[cfg(test)]
mod tests;

const PREVIEW_USAGE: f64 = 4.82;
