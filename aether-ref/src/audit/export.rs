use crate::error::{AetherError, Result};
use crate::types::audit::AuditEntry;

/// Export audit entries to JSON.
pub fn export_json(entries: &[AuditEntry]) -> Result<String> {
    serde_json::to_string_pretty(entries)
        .map_err(|e| AetherError::Serialization(e.to_string()))
}

/// Export audit entries to a file.
pub fn export_to_file(entries: &[AuditEntry], path: &std::path::Path) -> Result<()> {
    let json = export_json(entries)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Query audit entries by time range.
pub fn query_by_time_range(
    entries: &[AuditEntry],
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
) -> Vec<&AuditEntry> {
    entries
        .iter()
        .filter(|e| e.decision.timestamp >= start && e.decision.timestamp <= end)
        .collect()
}

/// Query audit entries by decision ID.
pub fn query_by_decision_id<'a>(
    entries: &'a [AuditEntry],
    decision_id: &str,
) -> Option<&'a AuditEntry> {
    entries
        .iter()
        .find(|e| e.decision.decision_id == decision_id)
}
