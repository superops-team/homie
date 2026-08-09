//! Command palette model for Homie.
//! Simplified port from diri-app/src/palette.rs.

/// Commands the palette can dispatch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PaletteCommand {
    SpawnShell,
    OpenQuickOpen,
    ToggleSidebar,
    OpenSettings,
    OpenFind,
    CheckForUpdates,
}

/// A single palette entry.
#[derive(Clone, Debug)]
pub struct PaletteAction {
    pub title: String,
    pub shortcut: Option<&'static str>,
    pub command: PaletteCommand,
    pub keywords: String,
}

/// A matched palette item for display.
#[derive(Clone, Debug)]
pub struct PaletteMatch {
    pub title: String,
    pub shortcut: Option<&'static str>,
    pub keywords: String,
    pub command: PaletteCommand,
}

/// The palette UI model.
#[derive(Clone, Debug)]
pub struct PaletteView {
    query: String,
    actions: Vec<PaletteAction>,
    filtered: Vec<PaletteMatch>,
    pub selected: usize,
}

impl PaletteView {
    pub fn new() -> Self {
        let actions = vec![
            PaletteAction {
                title: "New Terminal".into(),
                shortcut: Some("Cmd+T"),
                command: PaletteCommand::SpawnShell,
                keywords: "shell console zsh bash tty".into(),
            },
            PaletteAction {
                title: "Quick Open...".into(),
                shortcut: Some("Cmd+P"),
                command: PaletteCommand::OpenQuickOpen,
                keywords: "file open search goto".into(),
            },
            PaletteAction {
                title: "Toggle Sidebar".into(),
                shortcut: Some("Cmd+B"),
                command: PaletteCommand::ToggleSidebar,
                keywords: "panel sidebar hide show".into(),
            },
            PaletteAction {
                title: "Settings...".into(),
                shortcut: Some("Cmd+,"),
                command: PaletteCommand::OpenSettings,
                keywords: "preferences config settings".into(),
            },
            PaletteAction {
                title: "Find in Terminal".into(),
                shortcut: Some("Cmd+F"),
                command: PaletteCommand::OpenFind,
                keywords: "search find terminal output".into(),
            },
            PaletteAction {
                title: "Check for Updates".into(),
                shortcut: None,
                command: PaletteCommand::CheckForUpdates,
                keywords: "update version upgrade".into(),
            },
        ];
        let filtered = actions
            .iter()
            .map(|a| PaletteMatch {
                title: a.title.clone(),
                shortcut: a.shortcut,
                keywords: a.keywords.clone(),
                command: a.command.clone(),
            })
            .collect();
        Self {
            query: String::new(),
            actions,
            filtered,
            selected: 0,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }
    pub fn matches(&self) -> &[PaletteMatch] {
        &self.filtered
    }

    pub fn push_char(&mut self, ch: char) {
        self.query.push(ch);
        self.refilter();
    }

    pub fn pop_char(&mut self) {
        self.query.pop();
        self.refilter();
    }

    pub fn move_down(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = (self.selected + 1).min(self.filtered.len() - 1);
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn selected_command(&self) -> Option<PaletteCommand> {
        self.filtered.get(self.selected).map(|m| m.command.clone())
    }

    fn refilter(&mut self) {
        let q = self.query.to_lowercase();
        self.filtered = self
            .actions
            .iter()
            .filter(|a| {
                q.is_empty()
                    || a.title.to_lowercase().contains(&q)
                    || a.keywords.to_lowercase().contains(&q)
            })
            .map(|a| PaletteMatch {
                title: a.title.clone(),
                shortcut: a.shortcut,
                keywords: a.keywords.clone(),
                command: a.command.clone(),
            })
            .collect();
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }
}

impl Default for PaletteView {
    fn default() -> Self {
        Self::new()
    }
}
