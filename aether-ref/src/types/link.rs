use serde::{Deserialize, Serialize};

/// Stable, operator-defined link identifier.
/// Must be consistent across restarts and not derived from transient attributes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LinkId(pub String);

impl std::fmt::Display for LinkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for LinkId {
    fn from(s: &str) -> Self {
        LinkId(s.to_string())
    }
}

/// Link availability state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Up,
    Down,
    Degraded,
}

/// Observable link-level state at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkState {
    pub link_id: LinkId,
    pub latency_ms: Option<f64>,
    pub jitter_ms: Option<f64>,
    pub availability: Availability,
    pub capacity_mbps: Option<f64>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source_id: String,
}

/// A telemetry record as received by the ingestion layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryRecord {
    pub link_id: LinkId,
    pub state: LinkState,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub received_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub sequence_number: Option<u64>,
    #[serde(default)]
    pub hmac_signature: Option<String>,
}

/// A snapshot of all link states at evaluation time.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub links: std::collections::BTreeMap<LinkId, LinkState>,
}
