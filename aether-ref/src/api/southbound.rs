use crate::adapter::traits::SouthboundAdapter;
use crate::error::Result;
use crate::types::decision::{Decision, Directive};
use crate::types::link::{LinkId, TelemetryRecord};

/// Issue a directive to equipment via an adapter, based on a decision.
pub async fn issue_directive(
    adapter: &dyn SouthboundAdapter,
    decision: &Decision,
) -> Result<()> {
    let selected_links: Vec<LinkId> = decision.selected_links.clone();

    let directive = Directive {
        decision_id: decision.decision_id.clone(),
        traffic_class: decision.traffic_class.clone(),
        selected_links,
        fallback: decision.action_issued.link_selection.fallback.clone(),
    };

    adapter.apply_directive(&directive).await
}

/// Read telemetry from equipment via an adapter.
pub async fn read_link_telemetry(
    adapter: &dyn SouthboundAdapter,
    link_id: &LinkId,
) -> Result<TelemetryRecord> {
    let state = adapter.read_telemetry(link_id).await?;
    Ok(TelemetryRecord {
        link_id: link_id.clone(),
        state,
        received_at: chrono::Utc::now(),
    })
}
