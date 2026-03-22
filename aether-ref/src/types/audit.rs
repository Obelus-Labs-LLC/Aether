use serde::{Deserialize, Serialize};

use super::decision::Decision;

/// A single entry in the tamper-evident audit log.
/// Each entry is chained to the previous via HMAC-SHA256.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub sequence: u64,
    pub decision: Decision,
    /// HMAC-SHA256 hex digest of this entry's content + previous HMAC.
    pub hmac: String,
    /// HMAC of the previous entry (or initial seed value for sequence 0).
    pub previous_hmac: String,
}
