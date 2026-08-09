//! Manifest schema: detection rules for agent terminal screens.
//!
//! Ported from diri-engine. Both engines read the same manifest files so
//! adding an agent stays a one-file change.

use std::fmt;

use regex::Regex;
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegionKind {
    OscTitle,
    OscProgress,
    WholeRecent,
    PromptBoxBody,
    BottomNonEmptyLines,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ManifestState {
    Working,
    Idle,
    BlockedPermission,
    BlockedQuestion,
    Skip,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StatusModel {
    Full,
    ProcessOnly,
}

/// What a predicate is evaluated against.
pub struct PredicateContext<'a> {
    pub text: &'a str,
    pub lines: &'a [String],
    pub progress_state: Option<i64>,
}

/// A recursive detection predicate. Patterns compile once, at load.
#[derive(Debug)]
pub enum Predicate {
    Contains(String),
    Regex(Regex),
    LineRegex(Regex),
    Progress(i64),
    Any(Vec<Predicate>),
    All(Vec<Predicate>),
    Not(Box<Predicate>),
}

impl Predicate {
    pub fn evaluate(&self, context: &PredicateContext) -> bool {
        match self {
            Predicate::Contains(needle) => {
                context.text.to_lowercase().contains(&needle.to_lowercase())
            }
            Predicate::Regex(regex) => regex.is_match(context.text),
            Predicate::LineRegex(regex) => context.lines.iter().any(|line| regex.is_match(line)),
            Predicate::Progress(state) => context.progress_state == Some(*state),
            Predicate::Any(inner) => inner.iter().any(|p| p.evaluate(context)),
            Predicate::All(inner) => inner.iter().all(|p| p.evaluate(context)),
            Predicate::Not(inner) => !inner.evaluate(context),
        }
    }
}

impl<'de> Deserialize<'de> for Predicate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PredicateVisitor;

        #[derive(Deserialize)]
        struct ProgressSpec {
            state: i64,
        }

        impl<'de> Visitor<'de> for PredicateVisitor {
            type Value = Predicate;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a predicate object")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Predicate, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut found: Option<Predicate> = None;
                while let Some(key) = map.next_key::<String>()? {
                    let predicate = match key.as_str() {
                        "contains" => Predicate::Contains(map.next_value()?),
                        "regex" => Predicate::Regex(compile::<M>(map.next_value()?)?),
                        "lineRegex" => Predicate::LineRegex(compile::<M>(map.next_value()?)?),
                        "progress" => {
                            let spec: ProgressSpec = map.next_value()?;
                            Predicate::Progress(spec.state)
                        }
                        "any" => Predicate::Any(map.next_value()?),
                        "all" => Predicate::All(map.next_value()?),
                        "not" => Predicate::Not(Box::new(map.next_value()?)),
                        other => {
                            return Err(de::Error::custom(format!(
                                "unrecognized predicate key {other:?}"
                            )));
                        }
                    };
                    found = Some(predicate);
                }
                found.ok_or_else(|| de::Error::custom("predicate object had no recognized key"))
            }
        }

        fn compile<'de, M: MapAccess<'de>>(pattern: String) -> Result<Regex, M::Error> {
            Regex::new(&pattern).map_err(|error| {
                de::Error::custom(format!("pattern {pattern:?} did not compile: {error}"))
            })
        }

        deserializer.deserialize_map(PredicateVisitor)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capture {
    pub region: RegionKind,
    #[serde(default = "default_region_lines")]
    pub region_lines: usize,
    #[serde(default = "default_max_chars")]
    pub max_chars: usize,
}

fn default_region_lines() -> usize {
    5
}

fn default_max_chars() -> usize {
    400
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub state: ManifestState,
    pub priority: i64,
    pub region: RegionKind,
    #[serde(default = "default_region_lines")]
    pub region_lines: usize,
    pub when: Predicate,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub capture: Option<Capture>,
}

impl Rule {
    pub fn is_blocker(&self) -> bool {
        matches!(
            self.state,
            ManifestState::BlockedPermission | ManifestState::BlockedQuestion
        )
    }

    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.iter().any(|f| f == flag)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: i64,
    pub id: String,
    pub version: String,
    pub status_model: StatusModel,
    #[serde(default)]
    pub agent: Option<crate::AgentManifest>,
    #[serde(deserialize_with = "rules_by_priority")]
    pub rules: Vec<Rule>,
}

fn rules_by_priority<'de, D>(deserializer: D) -> Result<Vec<Rule>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut rules = Vec::<Rule>::deserialize(deserializer)?;
    rules.sort_by_key(|rule| std::cmp::Reverse(rule.priority));
    Ok(rules)
}
