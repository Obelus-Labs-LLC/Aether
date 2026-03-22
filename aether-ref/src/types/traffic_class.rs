use serde::{Deserialize, Serialize};

/// An opaque traffic class label assigned by an external classifier.
/// Aether reads these but never interprets their semantic meaning.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrafficClassLabel {
    pub label_id: String,
    /// Classification source (e.g., "DSCP", "VLAN", "policy_engine").
    /// Use "*" for wildcard matching.
    pub label_source: String,
}
