use std::sync::Arc;
use tokio::sync::RwLock;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use crate::api::http_types::*;
use crate::api::northbound::AetherEngine;
use crate::audit::export::{query_by_decision_id, query_by_time_range};
use crate::hcm::clock::SystemClock;
use crate::types::hcm::HcmActivation;
use crate::types::link::TelemetryRecord;
use crate::types::policy::PolicySet;

pub type SharedEngine = Arc<RwLock<AetherEngine<SystemClock>>>;

#[derive(Clone)]
pub struct AppState {
    pub engine: SharedEngine,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/policies", get(get_policies).post(load_policies))
        .route("/api/v1/evaluate", post(evaluate_traffic))
        .route("/api/v1/telemetry", post(ingest_telemetry))
        .route("/api/v1/audit", get(query_audit))
        .route("/api/v1/audit/{decision_id}", get(get_audit_entry))
        .route("/api/v1/hcm/activate", post(activate_hcm))
        .route("/api/v1/hcm/deactivate", post(deactivate_hcm))
        .route("/api/v1/hcm/state", get(get_hcm_state))
        .route("/api/v1/health", get(health))
        .with_state(state)
}

async fn load_policies(
    State(state): State<AppState>,
    Json(policy_set): Json<PolicySet>,
) -> impl IntoResponse {
    let mut engine = state.engine.write().await;
    match engine.load_policy_set(policy_set) {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "loaded"}))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "POLICY_VALIDATION_ERROR".to_string(),
            }),
        )
            .into_response(),
    }
}

async fn get_policies(State(state): State<AppState>) -> impl IntoResponse {
    let engine = state.engine.read().await;
    match engine.policy_set() {
        Some(ps) => (StatusCode::OK, Json(serde_json::json!(ps))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "no policy set loaded".to_string(),
                code: "NOT_FOUND".to_string(),
            }),
        )
            .into_response(),
    }
}

async fn evaluate_traffic(
    State(state): State<AppState>,
    Json(req): Json<EvaluateRequest>,
) -> impl IntoResponse {
    let mut engine = state.engine.write().await;
    match engine.evaluate(&req.traffic_class) {
        Ok(decision) => {
            let resp = EvaluateResponse {
                decision,
                directive_applied: false,
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => {
            let code = match &e {
                crate::error::AetherError::PolicyValidation(_) => "POLICY_ERROR",
                crate::error::AetherError::Telemetry(_) => "TELEMETRY_ERROR",
                _ => "EVALUATION_ERROR",
            };
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                    code: code.to_string(),
                }),
            )
                .into_response()
        }
    }
}

async fn ingest_telemetry(
    State(state): State<AppState>,
    Json(record): Json<TelemetryRecord>,
) -> impl IntoResponse {
    let mut engine = state.engine.write().await;
    engine.telemetry_store_mut().ingest(record);
    StatusCode::NO_CONTENT
}

async fn query_audit(
    State(state): State<AppState>,
    Query(params): Query<AuditQueryParams>,
) -> impl IntoResponse {
    let engine = state.engine.read().await;
    let entries = engine.audit_log().entries();

    let filtered = match (params.from, params.to) {
        (Some(from_str), Some(to_str)) => {
            let from = match chrono::DateTime::parse_from_rfc3339(&from_str) {
                Ok(dt) => dt.with_timezone(&chrono::Utc),
                Err(e) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!(ErrorResponse {
                            error: format!("invalid 'from' timestamp: {e}"),
                            code: "INVALID_PARAMETER".to_string(),
                        })),
                    )
                        .into_response();
                }
            };
            let to = match chrono::DateTime::parse_from_rfc3339(&to_str) {
                Ok(dt) => dt.with_timezone(&chrono::Utc),
                Err(e) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!(ErrorResponse {
                            error: format!("invalid 'to' timestamp: {e}"),
                            code: "INVALID_PARAMETER".to_string(),
                        })),
                    )
                        .into_response();
                }
            };
            let refs = query_by_time_range(entries, from, to);
            refs.into_iter().cloned().collect::<Vec<_>>()
        }
        (Some(from_str), None) => {
            let from = match chrono::DateTime::parse_from_rfc3339(&from_str) {
                Ok(dt) => dt.with_timezone(&chrono::Utc),
                Err(e) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!(ErrorResponse {
                            error: format!("invalid 'from' timestamp: {e}"),
                            code: "INVALID_PARAMETER".to_string(),
                        })),
                    )
                        .into_response();
                }
            };
            entries
                .iter()
                .filter(|e| e.decision.timestamp >= from)
                .cloned()
                .collect::<Vec<_>>()
        }
        (None, Some(to_str)) => {
            let to = match chrono::DateTime::parse_from_rfc3339(&to_str) {
                Ok(dt) => dt.with_timezone(&chrono::Utc),
                Err(e) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!(ErrorResponse {
                            error: format!("invalid 'to' timestamp: {e}"),
                            code: "INVALID_PARAMETER".to_string(),
                        })),
                    )
                        .into_response();
                }
            };
            entries
                .iter()
                .filter(|e| e.decision.timestamp <= to)
                .cloned()
                .collect::<Vec<_>>()
        }
        (None, None) => entries.to_vec(),
    };

    (StatusCode::OK, Json(filtered)).into_response()
}

async fn get_audit_entry(
    State(state): State<AppState>,
    Path(decision_id): Path<String>,
) -> impl IntoResponse {
    let engine = state.engine.read().await;
    let entries = engine.audit_log().entries();

    match query_by_decision_id(entries, &decision_id) {
        Some(entry) => (StatusCode::OK, Json(serde_json::json!(entry))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("audit entry not found for decision_id: {decision_id}"),
                code: "NOT_FOUND".to_string(),
            }),
        )
            .into_response(),
    }
}

async fn activate_hcm(
    State(state): State<AppState>,
    Json(activation): Json<HcmActivation>,
) -> impl IntoResponse {
    let mut engine = state.engine.write().await;
    match engine.activate_hcm(activation) {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "activated"}))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "HCM_ERROR".to_string(),
            }),
        )
            .into_response(),
    }
}

async fn deactivate_hcm(State(state): State<AppState>) -> impl IntoResponse {
    let mut engine = state.engine.write().await;
    match engine.deactivate_hcm() {
        Ok(()) => {
            (StatusCode::OK, Json(serde_json::json!({"status": "deactivated"}))).into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "HCM_ERROR".to_string(),
            }),
        )
            .into_response(),
    }
}

async fn get_hcm_state(State(state): State<AppState>) -> impl IntoResponse {
    let engine = state.engine.read().await;
    let export = engine.hcm_state().export();
    (StatusCode::OK, Json(export))
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let engine = state.engine.read().await;
    let policy_loaded = engine.policy_set().is_some();
    let audit_chain_valid = engine.verify_audit_chain().is_ok();
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok".to_string(),
            policy_loaded,
            audit_chain_valid,
        }),
    )
}

pub async fn serve(state: AppState, bind_addr: &str) {
    let app = router(state);
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind to {bind_addr}: {e}"));
    tracing::info!("Aether HTTP API listening on {bind_addr}");
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("server error: {e}"));
}
