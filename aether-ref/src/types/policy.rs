use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::link::LinkId;

/// The complete collection of active policies governing an Aether deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySet {
    pub policy_set_version: String,
    pub policies: Vec<Policy>,
    pub defaults: Defaults,
    #[serde(default)]
    pub conflict_resolution: ConflictResolution,
    #[serde(default)]
    pub validation_mode: ValidationMode,
}

/// A single policy within a policy set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub name: String,
    pub version: String,
    /// Higher value = higher precedence.
    pub priority: i64,
    #[serde(default)]
    pub triggers: Option<TriggerBlock>,
    pub rules: Vec<Rule>,
}

/// Trigger conditions that must be met for a policy to be active.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerBlock {
    #[serde(default)]
    pub all_of: Option<Vec<TriggerCondition>>,
    #[serde(default)]
    pub any_of: Option<Vec<TriggerCondition>>,
}

/// A single trigger condition comparing a named state variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCondition {
    pub name: String,
    pub op: ComparisonOp,
    pub value: TriggerValue,
}

/// Comparison operators for conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOp {
    #[serde(rename = "==")]
    Eq,
    #[serde(rename = "!=")]
    Ne,
    #[serde(rename = "<")]
    Lt,
    #[serde(rename = "<=")]
    Le,
    #[serde(rename = ">")]
    Gt,
    #[serde(rename = ">=")]
    Ge,
}

/// A trigger value — supports multiple primitive types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TriggerValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

impl TriggerValue {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            TriggerValue::Int(i) => Some(*i as f64),
            TriggerValue::Float(f) => Some(*f),
            _ => None,
        }
    }
}

/// A rule within a policy: match conditions → actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    #[serde(rename = "match")]
    pub match_block: MatchBlock,
    pub actions: ActionBlock,
}

/// Match conditions for a rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchBlock {
    pub traffic_class: TrafficClassMatch,
    #[serde(default)]
    pub link_state: Option<BTreeMap<String, LinkStateConditions>>,
}

/// Traffic class matching within a rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficClassMatch {
    pub label_id: String,
    pub label_source: String,
}

impl TrafficClassMatch {
    /// Check if this match pattern matches a given label.
    /// Wildcard "*" matches any value. Exact matches have higher specificity.
    pub fn matches(&self, label_id: &str, label_source: &str) -> bool {
        let id_match = self.label_id == "*" || self.label_id == label_id;
        let source_match = self.label_source == "*" || self.label_source == label_source;
        id_match && source_match
    }

}

/// Link state conditions for a specific link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkStateConditions {
    #[serde(default)]
    pub latency_ms: Option<Condition>,
    #[serde(default)]
    pub availability: Option<StringCondition>,
    #[serde(default)]
    pub capacity_mbps: Option<Condition>,
}

/// A numeric condition (op + value).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub op: ComparisonOp,
    pub value: f64,
}

/// A string condition (op + value).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringCondition {
    pub op: ComparisonOp,
    pub value: String,
}

/// Actions to take when a rule matches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionBlock {
    pub link_selection: LinkSelection,
}

/// Link selection action specifying preferred links and fallback behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkSelection {
    #[serde(default)]
    pub prefer: Vec<LinkId>,
    pub fallback: FallbackMode,
}

/// Fallback behavior when preferred links are unavailable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackMode {
    AnyAvailable,
    DeferToRouting,
    ShedViaEquipment,
}

/// Default action when no policy rule matches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
    pub no_match_action: ActionBlock,
}

/// How to resolve conflicts between policies with identical priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    LexicographicPolicyName,
    RequireUnique,
    FirstLoaded,
}

impl Default for ConflictResolution {
    fn default() -> Self {
        ConflictResolution::LexicographicPolicyName
    }
}

/// Policy validation strictness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationMode {
    Strict,
    Permissive,
}

impl Default for ValidationMode {
    fn default() -> Self {
        ValidationMode::Strict
    }
}
