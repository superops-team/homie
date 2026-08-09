use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentCatalogError {
    #[error("invalid manifest JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("manifest {0:?} did not include an agent descriptor")]
    MissingAgent(Option<String>),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentManifest {
    #[serde(default)]
    pub id: String,
    pub display_name: String,
    pub short_label: String,
    #[serde(default = "default_glyph")]
    pub glyph: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub binary: Option<String>,
    #[serde(default)]
    pub spawn_args: Vec<String>,
    #[serde(default, rename = "sessionIDFlag")]
    pub session_id_flag: Option<String>,
    pub status_authority: StatusAuthority,
    #[serde(default)]
    pub first_class: bool,
    #[serde(default)]
    pub resume: Option<ResumeSpec>,
    #[serde(default)]
    pub return_to_login_shell: bool,
    #[serde(default)]
    pub approve: Option<AgentKeystroke>,
    #[serde(default = "default_deny")]
    pub deny: Option<AgentKeystroke>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub env_scrub_prefixes: Vec<String>,
    #[serde(default)]
    pub injection: AgentInjection,
    #[serde(default)]
    pub foreground_exec_names: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusAuthority {
    Process,
    Screen,
    Hooks,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ResumeStyle {
    Flag,
    FlagJoined,
    Subcommand,
    Latest,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeSpec {
    pub style: ResumeStyle,
    pub token: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentKeystroke {
    #[serde(default)]
    pub text: String,
    pub submit: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInjection {
    #[serde(default)]
    pub claude_hooks: bool,
    #[serde(default, rename = "claudeMCP")]
    pub claude_mcp: bool,
    #[serde(default)]
    pub codex_notify: bool,
    #[serde(default, rename = "codexMCP")]
    pub codex_mcp: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReadinessItem {
    pub id: String,
    pub binary: String,
    pub path: Option<String>,
    pub available: bool,
    pub descriptor: AgentManifest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReadinessResult {
    pub agents: Vec<AgentReadinessItem>,
}

impl AgentReadinessResult {
    pub fn agent(&self, id: &str) -> Option<&AgentReadinessItem> {
        self.agents.iter().find(|agent| agent.id == id)
    }
}

pub struct AgentCatalog {
    descriptors: BTreeMap<String, AgentManifest>,
    fallback: AgentManifest,
}

impl AgentCatalog {
    pub fn new(manifests: Vec<AgentManifest>) -> Self {
        let descriptors = manifests
            .into_iter()
            .map(|manifest| (manifest.id.clone(), manifest))
            .collect::<BTreeMap<_, _>>();
        Self {
            descriptors,
            fallback: AgentManifest::fallback("unknown-agent"),
        }
    }

    pub fn ordered(&self) -> Vec<&AgentManifest> {
        self.descriptors.values().collect()
    }

    pub fn descriptor(&self, id: &str) -> AgentManifest {
        self.descriptors
            .get(id)
            .cloned()
            .unwrap_or_else(|| AgentManifest::fallback(id))
    }

    pub fn resolve(&self, name: &str) -> Option<&AgentManifest> {
        let needle = name.to_ascii_lowercase();
        self.descriptors.get(&needle).or_else(|| {
            self.descriptors.values().find(|manifest| {
                manifest.short_label.eq_ignore_ascii_case(&needle)
                    || manifest
                        .aliases
                        .iter()
                        .any(|alias| alias.eq_ignore_ascii_case(&needle))
            })
        })
    }

    pub fn launchable(&self) -> Vec<&AgentManifest> {
        self.descriptors
            .values()
            .filter(|manifest| manifest.binary.is_some())
            .collect()
    }

    pub fn readiness_with_resolver<F>(&self, mut resolve: F) -> AgentReadinessResult
    where
        F: FnMut(&str) -> Option<String>,
    {
        AgentReadinessResult {
            agents: self
                .launchable()
                .into_iter()
                .filter_map(|descriptor| {
                    let binary = descriptor.binary.clone()?;
                    let path = resolve(&binary);
                    Some(AgentReadinessItem {
                        id: descriptor.id.clone(),
                        binary,
                        available: path.is_some(),
                        path,
                        descriptor: descriptor.clone(),
                    })
                })
                .collect(),
        }
    }

    pub fn env_scrub_prefixes(&self) -> Vec<String> {
        let mut seen = BTreeMap::<String, ()>::new();
        let mut prefixes = Vec::new();
        for descriptor in self.descriptors.values() {
            for prefix in &descriptor.env_scrub_prefixes {
                if seen.insert(prefix.clone(), ()).is_none() {
                    prefixes.push(prefix.clone());
                }
            }
        }
        prefixes
    }

    pub fn fallback(&self) -> &AgentManifest {
        &self.fallback
    }
}

impl AgentManifest {
    pub fn fallback(id: &str) -> Self {
        Self {
            id: id.to_string(),
            display_name: title_from_id(id),
            short_label: id.to_string(),
            glyph: default_glyph(),
            aliases: Vec::new(),
            binary: None,
            spawn_args: Vec::new(),
            session_id_flag: None,
            status_authority: StatusAuthority::Process,
            first_class: false,
            resume: None,
            return_to_login_shell: false,
            approve: None,
            deny: None,
            env: BTreeMap::new(),
            env_scrub_prefixes: Vec::new(),
            injection: AgentInjection::default(),
            foreground_exec_names: Vec::new(),
        }
    }

    pub fn can_resume(&self) -> bool {
        let Some(resume) = &self.resume else {
            return false;
        };
        if resume.style == ResumeStyle::Latest {
            return true;
        }
        self.session_id_flag.is_some() || self.injection.claude_hooks || self.injection.codex_notify
    }

    pub fn resume_argv(&self, id: &str) -> Option<Vec<String>> {
        if !self.can_resume() {
            return None;
        }
        let resume = self.resume.as_ref()?;
        let argv = match resume.style {
            ResumeStyle::Flag | ResumeStyle::Subcommand => {
                vec![resume.token.clone(), id.to_string()]
            }
            ResumeStyle::FlagJoined => vec![format!("{}={id}", resume.token)],
            ResumeStyle::Latest => vec![resume.token.clone()],
        };
        Some(argv)
    }
}

pub mod detect;
pub mod hooks;
pub mod status;

pub use detect::manifest::{Manifest, ManifestState, RegionKind, Rule, StatusModel};
pub use detect::{ManifestEngine, RiskHint, ScreenObservation, ScreenSnapshot, classify_risk};
pub use hooks::{
    HookEvent, NotifyEvent, ParsedHook, ParsedNotify, parse_claude_hook, parse_codex_notify,
};
pub use status::{
    Authority, ClaudeHook, ReducerOutcome, ReducerTiming, StatusReducer, StatusSignal,
};

pub fn load_manifest(bytes: &[u8]) -> Result<AgentManifest, AgentCatalogError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ManifestHead {
        id: String,
        agent: Option<AgentManifest>,
    }

    let head: ManifestHead = serde_json::from_slice(bytes)?;
    let Some(mut agent) = head.agent else {
        return Err(AgentCatalogError::MissingAgent(Some(head.id)));
    };
    agent.id = head.id;
    if agent.short_label.is_empty() {
        agent.short_label = agent.id.clone();
    }
    Ok(agent)
}

fn default_glyph() -> String {
    "▸".to_string()
}

fn default_deny() -> Option<AgentKeystroke> {
    Some(AgentKeystroke {
        text: "\u{1b}".to_string(),
        submit: false,
    })
}

fn title_from_id(id: &str) -> String {
    id.split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
