# Aether Policy Schema

**Version:** 1.0  
**Status:** Normative  
**Scope:** Defines the policy language, evaluation semantics, and action model for Aether

---

## 1. Purpose

The Aether Policy Schema defines **how decisions are expressed**, **how they are evaluated**, and **how actions are issued**.

This schema is designed to be:

- Deterministic
- Auditable
- Vendor-neutral
- Resistant to ambiguous or adversarial interpretation

Policy is the **only semantic decision layer** in Aether.  
No hidden heuristics, inference, or vendor logic are permitted outside this schema.

---

## 2. Policy Set Model

### 2.1 Policy Set

A **policy set** is the complete collection of active policies governing an Aether deployment.

A policy set:

- MAY consist of multiple policy documents
- MUST be evaluated atomically
- MUST have a unique `policy_set_version` identifier (hash or semantic version)

All decisions reference the active policy set version in audit logs.

---

### 2.2 Policy Set Constraints (Normative)

- Policy names MUST be globally unique within the policy set
- Exactly **one** `defaults` configuration SHALL exist
- Policy set changes MUST trigger re-evaluation
- Policy set activation MUST be atomic

---

## 3. Policy Structure

### 3.1 Policy Metadata

```yaml
policy:
  name: "example-policy"
  version: "1.0"
  priority: 100
````

**Fields:**

* `name` (string, required): Unique policy identifier
* `version` (string, required): Operator-defined version
* `priority` (integer, required): Higher value = higher precedence

---

### 3.2 Conflict Resolution

Policies with identical priority and overlapping match conditions MUST be resolved deterministically.

```yaml
policy:
  conflict_resolution:
    same_priority: "lexicographic_policy_name"
```

**Allowed values:**

* `lexicographic_policy_name` (default)
* `require_unique`
* `first_loaded`

If not specified, `lexicographic_policy_name` SHALL be used.

---

## 4. Matching Semantics

### 4.1 Traffic Class Matching

Traffic classification is **external** to Aether.

```yaml
match:
  traffic_class:
    label_id: "emergency_medical"
    label_source: "policy_engine"
```

**Requirements (Normative):**

* `label_id` AND `label_source` MUST be present
* Labels are opaque; Aether does not interpret semantics
* Wildcard matching MUST be explicit

```yaml
label_source: "*"
```

Wildcard matches have **lower specificity** than exact matches.

---

### 4.2 Trigger Matching

Triggers are externally-asserted conditions.

```yaml
triggers:
  all_of:
    - {name: "human_continuity_mode", op: "==", value: true}
    - {name: "link_failure_count", op: ">=", value: 2}
```

**Rules:**

* `all_of` = logical AND
* `any_of` = logical OR
* Default (if unspecified) = `all_of`
* Trigger evaluation is deterministic

**Prohibited:**

* Stringly-typed triggers
* Implicit operator parsing

---

### 4.3 Link State Matching

```yaml
match:
  link_state:
    uplink_leo_01:
      latency_ms: {op: "<", value: 100}
      availability: {op: "==", value: "up"}
```

**Operator constraints:**

* No implicit type coercion
* Missing values are treated as worse than known values
* Attribute types MUST be enforced

---

## 5. Actions

### 5.1 Link Selection

```yaml
actions:
  link_selection:
    prefer: ["uplink_leo_01", "uplink_mss_01"]
    fallback: "any_available"
```

**Allowed fallback values:**

* `any_available`
* `defer_to_routing`
* `shed_via_equipment`

---

### 5.2 Traffic Shedding (Constrained)

`shed_via_equipment`:

* MUST be executed by existing equipment
* MUST be logged
* MUST be time-bounded where possible
* MUST NOT be executed by Aether itself

---

## 6. Evaluation Semantics

### 6.1 Policy Evaluation Order

1. Filter policies by trigger state
2. Sort by priority (descending)
3. Resolve conflicts deterministically
4. Evaluate first matching policy

---

### 6.2 Rule Evaluation Order

Rules within a policy:

* Are evaluated in listed order
* First match wins
* Remaining rules are NOT evaluated

Reordering rules changes behavior and MUST increment version.

---

### 6.3 Tie-Breaking

When multiple links satisfy constraints:

1. Explicit `prefer` order
2. Lowest latency (known values only)
3. Lexicographic link ID

Missing telemetry sorts last.

---

## 7. Defaults

Exactly one defaults block MUST exist.

```yaml
defaults:
  no_match_action:
    link_selection:
      fallback: "defer_to_routing"
```

Defaults MUST NOT introduce new logic.

---

## 8. Validation Modes

### 8.1 Strict Mode

* Unknown fields → rejection
* Recommended for production

### 8.2 Permissive Mode

* Unknown fields ignored
* Warnings MUST be logged
* Evaluation behavior MUST NOT change

Mode MUST be explicitly configured and logged.

---

## 9. Determinism Requirements

Given identical:

* Policy set
* Trigger state
* Link state
* Memory state

The implementation MUST produce:

* Identical decisions
* Identical justification codes
* Identical commands

---

## 10. Auditability

Every decision MUST log:

* Policy set version
* Trigger snapshot
* Rule matched
* Action issued
* Justification code

Audit logs MUST be tamper-evident.

---

## 11. Forbidden Behavior

❌ Payload inspection
❌ Per-flow state
❌ Hidden heuristics
❌ Non-deterministic decisions
❌ Adapter-side policy logic

Violations invalidate conformance.

---

## 12. Conformance

An implementation conforms to this schema **only if**:

* All rules above are satisfied
* No forbidden behaviors exist
* Determinism is provable

---

