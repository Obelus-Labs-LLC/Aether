use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::error::Result;
use crate::types::audit::AuditEntry;
use crate::types::decision::Decision;

type HmacSha256 = Hmac<Sha256>;

const INITIAL_HMAC: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Append-only, tamper-evident audit logger using HMAC-SHA256 chaining.
///
/// Each entry's HMAC is computed over: the entry's decision content + the previous entry's HMAC.
/// Modifying any entry breaks the chain from that point forward.
pub struct AuditLogger {
    key: Vec<u8>,
    entries: Vec<AuditEntry>,
    head_hmac: String,
    next_sequence: u64,
}

impl AuditLogger {
    /// Create a new audit logger with the given HMAC key.
    /// The key is used for HMAC computation but is NOT stored in audit logs.
    pub fn new(key: Vec<u8>) -> Self {
        Self {
            key,
            entries: Vec::new(),
            head_hmac: INITIAL_HMAC.to_string(),
            next_sequence: 0,
        }
    }

    /// Log a decision. Returns the audit entry with HMAC chain.
    pub fn log(&mut self, decision: Decision) -> Result<&AuditEntry> {
        let previous_hmac = self.head_hmac.clone();
        let sequence = self.next_sequence;

        // Compute HMAC over: decision JSON + previous HMAC
        let decision_json = serde_json::to_string(&decision)
            .map_err(|e| crate::error::AetherError::Serialization(e.to_string()))?;

        let mut mac = HmacSha256::new_from_slice(&self.key)
            .expect("HMAC accepts any key length");
        mac.update(decision_json.as_bytes());
        mac.update(previous_hmac.as_bytes());
        let hmac_result = hex::encode(mac.finalize().into_bytes());

        let entry = AuditEntry {
            sequence,
            decision,
            hmac: hmac_result.clone(),
            previous_hmac,
        };

        self.head_hmac = hmac_result;
        self.next_sequence += 1;
        self.entries.push(entry);

        Ok(self.entries.last().unwrap())
    }

    /// Get the current chain head HMAC.
    pub fn head_hmac(&self) -> &str {
        &self.head_hmac
    }

    /// Get the current sequence number.
    pub fn sequence(&self) -> u64 {
        self.next_sequence
    }

    /// Get all audit entries.
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// Verify the integrity of the entire audit chain.
    pub fn verify(&self) -> std::result::Result<(), u64> {
        let mut expected_prev = INITIAL_HMAC.to_string();

        for entry in &self.entries {
            if entry.previous_hmac != expected_prev {
                return Err(entry.sequence);
            }

            // Recompute HMAC
            let decision_json = serde_json::to_string(&entry.decision)
                .expect("decision serialization should not fail on stored entry");

            let mut mac = HmacSha256::new_from_slice(&self.key)
                .expect("HMAC accepts any key length");
            mac.update(decision_json.as_bytes());
            mac.update(entry.previous_hmac.as_bytes());
            let computed = hex::encode(mac.finalize().into_bytes());

            if computed != entry.hmac {
                return Err(entry.sequence);
            }

            expected_prev = entry.hmac.clone();
        }

        Ok(())
    }

    /// Reset the logger (for replay testing).
    pub fn reset(&mut self) {
        self.entries.clear();
        self.head_hmac = INITIAL_HMAC.to_string();
        self.next_sequence = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::decision::{Decision, JustificationCode};
    use crate::types::policy::{ActionBlock, FallbackMode, LinkSelection};
    use crate::types::traffic_class::TrafficClassLabel;
    use std::collections::BTreeMap;

    fn make_decision(id: &str) -> Decision {
        Decision {
            decision_id: id.to_string(),
            timestamp: chrono::Utc::now(),
            policy_set_version: "v1".to_string(),
            traffic_class: TrafficClassLabel {
                label_id: "test".to_string(),
                label_source: "test".to_string(),
            },
            trigger_snapshot: BTreeMap::new(),
            rule_matched: None,
            action_issued: ActionBlock {
                link_selection: LinkSelection {
                    prefer: vec![],
                    fallback: FallbackMode::DeferToRouting,
                },
            },
            selected_links: vec![],
            justification: JustificationCode::DefaultApplied,
            telemetry_snapshot: BTreeMap::new(),
        }
    }

    #[test]
    fn chain_integrity() {
        let mut logger = AuditLogger::new(b"test-key".to_vec());
        logger.log(make_decision("d1")).unwrap();
        logger.log(make_decision("d2")).unwrap();
        logger.log(make_decision("d3")).unwrap();

        assert!(logger.verify().is_ok());
        assert_eq!(logger.entries().len(), 3);
    }

    #[test]
    fn tampering_detected() {
        let mut logger = AuditLogger::new(b"test-key".to_vec());
        logger.log(make_decision("d1")).unwrap();
        logger.log(make_decision("d2")).unwrap();

        // Tamper with the first entry
        logger.entries.first_mut().unwrap().hmac = "tampered".to_string();

        assert!(logger.verify().is_err());
    }

    #[test]
    fn sequential_numbering() {
        let mut logger = AuditLogger::new(b"key".to_vec());
        logger.log(make_decision("d1")).unwrap();
        logger.log(make_decision("d2")).unwrap();

        assert_eq!(logger.entries()[0].sequence, 0);
        assert_eq!(logger.entries()[1].sequence, 1);
    }

    #[test]
    fn chain_links_correctly() {
        let mut logger = AuditLogger::new(b"key".to_vec());
        logger.log(make_decision("d1")).unwrap();
        logger.log(make_decision("d2")).unwrap();

        assert_eq!(logger.entries()[0].previous_hmac, INITIAL_HMAC);
        assert_eq!(logger.entries()[1].previous_hmac, logger.entries()[0].hmac);
    }

    #[test]
    fn reset_clears_state() {
        let mut logger = AuditLogger::new(b"key".to_vec());
        logger.log(make_decision("d1")).unwrap();
        logger.reset();

        assert_eq!(logger.entries().len(), 0);
        assert_eq!(logger.head_hmac(), INITIAL_HMAC);
        assert_eq!(logger.sequence(), 0);
    }
}
