use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::link::{LinkId, LinkState};
use super::policy::{ActionBlock, TriggerValue};
use super::traffic_class::TrafficClassLabel;

/// The output of a single policy evaluation — deterministic given identical inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub decision_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub policy_set_version: String,
    pub traffic_class: TrafficClassLabel,
    pub trigger_snapshot: BTreeMap<String, TriggerValue>,
    pub rule_matched: Option<RuleRef>,
    pub action_issued: ActionBlock,
    /// The actual links selected after tiebreak/availability filtering.
    pub selected_links: Vec<LinkId>,
    pub justification: JustificationCode,
    pub telemetry_snapshot: BTreeMap<LinkId, LinkState>,
}

/// Reference to the specific policy and rule that matched.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleRef {
    pub policy_name: String,
    pub rule_index: usize,
}

/// Machine-readable justification for the decision made.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JustificationCode {
    PolicyMatch {
        policy: String,
        rule_index: usize,
    },
    DefaultApplied,
    HcmOverride {
        activation_id: String,
    },
    TelemetryDegraded {
        missing_links: Vec<LinkId>,
    },
}

/// A directive issued to equipment as a result of a decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directive {
    pub decision_id: String,
    pub traffic_class: TrafficClassLabel,
    pub selected_links: Vec<LinkId>,
    pub fallback: super::policy::FallbackMode,
}
