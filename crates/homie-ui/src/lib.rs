use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Radius;

impl Radius {
    pub const CHIP: f32 = 5.0;
    pub const BADGE: f32 = 6.0;
    pub const ROW: f32 = 7.0;
    pub const CARD: f32 = 10.0;
    pub const PANEL: f32 = 12.0;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics;

impl Metrics {
    pub const TITLE_BAR: f32 = 42.0;
    pub const TOOLBAR_EDGE_INSET: f32 = 12.0;
    pub const TOOLBAR_TRAFFIC_LIGHT_LANE: f32 = 66.0;
    pub const TOOLBAR_ITEM_GAP: f32 = 8.0;
    pub const TOOLBAR_COMPACT_GAP: f32 = 4.0;
    pub const TOOLBAR_CONTROL_SIZE: f32 = 26.0;
    pub const TOOLBAR_CHIP_HEIGHT: f32 = 24.0;
    pub const ROW_HEIGHT: f32 = 28.0;
    pub const NEW_AGENT_FOOTER: f32 = 32.0;
    pub const TRAFFIC_LIGHT_X_OFFSET: f32 = 12.0;
    pub const TRAFFIC_LIGHT_Y_OFFSET: f32 = 6.0;
    pub const SIDEBAR_DEFAULT_WIDTH: f32 = 248.0;
    pub const SIDEBAR_MIN_WIDTH: f32 = 200.0;
    pub const SIDEBAR_MAX_WIDTH: f32 = 400.0;
    pub const MIN_WINDOW_WIDTH: f32 = 900.0;
    pub const MIN_WINDOW_HEIGHT: f32 = 560.0;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextRole {
    Meta,
    SectionHeader,
    Row,
    RowEmphasized,
    Title,
    DisplayTitle,
    MetaMono,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontWeightToken {
    Normal,
    Medium,
    Semibold,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeStyle {
    pub size: f32,
    pub weight: FontWeightToken,
    pub monospaced: bool,
}

impl TypeStyle {
    pub const fn new(size: f32, weight: FontWeightToken, monospaced: bool) -> Self {
        Self {
            size,
            weight,
            monospaced,
        }
    }
}

pub struct Typo;

impl Typo {
    pub const META: TypeStyle = TypeStyle::new(11.0, FontWeightToken::Medium, false);
    pub const SECTION_HEADER: TypeStyle = TypeStyle::new(11.0, FontWeightToken::Semibold, false);
    pub const ROW: TypeStyle = TypeStyle::new(13.0, FontWeightToken::Normal, false);
    pub const ROW_EMPHASIZED: TypeStyle = TypeStyle::new(13.0, FontWeightToken::Medium, false);
    pub const TITLE: TypeStyle = TypeStyle::new(13.0, FontWeightToken::Semibold, false);
    pub const DISPLAY_TITLE: TypeStyle = TypeStyle::new(15.0, FontWeightToken::Semibold, false);
    pub const META_MONO: TypeStyle = TypeStyle::new(11.0, FontWeightToken::Medium, true);

    pub const ALL: [(TextRole, TypeStyle); 7] = [
        (TextRole::Meta, Self::META),
        (TextRole::SectionHeader, Self::SECTION_HEADER),
        (TextRole::Row, Self::ROW),
        (TextRole::RowEmphasized, Self::ROW_EMPHASIZED),
        (TextRole::Title, Self::TITLE),
        (TextRole::DisplayTitle, Self::DISPLAY_TITLE),
        (TextRole::MetaMono, Self::META_MONO),
    ];
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgbaToken {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl RgbaToken {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Appearance {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextTone {
    Selected,
    Unselected,
    Label,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SemanticColors {
    pub appearance: Appearance,
    pub primary: RgbaToken,
    pub secondary: RgbaToken,
    pub tertiary: RgbaToken,
    pub background: RgbaToken,
}

impl SemanticColors {
    pub const fn light() -> Self {
        Self {
            appearance: Appearance::Light,
            primary: RgbaToken::new(0.0, 0.0, 0.0, 1.0),
            secondary: RgbaToken::new(0.0, 0.0, 0.0, 0.60),
            tertiary: RgbaToken::new(0.0, 0.0, 0.0, 0.30),
            background: RgbaToken::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    pub const fn dark() -> Self {
        Self {
            appearance: Appearance::Dark,
            primary: RgbaToken::new(1.0, 1.0, 1.0, 1.0),
            secondary: RgbaToken::new(1.0, 1.0, 1.0, 0.60),
            tertiary: RgbaToken::new(1.0, 1.0, 1.0, 0.30),
            background: RgbaToken::new(0.071, 0.075, 0.094, 1.0),
        }
    }

    pub const fn new(appearance: Appearance) -> Self {
        match appearance {
            Appearance::Light => Self::light(),
            Appearance::Dark => Self::dark(),
        }
    }

    pub const fn sidebar(appearance: Appearance) -> Self {
        match appearance {
            Appearance::Light => Self {
                secondary: RgbaToken::new(0.0, 0.0, 0.0, 0.68),
                tertiary: RgbaToken::new(0.0, 0.0, 0.0, 0.42),
                ..Self::light()
            },
            Appearance::Dark => Self {
                secondary: RgbaToken::new(1.0, 1.0, 1.0, 0.70),
                tertiary: RgbaToken::new(1.0, 1.0, 1.0, 0.44),
                ..Self::dark()
            },
        }
    }

    pub const fn text(self, tone: TextTone) -> RgbaToken {
        let alpha = match tone {
            TextTone::Selected => 1.0,
            TextTone::Unselected => 0.75,
            TextTone::Label => 0.85,
        };
        self.primary.alpha(alpha)
    }

    pub const fn floating_stroke(self) -> RgbaToken {
        match self.appearance {
            Appearance::Dark => RgbaToken::new(1.0, 1.0, 1.0, 0.08),
            Appearance::Light => RgbaToken::new(0.0, 0.0, 0.0, 0.10),
        }
    }

    pub const fn floating_surface(self) -> RgbaToken {
        match self.appearance {
            Appearance::Dark => RgbaToken::new(0.141, 0.161, 0.196, 1.0),
            Appearance::Light => RgbaToken::new(0.949, 0.953, 0.941, 1.0),
        }
    }

    pub const fn sidebar_surface(self) -> RgbaToken {
        match self.appearance {
            Appearance::Dark => RgbaToken::new(0.141, 0.161, 0.196, 0.89),
            Appearance::Light => RgbaToken::new(0.949, 0.953, 0.941, 0.89),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub response: f32,
    pub damping_fraction: f32,
}

impl Spring {
    pub const fn new(response: f32, damping_fraction: f32) -> Self {
        Self {
            response,
            damping_fraction,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion;

impl Motion {
    pub const SNAP: Spring = Spring::new(0.32, 0.74);
    pub const POP: Spring = Spring::new(0.40, 0.60);
    pub const SETTLE: Spring = Spring::new(0.55, 0.82);
    pub const FOOTER_PIN: Spring = Spring::new(0.32, 0.82);
    pub const ROW_SELECT_SECONDS: f32 = 0.16;
    pub const OVERLAY_FADE_SECONDS: f32 = 0.12;
    pub const SEAM_SLIDE_MS: u64 = 260;
    pub const BREATHE_SECONDS: f64 = 2.6;
    pub const SWEEP_SECONDS: f64 = 2.4;
    pub const PULSE_SECONDS: f64 = 1.8;
    pub const PING_PERIOD_RISK: f64 = 1.2;
    pub const SHELL_BLINK: f64 = 1.6;
    pub const TICK_HZ: u64 = 10;
}

pub struct Fill;

impl Fill {
    pub const HOVER_OPACITY: f32 = 0.06;
    pub const MULTI_SELECTED_OPACITY: f32 = 0.08;
    pub const SELECTED_OPACITY: f32 = 0.10;
    pub const SUBTLE_OPACITY: f32 = 0.06;
}

pub struct Space;

impl Space {
    pub const INDENT: f32 = 12.0;
    pub const ROW_H: f32 = 8.0;
    pub const INSET: f32 = 10.0;
}

pub struct MemoryFormat;

impl MemoryFormat {
    pub const SOFT_BYTES: u64 = 2 * 1_073_741_824;
    pub const LOUD_BYTES: u64 = 6 * 1_073_741_824;

    #[must_use]
    pub fn gb(bytes: u64) -> String {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    }

    #[must_use]
    pub fn badge(bytes: Option<u64>) -> Option<String> {
        bytes
            .filter(|bytes| *bytes > Self::SOFT_BYTES)
            .map(Self::gb)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BrandMark {
    pub wordmark: &'static str,
    pub monogram: &'static str,
    pub bundle_id: &'static str,
}

pub const HOMIE_BRAND: BrandMark = BrandMark {
    wordmark: "Homie",
    monogram: "H",
    bundle_id: "com.superops.homie",
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AgentKind {
    ClaudeCode,
    Codex,
    OpenCode,
    Gemini,
    Cursor,
    Shell,
    Generic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum StatusState {
    Starting,
    Working,
    NeedsInput { destructive: bool },
    DoneUnseen,
    Idle,
    Hibernated,
    Exited,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationPhase {
    pub breathe: f32,
    pub sweep_turns: f32,
    pub pulse: f32,
}

impl AnimationPhase {
    #[must_use]
    pub fn at(seconds: f64) -> Self {
        let wave = |period: f64| (seconds * std::f64::consts::TAU / period).sin() as f32;
        Self {
            breathe: 1.0 + 0.055 * wave(Motion::BREATHE_SECONDS),
            sweep_turns: (seconds / Motion::SWEEP_SECONDS) as f32,
            pulse: 0.5 + 0.5 * wave(Motion::PULSE_SECONDS),
        }
    }
}

#[must_use]
pub fn status_color_name(kind: AgentKind, state: StatusState) -> &'static str {
    match state {
        StatusState::NeedsInput { destructive: true } => "danger",
        StatusState::NeedsInput { destructive: false } => "attention",
        StatusState::DoneUnseen => "fresh",
        StatusState::Working | StatusState::Starting => match kind {
            AgentKind::ClaudeCode => "clay",
            AgentKind::Gemini => "gemini_blue",
            AgentKind::Shell | AgentKind::Generic => "generic_working",
            AgentKind::Codex | AgentKind::Cursor | AgentKind::OpenCode => "primary_working",
        },
        StatusState::Idle => "secondary",
        StatusState::Hibernated => "tertiary",
        StatusState::Exited => "muted",
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StatusGlyph {
    pub name: &'static str,
    pub symbol: &'static str,
    pub label: &'static str,
    pub tone: &'static str,
}

pub const STATUS_GLYPHS: &[StatusGlyph] = &[
    StatusGlyph {
        name: "working",
        symbol: "●",
        label: "Working",
        tone: "primary_working",
    },
    StatusGlyph {
        name: "attention",
        symbol: "!",
        label: "Needs input",
        tone: "attention",
    },
    StatusGlyph {
        name: "danger",
        symbol: "!",
        label: "Destructive approval",
        tone: "danger",
    },
    StatusGlyph {
        name: "idle",
        symbol: "○",
        label: "Idle",
        tone: "secondary",
    },
    StatusGlyph {
        name: "hibernated",
        symbol: "◌",
        label: "Hibernated",
        tone: "tertiary",
    },
    StatusGlyph {
        name: "exited",
        symbol: "×",
        label: "Exited",
        tone: "muted",
    },
];

#[must_use]
pub fn status_glyph(name: &str) -> Option<StatusGlyph> {
    STATUS_GLYPHS
        .iter()
        .copied()
        .find(|glyph| glyph.name == name)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesignGalleryEntry {
    pub id: &'static str,
    pub title: &'static str,
    pub surface: &'static str,
}

pub const DESIGN_GALLERY: &[DesignGalleryEntry] = &[
    DesignGalleryEntry {
        id: "workbench",
        title: "Workbench",
        surface: "sidebar-terminal-inspector",
    },
    DesignGalleryEntry {
        id: "settings",
        title: "Settings",
        surface: "floating-panel",
    },
    DesignGalleryEntry {
        id: "notifications",
        title: "Notifications",
        surface: "status-rollup",
    },
    DesignGalleryEntry {
        id: "quick-open",
        title: "Quick Open",
        surface: "command-surface",
    },
];

#[derive(Clone, Debug, PartialEq)]
pub struct SidebarState {
    pub visible: bool,
    pub width: f32,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            visible: true,
            width: Metrics::SIDEBAR_DEFAULT_WIDTH,
        }
    }
}

impl SidebarState {
    pub fn set_width(&mut self, width: f32) {
        self.width = width.clamp(Metrics::SIDEBAR_MIN_WIDTH, Metrics::SIDEBAR_MAX_WIDTH);
    }

    pub fn reset_width(&mut self) {
        self.width = Metrics::SIDEBAR_DEFAULT_WIDTH;
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidebarSessionRow {
    pub id: String,
    pub title: String,
    pub status: String,
    pub pinned: bool,
    pub archived: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SidebarSessionModel {
    pub rows: Vec<SidebarSessionRow>,
    pub selected: Option<String>,
    pub multi_selected: Vec<String>,
}

impl SidebarSessionModel {
    #[must_use]
    pub fn new(rows: Vec<SidebarSessionRow>) -> Self {
        Self {
            rows,
            selected: None,
            multi_selected: Vec::new(),
        }
    }

    pub fn select(&mut self, id: &str) {
        if self.rows.iter().any(|row| row.id == id && !row.archived) {
            self.selected = Some(id.to_string());
            self.multi_selected.clear();
        }
    }

    pub fn toggle_multi_select(&mut self, id: &str) {
        if !self.rows.iter().any(|row| row.id == id && !row.archived) {
            return;
        }
        if let Some(index) = self
            .multi_selected
            .iter()
            .position(|selected| selected == id)
        {
            self.multi_selected.remove(index);
        } else {
            self.multi_selected.push(id.to_string());
            self.multi_selected.sort();
        }
    }

    pub fn rename(&mut self, id: &str, title: impl Into<String>) {
        if let Some(row) = self.rows.iter_mut().find(|row| row.id == id) {
            row.title = title.into();
        }
    }

    pub fn toggle_pin(&mut self, id: &str) {
        if let Some(row) = self.rows.iter_mut().find(|row| row.id == id) {
            row.pinned = !row.pinned;
        }
        self.sort_rows();
    }

    pub fn archive(&mut self, id: &str) {
        if let Some(row) = self.rows.iter_mut().find(|row| row.id == id) {
            row.archived = true;
        }
        self.multi_selected.retain(|selected| selected != id);
        if self.selected.as_deref() == Some(id) {
            self.selected = None;
        }
    }

    pub fn move_before(&mut self, moved: &str, target: &str) {
        if moved == target {
            return;
        }
        let Some(index) = self.rows.iter().position(|row| row.id == moved) else {
            return;
        };
        let row = self.rows.remove(index);
        let target_index = self
            .rows
            .iter()
            .position(|candidate| candidate.id == target)
            .unwrap_or(self.rows.len());
        self.rows.insert(target_index, row);
    }

    pub fn move_to_end(&mut self, moved: &str) {
        let Some(index) = self.rows.iter().position(|row| row.id == moved) else {
            return;
        };
        let row = self.rows.remove(index);
        self.rows.push(row);
    }

    fn sort_rows(&mut self) {
        self.rows.sort_by(|left, right| {
            right
                .pinned
                .cmp(&left.pinned)
                .then_with(|| left.title.cmp(&right.title))
        });
    }
}

#[must_use]
pub fn status_glyph_name(status: &str) -> &'static str {
    match status {
        "needs_input" => "attention",
        "running" | "working" | "starting" => "working",
        "idle" => "idle",
        "hibernated" => "hibernated",
        "exited" => "exited",
        "archived" => "archived",
        _ => "unknown",
    }
}

pub fn move_before<T: Clone + PartialEq>(order: &mut Vec<T>, moved: &T, target: &T) {
    if moved == target {
        return;
    }
    let Some(index) = order.iter().position(|item| item == moved) else {
        return;
    };
    let item = order.remove(index);
    let target_index = order
        .iter()
        .position(|candidate| candidate == target)
        .unwrap_or(order.len());
    order.insert(target_index, item);
}

pub fn move_to_end<T: Clone + PartialEq>(order: &mut Vec<T>, moved: &T) {
    let Some(index) = order.iter().position(|item| item == moved) else {
        return;
    };
    let item = order.remove(index);
    order.push(item);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NotificationSeverity {
    Info,
    Success,
    Attention,
    Critical,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NotificationActionKind {
    Approve,
    Deny,
    OpenSession,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationAction {
    pub kind: NotificationActionKind,
    pub label: String,
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationItem {
    pub severity: NotificationSeverity,
    pub title: String,
    pub body: String,
    pub session_id: Option<String>,
    pub status: String,
    pub actions: Vec<NotificationAction>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationSession {
    pub id: String,
    pub title: String,
    pub status: String,
    pub needs_input: bool,
    pub destructive: bool,
    pub agent_has_approve_deny: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationRollup {
    pub total: usize,
    pub needs_input: usize,
    pub running: usize,
    pub exited: usize,
    pub items: Vec<NotificationItem>,
}

impl NotificationRollup {
    #[must_use]
    pub fn badge(&self) -> String {
        if self.needs_input > 0 {
            format!("{} need input", self.needs_input)
        } else if self.running > 0 {
            format!("{} running", self.running)
        } else {
            format!("{} sessions", self.total)
        }
    }
}

#[must_use]
pub fn notification_rollup(sessions: &[NotificationSession]) -> NotificationRollup {
    let mut rollup = NotificationRollup {
        total: sessions.len(),
        needs_input: 0,
        running: 0,
        exited: 0,
        items: Vec::new(),
    };
    for session in sessions {
        match session.status.as_str() {
            "running" | "working" | "starting" => rollup.running += 1,
            "exited" => rollup.exited += 1,
            _ => {}
        }
        if session.needs_input {
            rollup.needs_input += 1;
        }
        rollup.items.push(notification_item(session));
    }
    rollup
}

fn notification_item(session: &NotificationSession) -> NotificationItem {
    let severity = if session.needs_input && session.destructive {
        NotificationSeverity::Critical
    } else if session.needs_input {
        NotificationSeverity::Attention
    } else if session.status == "exited" {
        NotificationSeverity::Success
    } else {
        NotificationSeverity::Info
    };
    let mut actions = vec![NotificationAction {
        kind: NotificationActionKind::OpenSession,
        label: "Open".to_string(),
        session_id: Some(session.id.clone()),
    }];
    if session.needs_input && session.agent_has_approve_deny {
        actions.push(NotificationAction {
            kind: NotificationActionKind::Approve,
            label: "Approve".to_string(),
            session_id: Some(session.id.clone()),
        });
        actions.push(NotificationAction {
            kind: NotificationActionKind::Deny,
            label: "Deny".to_string(),
            session_id: Some(session.id.clone()),
        });
    }
    NotificationItem {
        severity,
        title: session.title.clone(),
        body: if session.needs_input {
            "Agent needs your input".to_string()
        } else {
            format!("Session is {}", session.status)
        },
        session_id: Some(session.id.clone()),
        status: session.status.clone(),
        actions,
    }
}

#[must_use]
pub fn macos_notification_command(item: &NotificationItem) -> Vec<String> {
    vec![
        "/usr/bin/osascript".to_string(),
        "-e".to_string(),
        format!(
            "display notification {} with title {} subtitle {}",
            applescript_string(&redact_notification_text(&item.body)),
            applescript_string("Homie"),
            applescript_string(&item.title),
        ),
    ]
}

#[must_use]
pub fn redact_notification_text(text: &str) -> String {
    let mut redacted = text.to_string();
    for marker in [
        "Authorization:",
        "authorization:",
        "Bearer ",
        "token=",
        "cookie=",
    ] {
        if let Some(index) = redacted.find(marker) {
            redacted.truncate(index + marker.len());
            redacted.push_str("[redacted]");
        }
    }
    redacted
}

fn applescript_string(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaneHeights {
    pub primary: f32,
    pub auxiliary: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkbenchLayout {
    primary_fraction: f32,
}

impl Default for WorkbenchLayout {
    fn default() -> Self {
        Self {
            primary_fraction: 0.62,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankedItem {
    pub label: String,
    pub score: i64,
}

pub fn fuzzy_score(query: &str, candidate: &str) -> Option<i64> {
    if query.is_empty() {
        return Some(0);
    }
    let query_lower = query.to_ascii_lowercase();
    let candidate_lower = candidate.to_ascii_lowercase();
    let mut score = 0_i64;
    let mut search_start = 0;
    if candidate_lower.contains(&query_lower) {
        score += 20;
    }
    if acronym_matches(&query_lower, &candidate_lower) {
        score += 30;
    }
    for ch in query_lower.chars() {
        let found = candidate_lower[search_start..].find(ch)?;
        let absolute = search_start + found;
        score += if absolute == 0 {
            12
        } else if candidate_lower.as_bytes()[absolute - 1].is_ascii_whitespace()
            || matches!(candidate_lower.as_bytes()[absolute - 1], b'-' | b'_' | b'/')
        {
            8
        } else {
            1
        };
        search_start = absolute + ch.len_utf8();
    }
    Some(score)
}

fn acronym_matches(query: &str, candidate: &str) -> bool {
    let initials = candidate
        .split(|ch: char| ch.is_ascii_whitespace() || matches!(ch, '-' | '_' | '/'))
        .filter_map(|word| word.chars().next())
        .collect::<String>();
    initials.starts_with(query)
}

pub fn rank_items<'a>(
    query: &str,
    candidates: impl IntoIterator<Item = &'a str>,
) -> Vec<RankedItem> {
    let mut ranked = candidates
        .into_iter()
        .filter_map(|candidate| {
            fuzzy_score(query, candidate).map(|score| RankedItem {
                label: candidate.to_string(),
                score,
            })
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.label.cmp(&right.label))
    });
    ranked
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryEntry {
    pub id: String,
    pub agent_kind: String,
    pub cwd: String,
    pub title: Option<String>,
    pub transcript_path: String,
    pub cwd_exists: bool,
}

impl HistoryEntry {
    #[must_use]
    pub fn can_resume(&self) -> bool {
        self.cwd_exists && !self.transcript_path.trim().is_empty()
    }
}

impl WorkbenchLayout {
    pub fn from_fraction(primary_fraction: f32) -> Self {
        Self {
            primary_fraction: primary_fraction.clamp(0.0, 1.0),
        }
    }

    #[must_use]
    pub fn primary_fraction(self) -> f32 {
        self.primary_fraction
    }

    #[must_use]
    pub fn pane_heights(self, available_height: f32) -> PaneHeights {
        let available_height = available_height.max(0.0);
        let primary = available_height * self.primary_fraction;
        PaneHeights {
            primary,
            auxiliary: available_height - primary,
        }
    }
}
