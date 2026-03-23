//! Linux netlink adapter — translates Aether directives to ip route commands.
//!
//! This adapter uses `std::process::Command` to execute `ip route` commands
//! rather than raw netlink sockets, for simplicity and debuggability.

use std::collections::BTreeMap;
use std::sync::Mutex;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::adapter::traits::SouthboundAdapter;
use crate::error::AetherError;
use crate::types::decision::Directive;
use crate::types::link::{Availability, LinkId, LinkState};

/// Maps an Aether link ID to a physical network interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceMapping {
    pub link_id: String,
    pub interface_name: String,
    pub routing_table: u32,
    pub gateway: Option<String>,
    pub ping_target: Option<String>,
}

/// Saved routing state for rollback.
#[derive(Debug, Clone)]
struct RouteSnapshot {
    table: u32,
    routes: Vec<String>, // Raw output lines from `ip route show table N`
}

/// Linux netlink adapter implementing SouthboundAdapter.
pub struct LinuxNetlinkAdapter {
    id: String,
    mappings: BTreeMap<LinkId, InterfaceMapping>,
    snapshots: Mutex<BTreeMap<u32, RouteSnapshot>>,
}

impl LinuxNetlinkAdapter {
    /// Create from a list of interface mappings.
    pub fn new(id: String, mappings: Vec<InterfaceMapping>) -> Self {
        let map = mappings
            .into_iter()
            .map(|m| (LinkId(m.link_id.clone()), m))
            .collect();
        Self {
            id,
            mappings: map,
            snapshots: Mutex::new(BTreeMap::new()),
        }
    }

    /// Load mappings from a YAML configuration file.
    pub fn from_config_file(id: String, path: &std::path::Path) -> Result<Self, AetherError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AetherError::Adapter(format!("failed to read config: {}", e)))?;
        let mappings: Vec<InterfaceMapping> = serde_yaml::from_str(&content)
            .map_err(|e| AetherError::Adapter(format!("failed to parse config: {}", e)))?;
        Ok(Self::new(id, mappings))
    }

    /// Snapshot current routing state for a table.
    fn snapshot_table(&self, table: u32) -> Result<RouteSnapshot, AetherError> {
        let output = std::process::Command::new("ip")
            .args(["route", "show", "table", &table.to_string()])
            .output()
            .map_err(|e| AetherError::Adapter(format!("ip route show failed: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let routes: Vec<String> = stdout.lines().map(|l| l.to_string()).collect();
        Ok(RouteSnapshot { table, routes })
    }

    /// Restore a previously saved routing snapshot.
    fn restore_snapshot(&self, snapshot: &RouteSnapshot) -> Result<(), AetherError> {
        // Flush the table
        let _ = std::process::Command::new("ip")
            .args(["route", "flush", "table", &snapshot.table.to_string()])
            .output();

        // Re-add saved routes
        for route in &snapshot.routes {
            if route.trim().is_empty() {
                continue;
            }
            let args: Vec<&str> = route.split_whitespace().collect();
            let mut cmd = std::process::Command::new("ip");
            cmd.arg("route").arg("add");
            for arg in &args {
                cmd.arg(arg);
            }
            cmd.arg("table").arg(&snapshot.table.to_string());
            let _ = cmd.output();
        }
        Ok(())
    }

    /// Apply a single route change.
    fn apply_route(
        &self,
        mapping: &InterfaceMapping,
        metric: u32,
    ) -> Result<(), AetherError> {
        let mut cmd = std::process::Command::new("ip");
        cmd.args(["route", "replace", "default"]);

        if let Some(ref gw) = mapping.gateway {
            cmd.args(["via", gw]);
        }

        cmd.args(["dev", &mapping.interface_name]);
        cmd.args(["table", &mapping.routing_table.to_string()]);
        cmd.args(["metric", &metric.to_string()]);

        let output = cmd
            .output()
            .map_err(|e| AetherError::Adapter(format!("ip route replace failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AetherError::Adapter(format!(
                "ip route replace failed for {}: {}",
                mapping.interface_name, stderr
            )));
        }
        Ok(())
    }

    /// Read interface carrier state from sysfs.
    fn read_carrier(&self, iface: &str) -> Availability {
        let path = format!("/sys/class/net/{}/carrier", iface);
        match std::fs::read_to_string(&path) {
            Ok(content) => match content.trim() {
                "1" => Availability::Up,
                "0" => Availability::Down,
                _ => Availability::Degraded,
            },
            Err(_) => Availability::Down,
        }
    }

    /// Measure latency via ping.
    fn measure_latency(&self, target: &str) -> Option<f64> {
        let output = std::process::Command::new("ping")
            .args(["-c", "1", "-W", "2", target])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse "time=XX.X ms" from ping output
        for line in stdout.lines() {
            if let Some(idx) = line.find("time=") {
                let after = &line[idx + 5..];
                if let Some(end) = after.find(' ') {
                    return after[..end].parse::<f64>().ok();
                }
            }
        }
        None
    }

    /// Read interface speed from sysfs.
    fn read_speed(&self, iface: &str) -> Option<f64> {
        let path = format!("/sys/class/net/{}/speed", iface);
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .filter(|&v| v > 0.0)
    }
}

#[async_trait]
impl SouthboundAdapter for LinuxNetlinkAdapter {
    async fn apply_directive(&self, directive: &Directive) -> Result<(), AetherError> {
        // Collect affected tables
        let mut affected_tables: Vec<u32> = Vec::new();
        let mut saved_snapshots: Vec<RouteSnapshot> = Vec::new();

        for link_id in &directive.selected_links {
            if let Some(mapping) = self.mappings.get(link_id) {
                if !affected_tables.contains(&mapping.routing_table) {
                    affected_tables.push(mapping.routing_table);
                }
            }
        }

        // Snapshot current state for rollback
        for &table in &affected_tables {
            let snapshot = self.snapshot_table(table)?;
            saved_snapshots.push(snapshot);
        }

        // Apply routes with metrics based on preference order
        for (idx, link_id) in directive.selected_links.iter().enumerate() {
            if let Some(mapping) = self.mappings.get(link_id) {
                let metric = (idx as u32 + 1) * 10; // 10, 20, 30...
                if let Err(e) = self.apply_route(mapping, metric) {
                    // Rollback on failure
                    tracing::error!(error = %e, "route application failed, rolling back");
                    for snapshot in &saved_snapshots {
                        let _ = self.restore_snapshot(snapshot);
                    }
                    return Err(e);
                }
            }
        }

        // Store snapshots for future rollback
        let mut guard = self.snapshots.lock().unwrap();
        for snapshot in saved_snapshots {
            guard.insert(snapshot.table, snapshot);
        }

        Ok(())
    }

    async fn read_telemetry(&self, link_id: &LinkId) -> Result<LinkState, AetherError> {
        let mapping = self
            .mappings
            .get(link_id)
            .ok_or_else(|| AetherError::Adapter(format!("unknown link: {}", link_id)))?;

        let availability = self.read_carrier(&mapping.interface_name);

        let latency_ms = mapping
            .ping_target
            .as_ref()
            .and_then(|target| self.measure_latency(target));

        let capacity_mbps = self.read_speed(&mapping.interface_name);

        Ok(LinkState {
            link_id: link_id.clone(),
            latency_ms,
            jitter_ms: None,
            availability,
            capacity_mbps,
            timestamp: chrono::Utc::now(),
            source_id: self.id.clone(),
        })
    }

    fn adapter_id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping_creation() {
        let mappings = vec![
            InterfaceMapping {
                link_id: "leo_01".to_string(),
                interface_name: "wwan0".to_string(),
                routing_table: 100,
                gateway: Some("10.0.1.1".to_string()),
                ping_target: Some("10.0.1.2".to_string()),
            },
            InterfaceMapping {
                link_id: "lte_01".to_string(),
                interface_name: "eth1".to_string(),
                routing_table: 200,
                gateway: Some("10.0.2.1".to_string()),
                ping_target: Some("10.0.2.2".to_string()),
            },
        ];

        let adapter = LinuxNetlinkAdapter::new("test-adapter".to_string(), mappings);
        assert_eq!(adapter.mappings.len(), 2);
        assert!(adapter.mappings.contains_key(&LinkId("leo_01".to_string())));
        assert!(adapter.mappings.contains_key(&LinkId("lte_01".to_string())));
        assert_eq!(adapter.adapter_id(), "test-adapter");
    }

    #[test]
    fn test_carrier_parsing() {
        let adapter = LinuxNetlinkAdapter::new("test".to_string(), vec![]);
        // Can't test sysfs on non-Linux, but verify the function exists
        // and returns Down for nonexistent interfaces
        let avail = adapter.read_carrier("nonexistent_iface_xyz");
        assert!(matches!(avail, Availability::Down));
    }
}
