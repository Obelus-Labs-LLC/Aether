# Aether Conformance Specification

**Version:** 1.0  
**Status:** Normative and Authoritative  
**Last Updated:** 2024-12-23

---

## 0. Conformance Model

### 0.1 Binary Conformance (Normative)

An implementation either **conforms** or **does not conform** to this specification.

Partial conformance claims are **forbidden**, including but not limited to:

- “Mostly compliant”
- “Roadmap compliant”
- “Reference-compatible”
- “Substantially conformant”

If any mandatory requirement is not met, the implementation is **non-conformant**.

---

### 0.2 Scope of Conformance (Normative)

Conformance applies across **all** of the following dimensions:

1. **Implementation conformance**  
   Software behavior, controller logic, adapters

2. **Deployment conformance**  
   Architecture, topology, integration model

3. **Operational conformance**  
   Runtime behavior, failure handling, HCM behavior

Failure in any dimension results in **overall non-conformance**.

---

### 0.3 Authority Hierarchy (Normative)

In the event of conflict, the following precedence applies:

1. This Conformance Specification
2. Normative sections of:
   - Architecture
   - Policy Schema
   - Integration Guide
   - Human Continuity Mode
3. Informative examples
4. Reference implementations or tools

Examples do **not** override normative requirements.

---

## 1. Conformance Levels

### 1.1 Level 1: Core Conformance (Required)

Level 1 is the minimum acceptable conformance level.

**Requirements include:**

- Control-plane-only operation
- Deterministic decision-making
- External traffic classification only
- Policy Schema compliance
- Integration Guide compliance
- Graceful degradation
- Complete audit logging
- No forbidden behaviors

---

### 1.2 Level 2: HCM Conformance (Additive)

Level 2 applies only if Human Continuity Mode support is claimed.

**Additional requirements include:**

- External HCM activation events
- Authorization metadata
- Time-bounded activation with monotonic clocks
- Scope enforcement
- Renewal handling
- Deterministic exit behavior
- HCM-specific audit logging

Claiming HCM support without Level 2 conformance is **misrepresentation**.

---

## 2. Core Requirements Checklist

Each requirement is mandatory unless explicitly stated otherwise.

---

### AETH-C-001: Control Plane Only

**Requirement:**  
Aether SHALL operate exclusively as a control-plane component and SHALL NOT forward, proxy, buffer, inspect, or modify user traffic.

**Verification:**  
- Remove Aether from deployment  
- User traffic MUST continue forwarding  
- Packet path MUST NOT traverse Aether processes

---

### AETH-C-002: No Firmware Modification

**Requirement:**  
Aether SHALL be deployable without firmware, kernel, or hardware modification.

**Verification:**  
- All integration uses existing management APIs  
- No custom firmware required

---

### AETH-C-003: Deterministic Decisions

**Requirement:**  
Given identical inputs, the system SHALL produce identical decisions.

**Verification:**  
- Capture full input state  
- Replay decision  
- Outputs MUST match after canonicalization

---

### AETH-C-004: External Classification Only

**Requirement:**  
Aether SHALL NOT inspect payloads or derive classification internally.

**Verification:**  
- No DPI libraries  
- Labels treated as opaque  
- Misclassified traffic is not corrected

---

### AETH-C-005: No Global State Dependency

**Requirement:**  
Aether SHALL operate with local state only and MUST NOT require global synchronization.

---

### AETH-C-006: Graceful Degradation

**Requirement:**  
Loss of Aether SHALL NOT cause traffic interruption.

Fallback behavior MUST be deterministic and documented.

---

### AETH-C-007: Audit Logging

**Requirement:**  
Every decision MUST be logged with complete metadata and tamper evidence.

---

### AETH-C-008: Policy Schema Compliance

**Requirement:**  
Evaluation MUST follow schema-defined precedence, matching, and tie-breaking.

---

### AETH-C-009: Integration Architecture

**Requirement:**  
Integration MUST follow sidecar/control-plane-only model.

---

### AETH-C-010: No Routing Protocol Participation

**Requirement:**  
Aether SHALL NOT originate or modify routing protocol advertisements.

---

## 3. Forbidden Behaviors

Presence of **any** forbidden behavior results in immediate non-conformance.

---

### AETH-F-001: Inline Data Path

User traffic traverses Aether → **Forbidden**

---

### AETH-F-002: Payload Inspection

Any DPI or application parsing → **Forbidden**

---

### AETH-F-003: Per-Flow or Per-User State

Flow/session/user tracking → **Forbidden**

---

### AETH-F-004: Hidden or Non-Deterministic State

Opaque ML models or hidden memory → **Forbidden**

---

### AETH-F-005: Adapter Policy Logic

Adapters performing decisions → **Forbidden**

---

### AETH-F-006: Control Traffic Subject to Aether Routing

Circular dependency → **Forbidden**

---

### AETH-F-007: Undocumented Fallback Behavior

Fallback not explicitly defined → **Forbidden**

---

### AETH-F-008: Concurrent HCM Activations

Multiple active HCM states → **Forbidden**

---

## 4. Determinism Verification

### 4.1 Deterministic Replay Test

Implementations MUST support:

- State export
- State reset
- Replay with identical outputs

Any divergence is non-conformance.

---

## 5. Policy Conformance Tests

Implementations MUST pass tests covering:

- Trigger semantics
- Rule ordering
- Conflict resolution
- Default behavior
- Tie-breaking with missing telemetry
- Label source specificity

---

## 6. HCM Conformance Tests (Level 2 Only)

Required coverage includes:

- Activation as trigger event
- Authorization metadata
- Monotonic expiration
- Scope enforcement
- Renewal handling
- Deterministic exit

---

## 7. Integration Conformance Tests

Required coverage includes:

- Aether removal without disruption
- Control traffic exemption
- Adapter mechanical translation
- Distributed controller independence

---

## 8. Audit Log Conformance

Audit logs MUST be:

- Complete
- Machine-parseable
- Tamper-evident
- Exportable

---

## 9. Conformance Claims

### 9.1 Claim Requirements

A valid claim MUST include:

- Conformance level
- Specification version
- Test date
- Evidence package

---

### 9.2 Prohibition on False Claims

False or misleading conformance claims are forbidden.

---

## 10. Summary

Conformance is **binary**.

If any requirement fails, the implementation is **non-conformant**.

---
