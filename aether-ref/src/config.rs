use serde::{Deserialize, Serialize};

use crate::types::policy::{ConflictResolution, ValidationMode};

/// Runtime configuration for an Aether engine instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub controller_instance_id: String,
    #[serde(default)]
    pub validation_mode: ValidationMode,
    #[serde(default)]
    pub conflict_resolution: ConflictResolution,
}
