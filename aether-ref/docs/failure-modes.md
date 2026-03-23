# Aether Failure Modes

This document defines the exact behavior of the Aether engine under failure conditions.

## 1. All Links Degraded But Not Down

**Scenario**: Every known link has `availability: degraded`. No link is `up`.

**Behavior**: The evaluator treats `Degraded` as available. Degraded links are NOT excluded from selection. The tiebreak algorithm (`select_link`) filters only `Down` links.

**Decision output**: The first preferred link (if degraded) is selected normally. If no preferred link is degraded, `AnyAvailable` fallback selects the degraded link with lowest latency.

**If all links are Down**:
| Fallback Mode | Decision |
|---------------|----------|
| `any_available` | `selected_links: []` (empty — no link available) |
| `defer_to_routing` | `selected_links: []` (defers to underlying routing protocol) |
| `shed_via_equipment` | `selected_links: []` (equipment sheds traffic) |

In all cases, the decision is still logged to the audit chain with `justification: PolicyMatch` or `DefaultApplied`.

## 2. Telemetry Missing for >X Seconds

**Scenario**: One or more links have telemetry older than `staleness_threshold_secs` (default: 300 seconds).

**Behavior** depends on `EngineConfig.missing_telemetry_action`:

| Action | Behavior |
|--------|----------|
| `use_last_known` (default) | Proceed with the last received telemetry. No error, no warning. The stale data is visible in `decision.telemetry_snapshot` for audit purposes. |
| `mark_degraded` | Proceed with last-known data. A warning is logged via tracing. The decision proceeds normally but operators can detect degraded operation from logs. |
| `reject_evaluation` | Return `Err(AetherError::Telemetry("stale telemetry for links: ..."))`. No decision is produced. No audit entry is created. The caller must handle the error (e.g., retry after telemetry refresh, or fall back to a local default). |

**Configuration**:
```yaml
missing_telemetry_action: reject_evaluation
staleness_threshold_secs: 60
```

## 3. HCM Expires Mid-Decision Cycle

**Scenario**: HCM is active, the operator calls `evaluate()`, and HCM's time limit expires between the expiry check and the actual evaluation.

**Behavior**: This race condition **cannot occur**. The `AetherEngine::evaluate()` method takes `&mut self`, which in Rust means exclusive access. No other method can run concurrently on the same engine instance.

The evaluation sequence is:
1. `check_expiry()` — if HCM expired, deactivate and clear trigger
2. Build telemetry snapshot
3. Call `evaluate()` with current (post-expiry) trigger state
4. Log decision

Steps 1-4 are atomic with respect to the engine state. Between step 1 and step 3, no external mutation can occur because `&mut self` is held.

**If HCM expires during step 1**: The trigger `human_continuity_mode` is set to `false` before evaluation. HCM-triggered policies will not match. The decision reflects post-expiry state.

**If the clock advances between step 1 and step 3 such that HCM WOULD have expired**: The check at step 1 uses the monotonic clock at that instant. If the duration is within the limit at step 1, evaluation proceeds with HCM active. The next call to `evaluate()` will detect the expiry. The maximum window of "expired but still active" is the duration of a single `evaluate()` call (microseconds).
