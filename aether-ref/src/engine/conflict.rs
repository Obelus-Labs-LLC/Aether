use crate::error::{AetherError, Result};
use crate::types::policy::{ConflictResolution, Policy};

/// Sort and resolve conflicts among policies that passed trigger filtering.
/// Returns policies in evaluation order (highest priority first, conflicts resolved).
pub fn resolve_conflicts(
    mut policies: Vec<&Policy>,
    mode: ConflictResolution,
) -> Result<Vec<&Policy>> {
    // Sort by priority descending
    policies.sort_by(|a, b| b.priority.cmp(&a.priority));

    // Check for same-priority conflicts and resolve them
    let mut i = 0;
    while i + 1 < policies.len() {
        if policies[i].priority == policies[i + 1].priority {
            match mode {
                ConflictResolution::LexicographicPolicyName => {
                    // Sort the tied group lexicographically by name
                    let priority = policies[i].priority;
                    let start = i;
                    let mut end = i + 1;
                    while end < policies.len() && policies[end].priority == priority {
                        end += 1;
                    }
                    policies[start..end].sort_by(|a, b| a.name.cmp(&b.name));
                    i = end;
                }
                ConflictResolution::RequireUnique => {
                    return Err(AetherError::ConflictResolution(
                        policies[i].name.clone(),
                        policies[i + 1].name.clone(),
                    ));
                }
                ConflictResolution::FirstLoaded => {
                    // Already in load order within same priority; no reordering needed
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
    }

    Ok(policies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::policy::Policy;

    fn make_policy(name: &str, priority: i64) -> Policy {
        Policy {
            name: name.to_string(),
            version: "1.0".to_string(),
            priority,
            triggers: None,
            rules: vec![],
        }
    }

    #[test]
    fn higher_priority_first() {
        let p1 = make_policy("low", 10);
        let p2 = make_policy("high", 100);
        let policies = vec![&p1, &p2];
        let result = resolve_conflicts(policies, ConflictResolution::LexicographicPolicyName).unwrap();
        assert_eq!(result[0].name, "high");
        assert_eq!(result[1].name, "low");
    }

    #[test]
    fn lexicographic_tiebreak() {
        let p1 = make_policy("beta", 50);
        let p2 = make_policy("alpha", 50);
        let policies = vec![&p1, &p2];
        let result = resolve_conflicts(policies, ConflictResolution::LexicographicPolicyName).unwrap();
        assert_eq!(result[0].name, "alpha");
        assert_eq!(result[1].name, "beta");
    }

    #[test]
    fn require_unique_rejects_conflict() {
        let p1 = make_policy("a", 50);
        let p2 = make_policy("b", 50);
        let policies = vec![&p1, &p2];
        let result = resolve_conflicts(policies, ConflictResolution::RequireUnique);
        assert!(result.is_err());
    }
}
