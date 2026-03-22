use async_trait::async_trait;

use crate::error::AetherError;
use crate::types::decision::Directive;
use crate::types::link::{LinkId, LinkState};

/// Southbound adapter trait for communicating with network equipment.
///
/// Adapters MUST be mechanical translators only (per spec docs/03-integration-guide.md).
/// They MUST NOT perform traffic inspection, policy evaluation, decision override,
/// per-flow logic, or caching/delay.
#[async_trait]
pub trait SouthboundAdapter: Send + Sync {
    /// Apply a directive to network equipment.
    async fn apply_directive(&self, directive: &Directive) -> Result<(), AetherError>;

    /// Read current telemetry for a specific link.
    async fn read_telemetry(&self, link_id: &LinkId) -> Result<LinkState, AetherError>;

    /// Unique identifier for this adapter instance.
    fn adapter_id(&self) -> &str;
}
