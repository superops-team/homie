//! Screen-driven status detection.
//!
//! An agent's status — working, idle, waiting on you — is inferred from what
//! it painted on its terminal, using per-agent rules that live in JSON
//! manifests. Ported from diri-engine.

pub mod manifest;
pub mod redact;
pub mod regions;

use std::collections::HashMap;
use std::path::Path;

use crate::detect::manifest::{ManifestState, PredicateContext, RegionKind, Rule};
use crate::detect::redact::redact;

pub use self::manifest::{Manifest, RegionKind as ManifestRegionKind};

/// What the emulator saw: the plain-text grid plus the OSC state.
#[derive(Clone, Debug, Default)]
pub struct ScreenSnapshot {
    pub lines: Vec<String>,
    pub osc_title: Option<String>,
    pub osc_progress_state: Option<i64>,
    /// Bumps whenever the visible content changes.
    pub content_seq: u64,
}

impl ScreenSnapshot {
    pub fn from_lines<I, S>(lines: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            lines: lines.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

/// The engine's verdict for one snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenObservation {
    pub state: ManifestState,
    pub matched_rule_id: String,
    pub priority: i64,
    pub content_seq: u64,
    pub prompt_excerpt: Option<String>,
    pub options: Option<Vec<String>>,
}

/// Immutable manifest storage, built once and shared across sessions.
pub struct ManifestEngine {
    manifests: HashMap<String, Manifest>,
}

impl ManifestEngine {
    pub fn new(manifests: Vec<Manifest>) -> Self {
        Self {
            manifests: manifests
                .into_iter()
                .map(|manifest| (manifest.id.clone(), manifest))
                .collect(),
        }
    }

    /// Loads every `*.json` in `dir`, later ids replacing earlier ones.
    ///
    /// Decoding is best-effort per file: one broken override must not take out
    /// detection for every other agent. The count of failed files is returned.
    pub fn load_dir(dir: &Path) -> std::io::Result<(Self, Vec<String>)> {
        let mut manifests = Vec::new();
        let mut failed = Vec::new();
        let mut entries: Vec<_> = std::fs::read_dir(dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect();
        entries.sort();

        for path in entries {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            match std::fs::read(&path)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<Manifest>(&bytes).ok())
            {
                Some(manifest) => manifests.push(manifest),
                None => failed.push(name),
            }
        }
        Ok((Self::new(manifests), failed))
    }

    pub fn manifest(&self, id: &str) -> Option<&Manifest> {
        self.manifests.get(id)
    }

    pub fn ids(&self) -> Vec<&str> {
        self.manifests.keys().map(String::as_str).collect()
    }

    /// Evaluates `snapshot` against the manifest for `manifest_id`.
    ///
    /// Rules are pre-sorted by descending priority, so the first match wins.
    pub fn evaluate(
        &self,
        snapshot: &ScreenSnapshot,
        manifest_id: &str,
    ) -> Option<ScreenObservation> {
        let manifest = self.manifests.get(manifest_id)?;

        let mut cache: HashMap<(RegionKind, usize), (Vec<String>, String)> = HashMap::new();

        let mut winner: Option<&Rule> = None;
        for rule in &manifest.rules {
            let key = (rule.region, rule.region_lines);
            let entry = cache.entry(key).or_insert_with(|| {
                let lines = regions::extract(rule.region, rule.region_lines, snapshot);
                let text = lines.join("\n");
                (lines, text)
            });
            let context = PredicateContext {
                text: &entry.1,
                lines: &entry.0,
                progress_state: snapshot.osc_progress_state,
            };
            if rule.when.evaluate(&context) {
                winner = Some(rule);
                break;
            }
        }

        let rule = winner?;

        let capture = match (&rule.capture, rule.is_blocker()) {
            (Some(capture), _) => Some((capture.region, capture.region_lines, capture.max_chars)),
            (None, true) => Some((RegionKind::PromptBoxBody, 5, 400)),
            (None, false) => None,
        };

        let mut excerpt = None;
        let mut options = None;
        if let Some((region, region_lines, max_chars)) = capture {
            let lines = regions::extract(region, region_lines, snapshot);
            let joined = lines.join("\n");
            if !joined.is_empty() {
                let redacted = redact(&joined);
                excerpt = Some(redacted.chars().take(max_chars).collect());
            }
            if rule.is_blocker() {
                let found = regions::numbered_options(&lines);
                if !found.is_empty() {
                    options = Some(found);
                }
            }
        }

        Some(ScreenObservation {
            state: rule.state,
            matched_rule_id: rule.id.clone(),
            priority: rule.priority,
            content_seq: snapshot.content_seq,
            prompt_excerpt: excerpt,
            options,
        })
    }
}

/// Risk classification for a pending permission prompt.
///
/// Drives how loudly the UI asks for attention. Ported from diri-engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiskHint {
    Neutral,
    FileWrite,
    Network,
    Destructive,
}

const DESTRUCTIVE: &[&str] = &[
    "rm -rf",
    "rm -fr",
    "sudo ",
    "force-push",
    "push --force",
    "push -f",
    "reset --hard",
    "clean -fd",
    "drop table",
    "mkfs",
    "> /dev/",
];

const NETWORK: &[&str] = &[
    "curl ", "wget ", "http://", "https://", "ssh ", "scp ", "nc ",
];

const FILE_WRITE: &[&str] = &[
    "write", "edit", "mv ", "cp ", "mkdir", "touch ", "chmod", "chown",
];

pub fn classify_risk(text: &str) -> RiskHint {
    let lower = text.to_lowercase();
    if DESTRUCTIVE.iter().any(|pattern| lower.contains(pattern)) {
        return RiskHint::Destructive;
    }
    if NETWORK.iter().any(|pattern| lower.contains(pattern)) {
        return RiskHint::Network;
    }
    if FILE_WRITE.iter().any(|pattern| lower.contains(pattern)) {
        return RiskHint::FileWrite;
    }
    RiskHint::Neutral
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destructive_beats_everything_else() {
        assert_eq!(
            classify_risk("rm -rf /tmp && curl https://example.com"),
            RiskHint::Destructive
        );
    }

    #[test]
    fn classifies_each_tier() {
        assert_eq!(classify_risk("sudo reboot"), RiskHint::Destructive);
        assert_eq!(classify_risk("curl example.com"), RiskHint::Network);
        assert_eq!(classify_risk("edit src/main.rs"), RiskHint::FileWrite);
        assert_eq!(classify_risk("list the files"), RiskHint::Neutral);
    }

    #[test]
    fn matching_is_case_insensitive() {
        assert_eq!(classify_risk("RM -RF build"), RiskHint::Destructive);
    }
}
