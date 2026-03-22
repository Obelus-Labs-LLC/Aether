use std::collections::BTreeMap;

use crate::types::link::{LinkId, LinkState, TelemetryRecord, TelemetrySnapshot};

/// Stores and queries link-level telemetry.
/// Maintains the latest state per link and detects staleness.
#[derive(Debug, Default)]
pub struct TelemetryStore {
    latest: BTreeMap<LinkId, TelemetryRecord>,
    staleness_threshold: std::time::Duration,
}

impl TelemetryStore {
    pub fn new(staleness_threshold: std::time::Duration) -> Self {
        Self {
            latest: BTreeMap::new(),
            staleness_threshold,
        }
    }

    /// Ingest a new telemetry record, updating the latest state for its link.
    pub fn ingest(&mut self, record: TelemetryRecord) {
        let link_id = record.link_id.clone();
        self.latest
            .entry(link_id)
            .and_modify(|existing| {
                if record.received_at > existing.received_at {
                    *existing = record.clone();
                }
            })
            .or_insert(record);
    }

    /// Get the latest state for a specific link.
    pub fn get(&self, link_id: &LinkId) -> Option<&LinkState> {
        self.latest.get(link_id).map(|r| &r.state)
    }

    /// Check if telemetry for a link is stale (older than threshold).
    pub fn is_stale(&self, link_id: &LinkId, now: chrono::DateTime<chrono::Utc>) -> bool {
        match self.latest.get(link_id) {
            Some(record) => {
                let age = now
                    .signed_duration_since(record.received_at)
                    .to_std()
                    .unwrap_or(std::time::Duration::MAX);
                age > self.staleness_threshold
            }
            None => true,
        }
    }

    /// Build a snapshot of all current link states for policy evaluation.
    pub fn snapshot(&self) -> TelemetrySnapshot {
        let links = self
            .latest
            .iter()
            .map(|(id, record)| (id.clone(), record.state.clone()))
            .collect();
        TelemetrySnapshot { links }
    }

    /// List all known link IDs.
    pub fn known_links(&self) -> Vec<LinkId> {
        self.latest.keys().cloned().collect()
    }

    /// Find links with stale telemetry.
    pub fn stale_links(&self, now: chrono::DateTime<chrono::Utc>) -> Vec<LinkId> {
        self.latest
            .keys()
            .filter(|id| self.is_stale(id, now))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::link::Availability;

    fn make_record(id: &str, latency: f64) -> TelemetryRecord {
        let now = chrono::Utc::now();
        TelemetryRecord {
            link_id: LinkId::from(id),
            state: LinkState {
                link_id: LinkId::from(id),
                latency_ms: Some(latency),
                jitter_ms: None,
                availability: Availability::Up,
                capacity_mbps: None,
                timestamp: now,
                source_id: "test".to_string(),
            },
            received_at: now,
        }
    }

    #[test]
    fn ingest_and_query() {
        let mut store = TelemetryStore::new(std::time::Duration::from_secs(60));
        store.ingest(make_record("link_a", 50.0));

        let state = store.get(&LinkId::from("link_a")).unwrap();
        assert_eq!(state.latency_ms, Some(50.0));
    }

    #[test]
    fn newer_record_replaces_older() {
        let mut store = TelemetryStore::new(std::time::Duration::from_secs(60));
        store.ingest(make_record("link_a", 50.0));
        store.ingest(make_record("link_a", 30.0));

        let state = store.get(&LinkId::from("link_a")).unwrap();
        assert_eq!(state.latency_ms, Some(30.0));
    }

    #[test]
    fn snapshot_includes_all_links() {
        let mut store = TelemetryStore::new(std::time::Duration::from_secs(60));
        store.ingest(make_record("link_a", 50.0));
        store.ingest(make_record("link_b", 30.0));

        let snap = store.snapshot();
        assert_eq!(snap.links.len(), 2);
    }
}
