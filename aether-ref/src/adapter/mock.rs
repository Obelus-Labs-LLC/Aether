use async_trait::async_trait;
use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::adapter::traits::SouthboundAdapter;
use crate::error::AetherError;
use crate::types::decision::Directive;
use crate::types::link::{Availability, LinkId, LinkState};

/// Mock adapter for testing. Records all directives and returns configurable telemetry.
pub struct MockAdapter {
    id: String,
    directives: Mutex<Vec<Directive>>,
    telemetry: Mutex<BTreeMap<LinkId, LinkState>>,
}

impl MockAdapter {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            directives: Mutex::new(Vec::new()),
            telemetry: Mutex::new(BTreeMap::new()),
        }
    }

    /// Configure the telemetry that will be returned for a given link.
    pub fn set_telemetry(&self, link_id: LinkId, state: LinkState) {
        self.telemetry.lock().unwrap_or_else(|p| p.into_inner()).insert(link_id, state);
    }

    /// Get all directives that have been applied.
    pub fn get_directives(&self) -> Vec<Directive> {
        self.directives.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }

    /// Clear recorded directives.
    pub fn clear_directives(&self) {
        self.directives.lock().unwrap_or_else(|p| p.into_inner()).clear();
    }
}

#[async_trait]
impl SouthboundAdapter for MockAdapter {
    async fn apply_directive(&self, directive: &Directive) -> Result<(), AetherError> {
        self.directives.lock().unwrap_or_else(|p| p.into_inner()).push(directive.clone());
        Ok(())
    }

    async fn read_telemetry(&self, link_id: &LinkId) -> Result<LinkState, AetherError> {
        self.telemetry
            .lock()
            .unwrap()
            .get(link_id)
            .cloned()
            .ok_or_else(|| {
                AetherError::Telemetry(format!("no telemetry for link {}", link_id))
            })
    }

    fn adapter_id(&self) -> &str {
        &self.id
    }
}

impl Default for MockAdapter {
    fn default() -> Self {
        let adapter = MockAdapter::new("mock-adapter");
        // Pre-populate with some default links
        let now = chrono::Utc::now();
        for (id, latency) in &[("leo_01", 50.0), ("lte_01", 20.0), ("mss_01", 200.0)] {
            adapter.set_telemetry(
                LinkId::from(*id),
                LinkState {
                    link_id: LinkId::from(*id),
                    latency_ms: Some(*latency),
                    jitter_ms: None,
                    availability: Availability::Up,
                    capacity_mbps: Some(100.0),
                    timestamp: now,
                    source_id: "mock".to_string(),
                },
            );
        }
        adapter
    }
}
