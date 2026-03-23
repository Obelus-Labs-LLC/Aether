use serde::{Deserialize, Serialize};

use crate::types::policy::{ConflictResolution, ValidationMode};

/// What to do when telemetry is missing or stale for links in a matched rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MissingTelemetryAction {
    /// Proceed with last-known data. Current behavior.
    #[serde(rename = "use_last_known")]
    UseLastKnown,
    /// Proceed but mark justification as degraded.
    #[serde(rename = "mark_degraded")]
    MarkDegraded,
    /// Reject evaluation and return an error.
    #[serde(rename = "reject_evaluation")]
    RejectEvaluation,
}

impl Default for MissingTelemetryAction {
    fn default() -> Self {
        Self::UseLastKnown
    }
}

/// Runtime configuration for an Aether engine instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub controller_instance_id: String,
    #[serde(default)]
    pub validation_mode: ValidationMode,
    #[serde(default)]
    pub conflict_resolution: ConflictResolution,
    /// How to handle missing or stale telemetry during evaluation.
    #[serde(default)]
    pub missing_telemetry_action: MissingTelemetryAction,
    /// Staleness threshold in seconds. Links with telemetry older than this
    /// are considered stale. Default: 300 (5 minutes).
    #[serde(default = "default_staleness_secs")]
    pub staleness_threshold_secs: u64,
}

fn default_staleness_secs() -> u64 {
    300
}
