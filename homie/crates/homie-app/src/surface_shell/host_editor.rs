use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum HostFormField {
    #[default]
    Name,
    Ssh,
    DefaultCwd,
    NodeEndpoint,
    NodeTokenFile,
    NodeId,
}

impl HostFormField {
    const ALL: [Self; 6] = [
        Self::Name,
        Self::Ssh,
        Self::DefaultCwd,
        Self::NodeEndpoint,
        Self::NodeTokenFile,
        Self::NodeId,
    ];

    pub(crate) fn adjacent(self, backwards: bool) -> Self {
        let index = Self::ALL
            .iter()
            .position(|field| *field == self)
            .unwrap_or(0);
        let delta = if backwards { Self::ALL.len() - 1 } else { 1 };
        Self::ALL[(index + delta) % Self::ALL.len()]
    }

    pub(crate) const fn debug_name(self) -> &'static str {
        match self {
            Self::Name => "NAME",
            Self::Ssh => "SSH",
            Self::DefaultCwd => "DEFAULT_CWD",
            Self::NodeEndpoint => "NODE_ENDPOINT",
            Self::NodeTokenFile => "NODE_TOKEN_FILE",
            Self::NodeId => "NODE_ID",
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Name => 0,
            Self::Ssh => 1,
            Self::DefaultCwd => 2,
            Self::NodeEndpoint => 3,
            Self::NodeTokenFile => 4,
            Self::NodeId => 5,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct HostEditor {
    pub(crate) original_id: Option<String>,
    pub(crate) name: QueryEditor,
    pub(crate) ssh: QueryEditor,
    pub(crate) default_cwd: QueryEditor,
    pub(crate) node_endpoint: QueryEditor,
    pub(crate) node_token_file: QueryEditor,
    pub(crate) node_id: QueryEditor,
    pub(crate) active_field: HostFormField,
    pub(crate) error: Option<String>,
    pub(crate) confirm_remove: bool,
}

impl HostEditor {
    pub(crate) fn adding() -> Self {
        Self::from_draft(HostDraft::new())
    }

    pub(crate) fn editing(host: &HostEntry) -> Self {
        Self::from_draft(HostDraft::editing(host))
    }

    fn from_draft(draft: HostDraft) -> Self {
        Self {
            original_id: draft.original_id,
            name: text_editor(&draft.name),
            ssh: text_editor(&draft.ssh),
            default_cwd: text_editor(&draft.default_cwd),
            node_endpoint: text_editor(&draft.node_endpoint),
            node_token_file: text_editor(&draft.node_token_file),
            node_id: text_editor(&draft.node_id),
            active_field: HostFormField::Name,
            error: None,
            confirm_remove: false,
        }
    }

    pub(crate) fn draft(&self) -> HostDraft {
        HostDraft {
            original_id: self.original_id.clone(),
            name: self.name.text().to_owned(),
            ssh: self.ssh.text().to_owned(),
            default_cwd: self.default_cwd.text().to_owned(),
            node_endpoint: self.node_endpoint.text().to_owned(),
            node_token_file: self.node_token_file.text().to_owned(),
            node_id: self.node_id.text().to_owned(),
        }
    }

    pub(crate) fn field_mut(&mut self) -> &mut QueryEditor {
        match self.active_field {
            HostFormField::Name => &mut self.name,
            HostFormField::Ssh => &mut self.ssh,
            HostFormField::DefaultCwd => &mut self.default_cwd,
            HostFormField::NodeEndpoint => &mut self.node_endpoint,
            HostFormField::NodeTokenFile => &mut self.node_token_file,
            HostFormField::NodeId => &mut self.node_id,
        }
    }

    pub(crate) fn field(&self, field: HostFormField) -> &QueryEditor {
        match field {
            HostFormField::Name => &self.name,
            HostFormField::Ssh => &self.ssh,
            HostFormField::DefaultCwd => &self.default_cwd,
            HostFormField::NodeEndpoint => &self.node_endpoint,
            HostFormField::NodeTokenFile => &self.node_token_file,
            HostFormField::NodeId => &self.node_id,
        }
    }
}

fn text_editor(value: &str) -> QueryEditor {
    let mut editor = QueryEditor::default();
    editor.insert(value);
    editor
}
