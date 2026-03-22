use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::telemetry::memory::ExperienceMemoryExport;
use crate::types::hcm::HcmStateExport;
use crate::types::link::TelemetrySnapshot;
use crate::types::policy::{PolicySet, TriggerValue};

/// Complete engine state snapshot for deterministic replay verification.
///
/// Per conformance spec section 4.1: export state, reset, replay inputs,
/// verify identical decisions and audit entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineSnapshot {
    pub policy_set: PolicySet,
    pub trigger_state: BTreeMap<String, TriggerValue>,
    pub telemetry_snapshot: TelemetrySnapshot,
    pub experience_memory: ExperienceMemoryExport,
    pub hcm_state: HcmStateExport,
    pub audit_chain_head: String,
    pub audit_sequence: u64,
}

/// A single evaluation input for replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayInput {
    pub decision_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub traffic_class: crate::types::traffic_class::TrafficClassLabel,
    pub trigger_state: BTreeMap<String, TriggerValue>,
    pub telemetry_snapshot: TelemetrySnapshot,
}

/// Result of a replay verification.
#[derive(Debug)]
pub struct ReplayResult {
    pub total_inputs: usize,
    pub matched: usize,
    pub mismatched: Vec<ReplayMismatch>,
}

#[derive(Debug)]
pub struct ReplayMismatch {
    pub input_index: usize,
    pub decision_id: String,
    pub expected_justification: String,
    pub actual_justification: String,
}

impl ReplayResult {
    pub fn is_deterministic(&self) -> bool {
        self.mismatched.is_empty()
    }
}
