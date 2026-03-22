use clap::{Parser, Subcommand};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::path::PathBuf;

use aether_ref::engine::evaluator::{evaluate, validate_policy_set};
use aether_ref::types::hcm::HcmState;
use aether_ref::types::link::TelemetrySnapshot;
use aether_ref::types::policy::{PolicySet, TriggerValue};
use aether_ref::types::traffic_class::TrafficClassLabel;

type HmacSha256 = Hmac<Sha256>;

#[derive(Parser)]
#[command(name = "aether")]
#[command(about = "Aether reference implementation — policy-driven uplink arbitration")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate a traffic class against a policy set
    Evaluate {
        #[arg(long)]
        policy: PathBuf,
        #[arg(long)]
        telemetry: PathBuf,
        #[arg(long)]
        label_id: String,
        #[arg(long, default_value = "*")]
        label_source: String,
    },

    /// Validate a policy set for structural correctness
    Validate {
        #[arg(long)]
        policy: PathBuf,
    },

    /// Verify an audit log's HMAC chain integrity
    AuditVerify {
        /// Path to audit log JSON file
        #[arg(long)]
        log: PathBuf,

        /// HMAC key file path (binary). Use --key-hex for hex string.
        #[arg(long, conflicts_with = "key_hex")]
        key_file: Option<PathBuf>,

        /// HMAC key as hex string. Prefer --key-file or AETHER_AUDIT_KEY env var.
        #[arg(long, conflicts_with = "key_file")]
        key_hex: Option<String>,
    },
}

/// Resolve HMAC key from --key-file, --key-hex, or AETHER_AUDIT_KEY env var.
fn resolve_audit_key(key_file: Option<PathBuf>, key_hex: Option<String>) -> Vec<u8> {
    if let Some(path) = key_file {
        return std::fs::read(&path)
            .unwrap_or_else(|e| panic!("failed to read key file: {e}"));
    }
    if let Some(hex_str) = key_hex {
        return hex::decode(&hex_str)
            .unwrap_or_else(|e| panic!("invalid hex key: {e}"));
    }
    if let Ok(env_key) = std::env::var("AETHER_AUDIT_KEY") {
        return hex::decode(&env_key)
            .unwrap_or_else(|e| panic!("invalid hex in AETHER_AUDIT_KEY: {e}"));
    }
    eprintln!("Error: provide HMAC key via --key-file, --key-hex, or AETHER_AUDIT_KEY env var");
    std::process::exit(1);
}

fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Evaluate {
            policy,
            telemetry,
            label_id,
            label_source,
        } => {
            let policy_content = std::fs::read_to_string(&policy)
                .unwrap_or_else(|e| panic!("failed to read policy file: {e}"));

            let policy_set: PolicySet = if policy
                .extension()
                .map_or(false, |ext| ext == "yaml" || ext == "yml")
            {
                serde_yaml::from_str(&policy_content)
                    .unwrap_or_else(|e| panic!("failed to parse YAML policy: {e}"))
            } else {
                serde_json::from_str(&policy_content)
                    .unwrap_or_else(|e| panic!("failed to parse JSON policy: {e}"))
            };

            let telemetry_content = std::fs::read_to_string(&telemetry)
                .unwrap_or_else(|e| panic!("failed to read telemetry file: {e}"));
            let telemetry_snapshot: TelemetrySnapshot =
                serde_json::from_str(&telemetry_content)
                    .unwrap_or_else(|e| panic!("failed to parse telemetry: {e}"));

            let tc = TrafficClassLabel {
                label_id,
                label_source,
            };

            let triggers = std::collections::BTreeMap::<String, TriggerValue>::new();
            let decision_id = uuid::Uuid::new_v4().to_string();

            match evaluate(
                &policy_set,
                &triggers,
                &telemetry_snapshot,
                &tc,
                &HcmState::default(),
                decision_id,
                chrono::Utc::now(),
            ) {
                Ok(decision) => {
                    let json = serde_json::to_string_pretty(&decision).unwrap();
                    println!("{json}");
                }
                Err(e) => {
                    eprintln!("Evaluation error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Validate { policy } => {
            let policy_content = std::fs::read_to_string(&policy)
                .unwrap_or_else(|e| panic!("failed to read policy file: {e}"));

            let policy_set: PolicySet = if policy
                .extension()
                .map_or(false, |ext| ext == "yaml" || ext == "yml")
            {
                serde_yaml::from_str(&policy_content)
                    .unwrap_or_else(|e| panic!("failed to parse YAML policy: {e}"))
            } else {
                serde_json::from_str(&policy_content)
                    .unwrap_or_else(|e| panic!("failed to parse JSON policy: {e}"))
            };

            match validate_policy_set(&policy_set) {
                Ok(()) => println!("Policy set is valid."),
                Err(e) => {
                    eprintln!("Validation error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::AuditVerify {
            log,
            key_file,
            key_hex,
        } => {
            let key_bytes = resolve_audit_key(key_file, key_hex);

            let log_content = std::fs::read_to_string(&log)
                .unwrap_or_else(|e| panic!("failed to read audit log: {e}"));

            let entries: Vec<aether_ref::types::audit::AuditEntry> =
                serde_json::from_str(&log_content)
                    .unwrap_or_else(|e| panic!("failed to parse audit log: {e}"));

            // BUG-4 fix: Verify the STORED HMACs in the file, not a replayed chain
            let initial_hmac =
                "0000000000000000000000000000000000000000000000000000000000000000";
            let mut expected_prev = initial_hmac.to_string();

            for entry in &entries {
                // Verify chain linkage
                if entry.previous_hmac != expected_prev {
                    eprintln!(
                        "Chain linkage broken at sequence {}: expected previous_hmac={}, got={}",
                        entry.sequence, expected_prev, entry.previous_hmac
                    );
                    std::process::exit(1);
                }

                // Recompute HMAC and compare against stored value
                let decision_json = serde_json::to_string(&entry.decision)
                    .unwrap_or_else(|e| panic!("failed to serialize decision: {e}"));

                let mut mac = HmacSha256::new_from_slice(&key_bytes)
                    .expect("HMAC accepts any key length");
                mac.update(decision_json.as_bytes());
                mac.update(entry.previous_hmac.as_bytes());
                let computed = hex::encode(mac.finalize().into_bytes());

                if computed != entry.hmac {
                    eprintln!(
                        "HMAC mismatch at sequence {}: computed={}, stored={}",
                        entry.sequence, computed, entry.hmac
                    );
                    std::process::exit(1);
                }

                expected_prev = entry.hmac.clone();
            }

            println!(
                "Audit chain integrity verified. {} entries.",
                entries.len()
            );
        }
    }
}
