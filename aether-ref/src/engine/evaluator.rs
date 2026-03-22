use std::collections::BTreeMap;

use crate::engine::conflict::resolve_conflicts;
use crate::engine::tiebreak::select_link;
use crate::error::{AetherError, Result};
use crate::types::decision::{Decision, JustificationCode, RuleRef};
use crate::types::hcm::HcmState;
use crate::types::link::{Availability, LinkId, TelemetrySnapshot};
use crate::types::policy::{
    ComparisonOp, Condition, FallbackMode, LinkStateConditions, Policy, PolicySet, StringCondition,
    TriggerBlock, TriggerCondition, TriggerValue,
};
use crate::types::traffic_class::TrafficClassLabel;

/// Pure, deterministic policy evaluation.
///
/// Given identical inputs, this function MUST produce identical outputs.
/// No randomness, no wall-clock reads, no mutable state.
pub fn evaluate(
    policy_set: &PolicySet,
    trigger_state: &BTreeMap<String, TriggerValue>,
    telemetry: &TelemetrySnapshot,
    traffic_class: &TrafficClassLabel,
    hcm_state: &HcmState,
    decision_id: String,
    timestamp: chrono::DateTime<chrono::Utc>,
) -> Result<Decision> {
    // 1. Filter policies by trigger state
    let active_policies: Vec<&Policy> = policy_set
        .policies
        .iter()
        .filter(|p| triggers_match(&p.triggers, trigger_state))
        .collect();

    // 2. Sort by priority and resolve conflicts
    let ordered = resolve_conflicts(active_policies, policy_set.conflict_resolution)?;

    // Detect missing telemetry for degraded justification
    let stale_links: Vec<LinkId> = telemetry
        .links
        .iter()
        .filter(|(_, s)| s.latency_ms.is_none() && s.capacity_mbps.is_none())
        .map(|(id, _)| id.clone())
        .collect();

    // 3. Evaluate first matching policy's rules
    for policy in &ordered {
        for (rule_index, rule) in policy.rules.iter().enumerate() {
            // Check traffic class match
            if !rule
                .match_block
                .traffic_class
                .matches(&traffic_class.label_id, &traffic_class.label_source)
            {
                continue;
            }

            // HCM scope enforcement: if HCM is active, only HCM-scoped labels
            // may be affected by HCM-triggered policies
            if hcm_state.active {
                let is_hcm_policy = policy.triggers.as_ref().map_or(false, |t| {
                    t.all_of.as_ref().map_or(false, |conditions| {
                        conditions.iter().any(|c| c.name == "human_continuity_mode")
                    }) || t.any_of.as_ref().map_or(false, |conditions| {
                        conditions.iter().any(|c| c.name == "human_continuity_mode")
                    })
                });

                if is_hcm_policy && !hcm_state.is_in_scope(&traffic_class.label_id) {
                    // HCM policy must not affect out-of-scope labels — skip this rule
                    tracing::warn!(
                        policy = %policy.name,
                        label_id = %traffic_class.label_id,
                        "HCM scope violation: policy skipped for out-of-scope label"
                    );
                    continue;
                }
            }

            // Check link state conditions (if specified)
            if let Some(link_conditions) = &rule.match_block.link_state {
                if !link_state_matches(link_conditions, telemetry) {
                    continue;
                }
            }

            // First match wins — resolve links via tiebreak
            let all_link_ids: Vec<LinkId> = telemetry.links.keys().cloned().collect();
            let selected = resolve_action(
                &rule.actions.link_selection.prefer,
                &rule.actions.link_selection.fallback,
                &all_link_ids,
                telemetry,
            );

            // Justification always reflects the matched policy rule.
            // Degraded telemetry is tracked separately via stale_links in the
            // telemetry_snapshot (links with missing metrics), not by overwriting
            // the primary justification.
            let justification = JustificationCode::PolicyMatch {
                policy: policy.name.clone(),
                rule_index,
            };

            return Ok(Decision {
                decision_id,
                timestamp,
                policy_set_version: policy_set.policy_set_version.clone(),
                traffic_class: traffic_class.clone(),
                trigger_snapshot: trigger_state.clone(),
                rule_matched: Some(RuleRef {
                    policy_name: policy.name.clone(),
                    rule_index,
                }),
                action_issued: rule.actions.clone(),
                selected_links: selected,
                justification,
                telemetry_snapshot: telemetry.links.clone(),
            });
        }
    }

    // 4. No rule matched — apply defaults
    let all_link_ids: Vec<LinkId> = telemetry.links.keys().cloned().collect();
    let selected = resolve_action(
        &policy_set.defaults.no_match_action.link_selection.prefer,
        &policy_set.defaults.no_match_action.link_selection.fallback,
        &all_link_ids,
        telemetry,
    );

    Ok(Decision {
        decision_id,
        timestamp,
        policy_set_version: policy_set.policy_set_version.clone(),
        traffic_class: traffic_class.clone(),
        trigger_snapshot: trigger_state.clone(),
        rule_matched: None,
        action_issued: policy_set.defaults.no_match_action.clone(),
        selected_links: selected,
        justification: JustificationCode::DefaultApplied,
        telemetry_snapshot: telemetry.links.clone(),
    })
}

/// Validate a policy set for structural correctness.
/// In strict mode, rejects empty rules lists and warns on potential issues.
pub fn validate_policy_set(policy_set: &PolicySet) -> Result<()> {
    use crate::types::policy::ValidationMode;

    let mut seen = std::collections::HashSet::new();
    for policy in &policy_set.policies {
        if !seen.insert(&policy.name) {
            return Err(AetherError::DuplicatePolicyName(policy.name.clone()));
        }

        // In strict mode, reject policies with no rules
        if policy_set.validation_mode == ValidationMode::Strict && policy.rules.is_empty() {
            return Err(AetherError::PolicyValidation(format!(
                "policy '{}' has an empty rules list",
                policy.name
            )));
        }

        // In permissive mode, warn but don't error
        if policy_set.validation_mode == ValidationMode::Permissive && policy.rules.is_empty() {
            tracing::warn!(policy = %policy.name, "policy has an empty rules list");
        }
    }
    Ok(())
}

// --- Internal helpers ---

/// Check if a policy's triggers are satisfied by the current trigger state.
fn triggers_match(
    triggers: &Option<TriggerBlock>,
    state: &BTreeMap<String, TriggerValue>,
) -> bool {
    let block = match triggers {
        Some(b) => b,
        None => return true, // No triggers = always active
    };

    // If block is present but both fields are None, treat as always-active
    // (per spec: "Default (if unspecified) = all_of" — empty block = no conditions)
    if block.all_of.is_none() && block.any_of.is_none() {
        return true;
    }

    // all_of: every condition must be true (empty list = vacuously true)
    if let Some(all) = &block.all_of {
        if !all.iter().all(|c| condition_matches(c, state)) {
            return false;
        }
    }

    // any_of: at least one condition must be true
    // Empty list = no condition can be true = policy never active
    if let Some(any) = &block.any_of {
        if any.is_empty() {
            tracing::warn!("trigger block has empty any_of list — policy will never be active");
            return false;
        }
        if !any.iter().any(|c| condition_matches(c, state)) {
            return false;
        }
    }

    true
}

/// Evaluate a single trigger condition against the current state.
fn condition_matches(
    condition: &TriggerCondition,
    state: &BTreeMap<String, TriggerValue>,
) -> bool {
    let actual = match state.get(&condition.name) {
        Some(v) => v,
        None => return false, // Missing trigger value = condition not met
    };

    compare_values(actual, &condition.op, &condition.value)
}

/// Compare two TriggerValues using the given operator.
fn compare_values(actual: &TriggerValue, op: &ComparisonOp, expected: &TriggerValue) -> bool {
    match (actual, expected) {
        (TriggerValue::Bool(a), TriggerValue::Bool(b)) => match op {
            ComparisonOp::Eq => a == b,
            ComparisonOp::Ne => a != b,
            _ => false,
        },
        (TriggerValue::Str(a), TriggerValue::Str(b)) => match op {
            ComparisonOp::Eq => a == b,
            ComparisonOp::Ne => a != b,
            ComparisonOp::Lt => a < b,
            ComparisonOp::Le => a <= b,
            ComparisonOp::Gt => a > b,
            ComparisonOp::Ge => a >= b,
        },
        (a, b) => {
            // Numeric comparison
            match (a.as_f64(), b.as_f64()) {
                (Some(av), Some(bv)) => match op {
                    ComparisonOp::Eq => (av - bv).abs() < f64::EPSILON,
                    ComparisonOp::Ne => (av - bv).abs() >= f64::EPSILON,
                    ComparisonOp::Lt => av < bv,
                    ComparisonOp::Le => av <= bv,
                    ComparisonOp::Gt => av > bv,
                    ComparisonOp::Ge => av >= bv,
                },
                _ => false,
            }
        }
    }
}

/// Check if all link state conditions are satisfied.
fn link_state_matches(
    conditions: &BTreeMap<String, LinkStateConditions>,
    telemetry: &TelemetrySnapshot,
) -> bool {
    for (link_name, conds) in conditions {
        let link_id = LinkId(link_name.clone());
        let state = match telemetry.links.get(&link_id) {
            Some(s) => s,
            None => return false, // Missing telemetry = condition not met
        };

        if let Some(ref cond) = conds.latency_ms {
            match state.latency_ms {
                Some(lat) => {
                    if !check_numeric_condition(lat, cond) {
                        return false;
                    }
                }
                None => return false, // Missing value = condition not met
            }
        }

        if let Some(ref cond) = conds.availability {
            let avail_str = match state.availability {
                Availability::Up => "up",
                Availability::Down => "down",
                Availability::Degraded => "degraded",
            };
            if !check_string_condition(avail_str, cond) {
                return false;
            }
        }

        if let Some(ref cond) = conds.capacity_mbps {
            match state.capacity_mbps {
                Some(cap) => {
                    if !check_numeric_condition(cap, cond) {
                        return false;
                    }
                }
                None => return false,
            }
        }
    }
    true
}

fn check_numeric_condition(actual: f64, condition: &Condition) -> bool {
    let expected = condition.value;
    match condition.op {
        ComparisonOp::Eq => (actual - expected).abs() < f64::EPSILON,
        ComparisonOp::Ne => (actual - expected).abs() >= f64::EPSILON,
        ComparisonOp::Lt => actual < expected,
        ComparisonOp::Le => actual <= expected,
        ComparisonOp::Gt => actual > expected,
        ComparisonOp::Ge => actual >= expected,
    }
}

fn check_string_condition(actual: &str, condition: &StringCondition) -> bool {
    match condition.op {
        ComparisonOp::Eq => actual == condition.value,
        ComparisonOp::Ne => actual != condition.value,
        _ => false,
    }
}

/// Resolve a link selection action to a concrete set of selected links.
fn resolve_action(
    prefer: &[LinkId],
    fallback: &FallbackMode,
    all_links: &[LinkId],
    telemetry: &TelemetrySnapshot,
) -> Vec<LinkId> {
    // Try preferred links first via tie-breaking
    if let Some(selected) = select_link(prefer, all_links, telemetry) {
        return vec![selected];
    }

    // All preferred (or all) links are down
    match fallback {
        FallbackMode::AnyAvailable => {
            // Try any available link
            if let Some(selected) = select_link(&[], all_links, telemetry) {
                vec![selected]
            } else {
                vec![]
            }
        }
        FallbackMode::DeferToRouting => vec![],
        FallbackMode::ShedViaEquipment => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::link::{Availability, LinkState};
    use crate::types::policy::*;

    fn make_telemetry(
        entries: Vec<(&str, f64, Availability)>,
    ) -> TelemetrySnapshot {
        let mut links = BTreeMap::new();
        for (id, latency, avail) in entries {
            links.insert(
                LinkId(id.to_string()),
                LinkState {
                    link_id: LinkId(id.to_string()),
                    latency_ms: Some(latency),
                    jitter_ms: None,
                    availability: avail,
                    capacity_mbps: None,
                    timestamp: chrono::Utc::now(),
                    source_id: "test".to_string(),
                },
            );
        }
        TelemetrySnapshot { links }
    }

    fn make_simple_policy_set() -> PolicySet {
        PolicySet {
            policy_set_version: "test-v1".to_string(),
            policies: vec![
                Policy {
                    name: "critical-policy".to_string(),
                    version: "1.0".to_string(),
                    priority: 100,
                    triggers: None,
                    rules: vec![Rule {
                        match_block: MatchBlock {
                            traffic_class: TrafficClassMatch {
                                label_id: "critical".to_string(),
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
                },
                Policy {
                    name: "routine-policy".to_string(),
                    version: "1.0".to_string(),
                    priority: 50,
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
                                prefer: vec![LinkId::from("lte_01")],
                                fallback: FallbackMode::DeferToRouting,
                            },
                        },
                    }],
                },
            ],
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

    #[test]
    fn critical_traffic_matches_high_priority_policy() {
        let policy_set = make_simple_policy_set();
        let telemetry = make_telemetry(vec![
            ("leo_01", 50.0, Availability::Up),
            ("lte_01", 20.0, Availability::Up),
        ]);
        let triggers = BTreeMap::new();
        let tc = TrafficClassLabel {
            label_id: "critical".to_string(),
            label_source: "DSCP".to_string(),
        };

        let decision = evaluate(
            &policy_set,
            &triggers,
            &telemetry,
            &tc,
            &HcmState::default(),
            "test-001".to_string(),
            chrono::Utc::now(),
        )
        .unwrap();

        assert!(matches!(
            decision.justification,
            JustificationCode::PolicyMatch {
                ref policy,
                rule_index: 0,
            } if policy == "critical-policy"
        ));
    }

    #[test]
    fn routine_traffic_matches_lower_priority_wildcard() {
        let policy_set = make_simple_policy_set();
        let telemetry = make_telemetry(vec![
            ("leo_01", 50.0, Availability::Up),
            ("lte_01", 20.0, Availability::Up),
        ]);
        let triggers = BTreeMap::new();
        let tc = TrafficClassLabel {
            label_id: "routine".to_string(),
            label_source: "DSCP".to_string(),
        };

        let decision = evaluate(
            &policy_set,
            &triggers,
            &telemetry,
            &tc,
            &HcmState::default(),
            "test-002".to_string(),
            chrono::Utc::now(),
        )
        .unwrap();

        // "critical-policy" also has wildcard label_source, and it's higher priority,
        // but its label_id is "critical" not wildcard. So it won't match "routine".
        // "routine-policy" has wildcard for both, so it matches.
        assert!(matches!(
            decision.justification,
            JustificationCode::PolicyMatch {
                ref policy,
                rule_index: 0,
            } if policy == "routine-policy"
        ));
    }

    #[test]
    fn no_match_applies_default() {
        let mut policy_set = make_simple_policy_set();
        // Remove the wildcard policy
        policy_set.policies.retain(|p| p.name != "routine-policy");

        let telemetry = make_telemetry(vec![("leo_01", 50.0, Availability::Up)]);
        let triggers = BTreeMap::new();
        let tc = TrafficClassLabel {
            label_id: "unknown".to_string(),
            label_source: "test".to_string(),
        };

        let decision = evaluate(
            &policy_set,
            &triggers,
            &telemetry,
            &tc,
            &HcmState::default(),
            "test-003".to_string(),
            chrono::Utc::now(),
        )
        .unwrap();

        assert!(matches!(
            decision.justification,
            JustificationCode::DefaultApplied
        ));
    }

    #[test]
    fn trigger_filters_policy() {
        let mut policy_set = make_simple_policy_set();
        // Add trigger to critical policy: requires HCM active
        policy_set.policies[0].triggers = Some(TriggerBlock {
            all_of: Some(vec![TriggerCondition {
                name: "human_continuity_mode".to_string(),
                op: ComparisonOp::Eq,
                value: TriggerValue::Bool(true),
            }]),
            any_of: None,
        });

        let telemetry = make_telemetry(vec![
            ("leo_01", 50.0, Availability::Up),
            ("lte_01", 20.0, Availability::Up),
        ]);
        // HCM is NOT active in trigger state
        let triggers = BTreeMap::new();
        let tc = TrafficClassLabel {
            label_id: "critical".to_string(),
            label_source: "DSCP".to_string(),
        };

        let decision = evaluate(
            &policy_set,
            &triggers,
            &telemetry,
            &tc,
            &HcmState::default(),
            "test-004".to_string(),
            chrono::Utc::now(),
        )
        .unwrap();

        // Critical policy filtered out due to trigger, falls through to routine wildcard
        assert!(matches!(
            decision.justification,
            JustificationCode::PolicyMatch {
                ref policy,
                ..
            } if policy == "routine-policy"
        ));
    }

    #[test]
    fn trigger_activates_policy() {
        let mut policy_set = make_simple_policy_set();
        policy_set.policies[0].triggers = Some(TriggerBlock {
            all_of: Some(vec![TriggerCondition {
                name: "human_continuity_mode".to_string(),
                op: ComparisonOp::Eq,
                value: TriggerValue::Bool(true),
            }]),
            any_of: None,
        });

        let telemetry = make_telemetry(vec![
            ("leo_01", 50.0, Availability::Up),
            ("lte_01", 20.0, Availability::Up),
        ]);
        let mut triggers = BTreeMap::new();
        triggers.insert(
            "human_continuity_mode".to_string(),
            TriggerValue::Bool(true),
        );
        let tc = TrafficClassLabel {
            label_id: "critical".to_string(),
            label_source: "DSCP".to_string(),
        };

        let decision = evaluate(
            &policy_set,
            &triggers,
            &telemetry,
            &tc,
            &HcmState::default(),
            "test-005".to_string(),
            chrono::Utc::now(),
        )
        .unwrap();

        assert!(matches!(
            decision.justification,
            JustificationCode::PolicyMatch {
                ref policy,
                ..
            } if policy == "critical-policy"
        ));
    }

    #[test]
    fn link_state_condition_filtering() {
        let mut policy_set = make_simple_policy_set();
        // Require leo_01 latency < 40ms
        let mut link_conditions = BTreeMap::new();
        link_conditions.insert(
            "leo_01".to_string(),
            LinkStateConditions {
                latency_ms: Some(Condition {
                    op: ComparisonOp::Lt,
                    value: 40.0,
                }),
                availability: None,
                capacity_mbps: None,
            },
        );
        policy_set.policies[0].rules[0].match_block.link_state = Some(link_conditions);

        // leo_01 has 50ms latency — condition NOT met
        let telemetry = make_telemetry(vec![
            ("leo_01", 50.0, Availability::Up),
            ("lte_01", 20.0, Availability::Up),
        ]);
        let triggers = BTreeMap::new();
        let tc = TrafficClassLabel {
            label_id: "critical".to_string(),
            label_source: "DSCP".to_string(),
        };

        let decision = evaluate(
            &policy_set,
            &triggers,
            &telemetry,
            &tc,
            &HcmState::default(),
            "test-006".to_string(),
            chrono::Utc::now(),
        )
        .unwrap();

        // Link state condition failed, falls through to routine wildcard
        assert!(matches!(
            decision.justification,
            JustificationCode::PolicyMatch {
                ref policy,
                ..
            } if policy == "routine-policy"
        ));
    }

    #[test]
    fn deterministic_replay() {
        let policy_set = make_simple_policy_set();
        let telemetry = make_telemetry(vec![
            ("leo_01", 50.0, Availability::Up),
            ("lte_01", 20.0, Availability::Up),
        ]);
        let triggers = BTreeMap::new();
        let tc = TrafficClassLabel {
            label_id: "critical".to_string(),
            label_source: "DSCP".to_string(),
        };
        let ts = chrono::Utc::now();

        let d1 = evaluate(
            &policy_set,
            &triggers,
            &telemetry,
            &tc,
            &HcmState::default(),
            "replay-1".to_string(),
            ts,
        )
        .unwrap();

        let d2 = evaluate(
            &policy_set,
            &triggers,
            &telemetry,
            &tc,
            &HcmState::default(),
            "replay-1".to_string(),
            ts,
        )
        .unwrap();

        // Same inputs → same outputs (determinism)
        assert_eq!(
            serde_json::to_string(&d1.justification).unwrap(),
            serde_json::to_string(&d2.justification).unwrap()
        );
        assert_eq!(
            serde_json::to_string(&d1.action_issued).unwrap(),
            serde_json::to_string(&d2.action_issued).unwrap()
        );
    }

    #[test]
    fn validate_rejects_duplicate_names() {
        let mut policy_set = make_simple_policy_set();
        policy_set.policies[1].name = "critical-policy".to_string();
        assert!(validate_policy_set(&policy_set).is_err());
    }
}
