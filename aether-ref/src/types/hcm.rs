use serde::{Deserialize, Serialize};

/// An HCM (Human Continuity Mode) activation event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HcmActivation {
    pub event_type: String,
    pub event_id: String,
    pub actor_id: String,
    pub actor_type: ActorType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub reason: String,
    /// Affected traffic class label_ids.
    pub scope: Vec<String>,
    pub authorization_method: String,
    /// Maximum duration for this activation in seconds. Default: 14400 (4 hours).
    #[serde(default = "default_max_duration")]
    pub max_duration_seconds: u64,
    /// Maximum cumulative HCM duration in seconds. Default: 259200 (72 hours).
    #[serde(default = "default_max_total")]
    pub max_total_duration_seconds: u64,
    /// Whether renewal is allowed. Per spec, default is false (not allowed).
    #[serde(default)]
    pub allow_renewal: bool,
}

fn default_max_duration() -> u64 {
    14_400
}

fn default_max_total() -> u64 {
    259_200
}

/// Who activated HCM.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    HumanOperator,
    AutomatedSystem,
    ExternalAuthority,
}

/// Runtime HCM state tracked by the engine.
#[derive(Debug, Clone)]
pub struct HcmState {
    pub active: bool,
    pub activation: Option<HcmActivation>,
    pub started_monotonic: Option<std::time::Instant>,
    pub cumulative_seconds: u64,
    pub renewal_count: u32,
}

impl Default for HcmState {
    fn default() -> Self {
        Self {
            active: false,
            activation: None,
            started_monotonic: None,
            cumulative_seconds: 0,
            renewal_count: 0,
        }
    }
}

/// Exportable snapshot of HCM state (without Instant, which can't be serialized).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HcmStateExport {
    pub active: bool,
    pub activation: Option<HcmActivation>,
    pub cumulative_seconds: u64,
    pub renewal_count: u32,
    pub elapsed_current_seconds: Option<u64>,
}

impl HcmState {
    /// Export HCM state. Takes elapsed duration from caller (using injected clock)
    /// rather than calling Instant::elapsed() which bypasses mock clocks.
    pub fn export_with_elapsed(&self, elapsed: Option<std::time::Duration>) -> HcmStateExport {
        HcmStateExport {
            active: self.active,
            activation: self.activation.clone(),
            cumulative_seconds: self.cumulative_seconds,
            renewal_count: self.renewal_count,
            elapsed_current_seconds: elapsed.map(|d| d.as_secs()),
        }
    }

    /// Export HCM state using real clock (convenience for non-test use).
    pub fn export(&self) -> HcmStateExport {
        let elapsed = self
            .started_monotonic
            .map(|start| start.elapsed());
        self.export_with_elapsed(elapsed)
    }

    /// Check if the given label_id is within the current HCM scope.
    pub fn is_in_scope(&self, label_id: &str) -> bool {
        match &self.activation {
            Some(act) if self.active => act.scope.iter().any(|s| s == label_id),
            _ => false,
        }
    }
}
