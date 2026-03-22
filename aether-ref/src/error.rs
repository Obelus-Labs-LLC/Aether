use thiserror::Error;

#[derive(Debug, Error)]
pub enum AetherError {
    #[error("policy validation error: {0}")]
    PolicyValidation(String),

    #[error("duplicate policy name: {0}")]
    DuplicatePolicyName(String),

    #[error("conflict resolution failed: policies {0} and {1} have same priority")]
    ConflictResolution(String, String),

    #[error("HCM error: {0}")]
    Hcm(String),

    #[error("audit chain integrity violation at sequence {0}")]
    AuditIntegrity(u64),

    #[error("adapter error: {0}")]
    Adapter(String),

    #[error("telemetry error: {0}")]
    Telemetry(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AetherError>;
