# Human Continuity Mode (HCM)

**Version:** 1.0  
**Status:** Normative  
**Last Updated:** 2024-12-23

---

## 1. Overview

Human Continuity Mode (HCM) is a strictly bounded emergency operating mode that prioritizes predefined traffic classes during disasters, infrastructure failures, or humanitarian crises.

HCM is designed to preserve **human safety, medical response, and public continuity** without introducing autonomous inference, payload inspection, or centralized override authority.

---

## 1.1 Design Principles (Normative)

1. **Explicit activation:**  
   HCM MUST be activated only via externally asserted events.

2. **Time-bounded:**  
   HCM SHALL operate for a finite duration with enforced expiration.

3. **Auditable:**  
   All activations, renewals, decisions, and exits MUST be logged.

4. **Scope-limited:**  
   HCM affects only explicitly declared traffic classes.

5. **Deterministic:**  
   HCM behavior MUST be deterministic and policy-driven.

6. **Reversible:**  
   Exit behavior MUST be predictable and policy-defined.

---

## 1.2 What HCM Is NOT (Normative)

HCM is NOT:

- A permanent operating mode
- An autonomous emergency detector
- A payload-inspecting system
- A centralized override authority
- A mechanism that suspends policy evaluation
- A traffic classification engine

---

## 2. Activation Model

### 2.1 Activation as Trigger State (Normative)

HCM activation is represented as an **externally asserted activation event** that sets the trigger state:

```

human_continuity_mode = true

```

Policy evaluation consumes this trigger state according to the Policy Schema.

Activation events are **separate from policy documents** and carry authorization, scope, and time bounds.

---

### 2.2 HCM Activation Event Schema (Normative)

An HCM activation MUST include the following fields:

- `event_type`: `"hcm_activation"`
- `event_id`: Unique identifier
- `actor_id`: Identifier of human operator or system
- `actor_type`: `human_operator | automated_system | external_authority`
- `timestamp`: ISO 8601 wall-clock time
- `reason`: Human-readable justification
- `scope`: List of traffic class `label_id` values affected
- `authorization_method`: How activation was authorized
- `max_duration_seconds`: Duration before expiration

---

### 2.3 Single Active HCM Constraint (Normative)

At most **one HCM activation** SHALL be active at any time.

If a new activation is attempted while HCM is active:

- The attempt MUST be treated as a **renewal**
- The existing activation is extended per renewal rules
- The event MUST be logged as a renewal, not a new activation

Concurrent HCM activations are **forbidden**.

---

## 3. Time Bounds and Renewal

### 3.1 Default Time Limits (Normative)

Unless otherwise specified:

- **Default max duration:** 4 hours (14,400 seconds)
- **Default renewals:** Not allowed
- **Maximum cumulative duration:** 72 hours (259,200 seconds)

These limits MUST be enforced.

---

### 3.2 Renewal Semantics (Normative)

If renewals are permitted:

- Each renewal MUST be explicitly authorized
- Each renewal MUST be logged
- Cumulative duration MUST NOT exceed `max_total_duration_seconds`
- Renewal does NOT reset audit history

Renewal without new authorization is forbidden.

---

### 3.3 Time Semantics (Normative)

- Expiration MUST be enforced using a **monotonic clock**
- Wall-clock time is used only for audit readability
- Clock adjustments MUST NOT affect expiration

Audit logs MUST include:
- Wall-clock timestamp
- Monotonic elapsed time

---

## 4. Scope Enforcement

### 4.1 Scope Definition (Normative)

The activation `scope` defines the **only traffic classes** that HCM-triggered policies MAY affect.

HCM-triggered policies:

- MAY affect traffic classes listed in `scope`
- MUST NOT affect traffic classes outside `scope`

Violations MUST be logged.

---

### 4.2 Unmapped Scope Labels (Normative)

If the activation scope includes labels not referenced by any HCM policy:

- HCM SHALL still activate
- A warning event MUST be logged listing unused labels

This condition is **non-fatal**.

---

## 5. Traffic Handling Semantics

### 5.1 Degradation vs Termination (Normative)

HCM SHALL prefer **degradation** of non-priority traffic over termination.

Permitted degradation actions include:

- Deprioritization
- Steering to lower-preference links
- Deferral to existing routing

---

### 5.2 Traffic Shedding (Normative)

Traffic shedding is permitted ONLY if:

- Explicitly defined in policy
- Executed by existing network equipment
- Issued via `shed_via_equipment`
- Authorized and logged

Aether SHALL NOT directly drop traffic.

---

## 6. Exit Conditions and Post-Exit Behavior

### 6.1 Exit Triggers (Normative)

HCM exits when any of the following occur:

- Time limit expires
- Authorized manual deactivation
- External trigger de-asserted
- Policy-defined exit condition met

Exit MUST be logged.

---

### 6.2 Post-Exit Behavior (Normative)

Post-exit behavior MUST be explicitly defined in policy.

Permitted behaviors:

- **Immediate return to normal**
- **Gradual transition** (time-bounded)
- **Backlog handling via equipment**

Aether SHALL NOT buffer traffic.

---

### 6.3 Gradual Transition Requirements (Normative)

If gradual transition is specified:

- `transition_duration_seconds` is REQUIRED
- HCM policies remain active for exactly that duration
- Reversion MUST be atomic and logged

---

## 7. Auditability and Oversight

### 7.1 Mandatory Audit Logging (Normative)

Audit logs MUST include:

- Activation events
- Renewal events
- Exit events
- Per-decision logs referencing activation ID

Logs MUST be tamper-evident.

---

### 7.2 Operational Oversight (Normative)

Implementations MUST support:

- Alertable activation events
- Activation frequency metrics
- Cumulative duration metrics

Metrics MUST NOT include payload or user identity.

---

## 8. Governance Boundaries

### 8.1 Classification Integrity

HCM relies on upstream classification.

Aether:

- Executes policy for labels it receives
- Does NOT validate correctness of labels
- Does NOT inspect payloads

Misclassification is a governance issue, not a technical one.

---

## 9. Conformance Requirements

An implementation claiming HCM support MUST satisfy:

- External activation only
- Authorization metadata present
- Time bounds enforced
- Scope enforced
- Deterministic behavior
- Complete audit logs
- No concurrent activations

Failure of any requirement results in **non-conformance**.

---

## 10. Cross-References

This document depends on:

- `00-non-goals.md`
- `01-architecture.md`
- `02-policy-schema.md`
- `03-integration-guide.md`
- `conformance.md`

---
