use serde::{Deserialize, Serialize};

use crate::types::traffic_class::TrafficClassLabel;

#[derive(Debug, Deserialize)]
pub struct EvaluateRequest {
    pub traffic_class: TrafficClassLabel,
}

#[derive(Debug, Serialize)]
pub struct EvaluateResponse {
    pub decision: crate::types::decision::Decision,
    pub directive_applied: bool,
}

#[derive(Debug, Deserialize)]
pub struct AuditQueryParams {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub policy_loaded: bool,
    pub audit_chain_valid: bool,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}
