use std::collections::BTreeMap;

use crate::audit::logger::AuditLogger;
use crate::engine::evaluator::{evaluate, validate_policy_set};
use crate::error::Result;
use crate::hcm::activation::HcmManager;
use crate::hcm::clock::MonotonicClock;
use crate::state::snapshot::EngineSnapshot;
use crate::telemetry::ingestion::TelemetryStore;
use crate::telemetry::memory::ExperienceMemory;
use crate::types::decision::Decision;
use crate::types::hcm::HcmActivation;
use crate::types::policy::{PolicySet, TriggerValue};
use crate::types::traffic_class::TrafficClassLabel;

/// The Aether engine — northbound interface for operators.
pub struct AetherEngine<C: MonotonicClock> {
    policy_set: Option<PolicySet>,
    trigger_state: BTreeMap<String, TriggerValue>,
    telemetry: TelemetryStore,
    memory: ExperienceMemory,
    hcm: HcmManager<C>,
    audit: AuditLogger,
}

impl<C: MonotonicClock> AetherEngine<C> {
    pub fn new(clock: C, audit_key: Vec<u8>, memory_capacity: usize) -> Self {
        Self {
            policy_set: None,
            trigger_state: BTreeMap::new(),
            telemetry: TelemetryStore::new(std::time::Duration::from_secs(300)),
            memory: ExperienceMemory::new(memory_capacity),
            hcm: HcmManager::new(clock),
            audit: AuditLogger::new(audit_key),
        }
    }

    // --- Policy Management ---

    /// Load a policy set. Validates and replaces the current set atomically.
    pub fn load_policy_set(&mut self, policy_set: PolicySet) -> Result<()> {
        validate_policy_set(&policy_set)?;
        self.policy_set = Some(policy_set);
        Ok(())
    }

    pub fn policy_set(&self) -> Option<&PolicySet> {
        self.policy_set.as_ref()
    }

    // --- Trigger Management ---

    pub fn set_trigger(&mut self, name: String, value: TriggerValue) {
        self.trigger_state.insert(name, value);
    }

    pub fn clear_trigger(&mut self, name: &str) {
        self.trigger_state.remove(name);
    }

    pub fn trigger_state(&self) -> &BTreeMap<String, TriggerValue> {
        &self.trigger_state
    }

    // --- HCM ---

    /// Activate Human Continuity Mode.
    pub fn activate_hcm(&mut self, activation: HcmActivation) -> Result<()> {
        self.hcm.activate(activation)?;
        self.set_trigger(
            "human_continuity_mode".to_string(),
            TriggerValue::Bool(true),
        );
        Ok(())
    }

    /// Deactivate Human Continuity Mode.
    pub fn deactivate_hcm(&mut self) -> Result<()> {
        self.hcm.deactivate()?;
        self.set_trigger(
            "human_continuity_mode".to_string(),
            TriggerValue::Bool(false),
        );
        Ok(())
    }

    pub fn hcm_state(&self) -> &crate::types::hcm::HcmState {
        self.hcm.state()
    }

    // --- Telemetry ---

    pub fn telemetry_store(&self) -> &TelemetryStore {
        &self.telemetry
    }

    pub fn telemetry_store_mut(&mut self) -> &mut TelemetryStore {
        &mut self.telemetry
    }

    // --- Memory ---

    pub fn memory(&self) -> &ExperienceMemory {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut ExperienceMemory {
        &mut self.memory
    }

    pub fn reset_memory(&mut self) {
        self.memory.reset();
    }

    // --- Evaluation ---

    /// Evaluate a traffic class against the current policy set.
    /// This is the core operation — deterministic given identical inputs.
    pub fn evaluate(&mut self, traffic_class: &TrafficClassLabel) -> Result<Decision> {
        let policy_set = self
            .policy_set
            .as_ref()
            .ok_or_else(|| crate::error::AetherError::PolicyValidation("no policy set loaded".to_string()))?;

        // BUG-3 fix: Check HCM expiry and sync trigger state
        let was_active = self.hcm.state().active;
        self.hcm.check_expiry();
        if was_active && !self.hcm.state().active {
            // HCM just expired — clear the trigger
            self.trigger_state.insert(
                "human_continuity_mode".to_string(),
                TriggerValue::Bool(false),
            );
            tracing::info!("HCM expired, trigger cleared");
        }

        let telemetry_snapshot = self.telemetry.snapshot();
        let decision_id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now();

        let decision = evaluate(
            policy_set,
            &self.trigger_state,
            &telemetry_snapshot,
            traffic_class,
            self.hcm.state(),
            decision_id,
            timestamp,
        )?;

        // Log the decision (tamper-evident)
        self.audit.log(decision.clone())?;

        Ok(decision)
    }

    // --- Audit ---

    pub fn audit_log(&self) -> &AuditLogger {
        &self.audit
    }

    pub fn verify_audit_chain(&self) -> std::result::Result<(), u64> {
        self.audit.verify()
    }

    // --- State Snapshot ---

    /// Export full engine state for replay/audit. Returns None if no policy set loaded.
    pub fn export_snapshot(&self) -> Option<EngineSnapshot> {
        let policy_set = self.policy_set.clone()?;
        Some(EngineSnapshot {
            policy_set,
            trigger_state: self.trigger_state.clone(),
            telemetry_snapshot: self.telemetry.snapshot(),
            experience_memory: self.memory.export(),
            hcm_state: self.hcm.state().export(),
            audit_chain_head: self.audit.head_hmac().to_string(),
            audit_sequence: self.audit.sequence(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hcm::clock::MockClock;
    use crate::types::hcm::ActorType;
    use crate::types::link::{Availability, LinkId, LinkState, TelemetryRecord};
    use crate::types::policy::*;
    use std::time::Duration;

    fn make_engine() -> AetherEngine<MockClock> {
        AetherEngine::new(MockClock::new(), b"test-key".to_vec(), 100)
    }

    fn make_policy_set() -> PolicySet {
        PolicySet {
            policy_set_version: "test-v1".to_string(),
            policies: vec![Policy {
                name: "test-policy".to_string(),
                version: "1.0".to_string(),
                priority: 100,
                triggers: None,
                rules: vec![Rule {
                    match_block: MatchBlock {
                        traffic_class: TrafficClassMatch {
                            label_id: "*".to_string(),
                            label_source: "*".to_string(),
                        },
                        link_state: None,
                    },
                    actions: ActionBlock {
                        link_selection: LinkSelection {
                            prefer: vec![LinkId::from("leo_01")],
                            fallback: FallbackMode::AnyAvailable,
                        },
                    },
                }],
            }],
            defaults: Defaults {
                no_match_action: ActionBlock {
                    link_selection: LinkSelection {
                        prefer: vec![],
                        fallback: FallbackMode::DeferToRouting,
                    },
                },
            },
            conflict_resolution: ConflictResolution::LexicographicPolicyName,
            validation_mode: ValidationMode::Strict,
        }
    }

    fn ingest_test_telemetry(engine: &mut AetherEngine<MockClock>) {
        let now = chrono::Utc::now();
        for (id, latency) in [("leo_01", 50.0), ("lte_01", 20.0)] {
            engine.telemetry_store_mut().ingest(TelemetryRecord {
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
            });
        }
    }

    #[test]
    fn full_engine_evaluation() {
        let mut engine = make_engine();
        engine.load_policy_set(make_policy_set()).unwrap();
        ingest_test_telemetry(&mut engine);

        let tc = TrafficClassLabel {
            label_id: "test".to_string(),
            label_source: "DSCP".to_string(),
        };

        let decision = engine.evaluate(&tc).unwrap();
        assert!(decision.rule_matched.is_some());
        // BUG-1 fix: selected_links should contain the resolved link
        assert!(!decision.selected_links.is_empty());

        assert_eq!(engine.audit_log().entries().len(), 1);
        assert!(engine.verify_audit_chain().is_ok());
    }

    #[test]
    fn trigger_management() {
        let mut engine = make_engine();
        engine.set_trigger("test_trigger".to_string(), TriggerValue::Bool(true));
        assert_eq!(
            engine.trigger_state().get("test_trigger"),
            Some(&TriggerValue::Bool(true))
        );
        engine.clear_trigger("test_trigger");
        assert!(engine.trigger_state().get("test_trigger").is_none());
    }

    #[test]
    fn snapshot_export() {
        let mut engine = make_engine();
        engine.load_policy_set(make_policy_set()).unwrap();
        ingest_test_telemetry(&mut engine);

        let snapshot = engine.export_snapshot().unwrap();
        assert_eq!(snapshot.policy_set.policy_set_version, "test-v1");
        assert_eq!(snapshot.telemetry_snapshot.links.len(), 2);
    }

    #[test]
    fn snapshot_export_returns_none_without_policy() {
        let engine = make_engine();
        assert!(engine.export_snapshot().is_none());
    }

    #[test]
    fn hcm_expiry_clears_trigger() {
        let mut engine = make_engine();
        engine.load_policy_set(make_policy_set()).unwrap();
        ingest_test_telemetry(&mut engine);

        let activation = HcmActivation {
            event_type: "hcm_activation".to_string(),
            event_id: "evt-1".to_string(),
            actor_id: "op-1".to_string(),
            actor_type: ActorType::HumanOperator,
            timestamp: chrono::Utc::now(),
            reason: "test".to_string(),
            scope: vec!["emergency".to_string()],
            authorization_method: "manual".to_string(),
            max_duration_seconds: 10,
            max_total_duration_seconds: 100,
            allow_renewal: false,
        };
        engine.activate_hcm(activation).unwrap();
        assert_eq!(
            engine.trigger_state().get("human_continuity_mode"),
            Some(&TriggerValue::Bool(true))
        );

        // Advance clock past expiry
        engine.hcm.clock.advance(Duration::from_secs(11));

        // Evaluate triggers expiry check
        let tc = TrafficClassLabel {
            label_id: "test".to_string(),
            label_source: "DSCP".to_string(),
        };
        let _ = engine.evaluate(&tc).unwrap();

        // Trigger should now be false
        assert_eq!(
            engine.trigger_state().get("human_continuity_mode"),
            Some(&TriggerValue::Bool(false))
        );
        assert!(!engine.hcm_state().active);
    }
}
