use std::collections::BTreeMap;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Registration entry for a telemetry adapter.
pub struct AdapterRegistration {
    pub adapter_id: String,
    shared_secret: Option<Vec<u8>>,
    heartbeat_interval: std::time::Duration,
    last_heartbeat: Option<DateTime<Utc>>,
    last_sequence: Option<u64>,
}

/// Registry of telemetry adapters with liveness tracking, sequence validation,
/// and HMAC-SHA256 signature verification.
pub struct AdapterRegistry {
    adapters: BTreeMap<String, AdapterRegistration>,
}

impl AdapterRegistry {
    /// Create an empty adapter registry.
    pub fn new() -> Self {
        Self {
            adapters: BTreeMap::new(),
        }
    }

    /// Register a new adapter with an optional shared secret and heartbeat interval.
    pub fn register_adapter(
        &mut self,
        id: String,
        shared_secret: Option<Vec<u8>>,
        heartbeat_interval: std::time::Duration,
    ) {
        self.adapters.insert(
            id.clone(),
            AdapterRegistration {
                adapter_id: id,
                shared_secret,
                heartbeat_interval,
                last_heartbeat: None,
                last_sequence: None,
            },
        );
    }

    /// Record a heartbeat for an adapter.
    pub fn record_heartbeat(&mut self, adapter_id: &str, now: DateTime<Utc>) {
        if let Some(reg) = self.adapters.get_mut(adapter_id) {
            reg.last_heartbeat = Some(now);
        }
    }

    /// Return adapter IDs where the time since the last heartbeat exceeds the
    /// configured heartbeat interval.
    pub fn check_liveness(&self, now: DateTime<Utc>) -> Vec<String> {
        self.adapters
            .values()
            .filter(|reg| {
                match reg.last_heartbeat {
                    Some(last) => {
                        let elapsed = now
                            .signed_duration_since(last)
                            .to_std()
                            .unwrap_or(std::time::Duration::MAX);
                        elapsed > reg.heartbeat_interval
                    }
                    // No heartbeat ever received — considered stale.
                    None => true,
                }
            })
            .map(|reg| reg.adapter_id.clone())
            .collect()
    }

    /// Validate that the sequence number is strictly increasing for the given adapter.
    pub fn check_sequence(&mut self, adapter_id: &str, sequence: u64) -> Result<(), String> {
        let reg = self
            .adapters
            .get_mut(adapter_id)
            .ok_or_else(|| format!("unknown adapter: {}", adapter_id))?;

        if let Some(last) = reg.last_sequence {
            if sequence <= last {
                return Err(format!(
                    "sequence regression for {}: got {} but last was {}",
                    adapter_id, sequence, last
                ));
            }
        }

        reg.last_sequence = Some(sequence);
        Ok(())
    }

    /// Verify an HMAC-SHA256 signature for the given data using the adapter's
    /// shared secret. If no secret is registered for the adapter, verification
    /// is skipped (returns Ok).
    pub fn verify_hmac(
        &self,
        adapter_id: &str,
        data: &[u8],
        signature: &str,
    ) -> Result<(), String> {
        let reg = self
            .adapters
            .get(adapter_id)
            .ok_or_else(|| format!("unknown adapter: {}", adapter_id))?;

        let secret = match &reg.shared_secret {
            Some(s) => s,
            None => return Ok(()), // No secret — skip verification.
        };

        let sig_bytes =
            hex::decode(signature).map_err(|e| format!("invalid hex signature: {}", e))?;

        let mut mac = HmacSha256::new_from_slice(secret)
            .map_err(|e| format!("HMAC init error: {}", e))?;
        mac.update(data);
        mac.verify_slice(&sig_bytes)
            .map_err(|_| format!("HMAC verification failed for adapter {}", adapter_id))
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration as ChronoDuration;
    use std::time::Duration;

    #[test]
    fn test_register_and_liveness() {
        let mut registry = AdapterRegistry::new();
        let interval = Duration::from_secs(30);
        registry.register_adapter("adapter-1".to_string(), None, interval);

        let now = Utc::now();

        // Before any heartbeat, adapter should be reported as stale.
        let stale = registry.check_liveness(now);
        assert!(stale.contains(&"adapter-1".to_string()));

        // Record a heartbeat — should no longer be stale.
        registry.record_heartbeat("adapter-1", now);
        let stale = registry.check_liveness(now);
        assert!(!stale.contains(&"adapter-1".to_string()));

        // Advance time past the heartbeat interval — should be stale again.
        let future = now + ChronoDuration::seconds(31);
        let stale = registry.check_liveness(future);
        assert!(stale.contains(&"adapter-1".to_string()));
    }

    #[test]
    fn test_sequence_monotonicity() {
        let mut registry = AdapterRegistry::new();
        registry.register_adapter("adapter-1".to_string(), None, Duration::from_secs(30));

        // First sequence should succeed.
        assert!(registry.check_sequence("adapter-1", 1).is_ok());
        // Strictly increasing should succeed.
        assert!(registry.check_sequence("adapter-1", 5).is_ok());
        // Same value should fail (not strictly increasing).
        assert!(registry.check_sequence("adapter-1", 5).is_err());
        // Lower value should fail (regression).
        assert!(registry.check_sequence("adapter-1", 3).is_err());
        // Higher value should succeed again.
        assert!(registry.check_sequence("adapter-1", 10).is_ok());
    }

    #[test]
    fn test_hmac_verification() {
        let secret = b"test-secret".to_vec();
        let mut registry = AdapterRegistry::new();
        registry.register_adapter("adapter-1".to_string(), Some(secret.clone()), Duration::from_secs(30));

        let data = b"hello world";

        // Compute a valid HMAC.
        let mut mac = HmacSha256::new_from_slice(&secret).unwrap();
        mac.update(data);
        let valid_sig = hex::encode(mac.finalize().into_bytes());

        // Valid signature should pass.
        assert!(registry.verify_hmac("adapter-1", data, &valid_sig).is_ok());

        // Invalid signature should fail.
        assert!(registry.verify_hmac("adapter-1", data, "deadbeef").is_err());
    }

    #[test]
    fn test_no_secret_skips_hmac() {
        let mut registry = AdapterRegistry::new();
        registry.register_adapter("adapter-1".to_string(), None, Duration::from_secs(30));

        // With no shared secret, any signature (even garbage) should pass.
        assert!(registry.verify_hmac("adapter-1", b"data", "anything").is_ok());
    }
}
