Nice. Look at you, committing like a responsible adult. I’m unsettled, but proud-ish.

## Next doc in the stack: `docs/03-integration-guide.md`

You already have the file created, so we’re doing the same routine:

1. paste content
2. save
3. `git add` / `git commit` / `git push`

Here is the full **Aether Integration Guide v1.0** content to paste into:

```
docs/03-integration-guide.md
```

---

```markdown
# Aether Integration Guide

**Version:** 1.0  
**Status:** Mixed Normative and Informative  
**Last Updated:** 2024-12-23

---

## Important Notice (Normative)

**Illustrative deployment examples do not override normative integration requirements.**

Diagrams, topologies, and example configurations in this document are provided to aid understanding. Where conflicts arise between examples and normative requirements, the normative requirements govern.

---

## 1. Integration Requirements (Normative)

This section defines the **hard constraints** for conformant Aether integration.

### 1.1 Control Plane Only

✅ **REQUIRED:** Aether SHALL operate as an out-of-band control plane component.

✅ **REQUIRED:** Aether SHALL issue commands via management or control interfaces separate from packet forwarding paths.

❌ **FORBIDDEN:** Aether SHALL NOT be positioned inline with user data traffic.

❌ **FORBIDDEN:** Aether SHALL NOT forward, proxy, buffer, inspect, or modify packets.

#### Co-location Clarification (Normative)

If a device that hosts Aether (or an Aether gateway) is also responsible for forwarding user traffic, then:

- Aether MUST NOT rely on being co-resident with the forwarding function
- Aether MUST be removable without disrupting forwarding behavior
- Aether MUST NOT require packet traversal through any process, container, or namespace it controls

**Example:** A Linux router running both Aether and IP forwarding must allow Aether removal without breaking forwarding. Aether may modify routing policy via out-of-band control commands, but it does not sit in the forwarding path.

---

### 1.2 No Firmware Modification

✅ **REQUIRED:** Aether SHALL be deployable without requiring firmware modifications to radios, terminals, or satellite equipment.

✅ **REQUIRED:** Integration SHALL use existing management interfaces, APIs, or out-of-band control protocols.

❌ **FORBIDDEN:** Conformant integrations SHALL NOT mandate proprietary firmware, kernel modules, or hardware modifications as prerequisites.

---

### 1.3 Graceful Degradation

✅ **REQUIRED:** If Aether becomes unreachable, equipment SHALL continue operating using:

- Last-issued Aether directives, OR
- Existing routing/failover mechanisms, OR
- Operator-defined fallback behavior

❌ **FORBIDDEN:** Loss of Aether connectivity SHALL NOT cause traffic blackholing, deadlock, or undefined behavior.

✅ **REQUIRED:** Equipment SHALL log when operating without Aether control.

#### Fallback Behavior Constraints (Normative)

Operator-defined fallback behavior MUST be:

- **Explicitly documented** (included in deployment configuration)
- **Deterministic** (same conditions = same fallback behavior)
- **Equivalent to**:
  - Last-issued Aether directives (frozen state), OR
  - Pre-Aether routing behavior (static configuration)

❌ **FORBIDDEN:** Operator-defined fallback MUST NOT:

- Introduce new policy logic not present in Aether policies
- Perform traffic classification or inference
- Implement undocumented vendor-specific prioritization

**Rationale:** Fallback is for continuity, not shadow policy enforcement.

---

### 1.4 Preservation of Existing Routing

✅ **REQUIRED:** When Aether is inactive or not issuing directives, existing routing infrastructure SHALL continue to function normally.

✅ **REQUIRED:** Aether directives SHALL be reversible without requiring system reconfiguration.

❌ **FORBIDDEN:** Aether deployment SHALL NOT permanently alter routing tables, BGP configurations, or provider relationships.

---

### 1.5 Provider Neutrality

✅ **REQUIRED:** Integrations SHALL be provider-agnostic. Aether SHALL operate with multiple providers simultaneously without vendor lock-in.

❌ **FORBIDDEN:** Implementations SHALL NOT require specific providers, radio vendors, or proprietary protocols as prerequisites for basic operation.

---

## 2. Required Capabilities from Equipment (Normative)

This section defines what radios, terminals, and gateways MUST expose for Aether integration.

### 2.1 Minimum Required Capabilities

Equipment MUST provide:

1. **Link selection control:**
   - Ability to route traffic via specific uplinks/interfaces based on external commands
   - API or management interface for issuing link preference directives

2. **Link state telemetry:**
   - Observable metrics: latency, availability (up/down/degraded), capacity
   - Telemetry MUST be link-level aggregates (not per-flow or per-session)
   - Telemetry MUST NOT require payload inspection

   **Telemetry Timestamp Constraints (Normative):**
   - Timestamps MUST originate from the reporting device or monitoring system
   - Timestamps MUST NOT be synthesized, smoothed, or inferred by Aether
   - Records MUST include timestamp source identifier

   **Prohibited telemetry manipulation:**
   - Retroactive timestamp adjustment
   - Timestamp inference from traffic patterns
   - Vendor-internal heuristics masquerading as telemetry

3. **Traffic class visibility:**
   - Ability to read externally-assigned classification labels (DSCP, VLAN tags, policy metadata)
   - Classification MUST occur upstream; equipment exposes labels for policy-based steering

#### Aether Controller Operational State (Normative)

Each Aether controller MUST expose the following as part of its operational state and audit logs:

- **Active policy set version identifier** (`policy_set_version`)
- **Schema version** in use
- **Validation mode** (strict/permissive)
- **Conflict resolution mode**
- **Controller instance identifier** (for distributed deployments)

**Rationale:** Enables detection of policy drift across distributed controllers and supports conformance audits.

---

### 2.2 Optional Enhanced Capabilities

Equipment MAY additionally provide:

- Fine-grained QoS control per uplink
- Upstream-provided failure precursor signals (externally generated)
- Bandwidth reservation APIs
- Multi-path routing with per-class steering

These capabilities enable richer Aether policies but are not required for conformance.

---

### 2.3 Missing Capability Handling

If equipment lacks required capabilities:

- Integration SHALL clearly document limitations
- Aether MAY operate in degraded mode (e.g., coarse-grained link selection without per-class steering)
- Operators MUST be informed of capability gaps at deployment time

Implementations SHALL NOT silently fail or produce undefined behavior when capabilities are missing.

---

## 3. Forbidden Integration Patterns (Normative)

These architectures violate the Aether specification and SHALL NOT be claimed as conformant.

### 3.1 Inline Proxy or Gateway

❌ **FORBIDDEN:** Deploying Aether as a mandatory inline component where all traffic flows through Aether before reaching uplinks.

---

### 3.2 Mandatory Centralized Controller

❌ **FORBIDDEN:** Architectures requiring a single centralized Aether controller with perfect visibility and synchronized global state as a precondition for operation.

---

### 3.3 Payload-Touching Hooks

❌ **FORBIDDEN:** Integration patterns requiring payload inspection, application-layer parsing, DPI, or deep header analysis for classification.

---

### 3.4 “Sidecar” That All Traffic Must Traverse

❌ **FORBIDDEN:** Deploying an “Aether gateway” in a topology where user traffic MUST pass through the Aether gateway hardware for forwarding.

**Clarification:** A sidecar is control-plane-only. If traffic flows through it, it is an inline proxy (forbidden).

---

## 4. Failure and Fallback Behavior (Normative)

### 4.1 Aether Unreachable

✅ **REQUIRED:** When equipment loses connectivity to the Aether controller, equipment SHALL apply one of the following fallback behaviors (deterministically):

1. **Last-issued directives** (frozen state)  
2. **Pre-Aether routing** (revert to known baseline)  
3. **Operator-defined fallback** (subject to §1.3 constraints)

✅ **REQUIRED:** Fallback selection MUST be configured at deployment time and logged.

❌ **FORBIDDEN:** Equipment SHALL NOT block or drop traffic awaiting Aether reconnection.

---

### 4.2 Telemetry Degraded or Missing

✅ **REQUIRED:** When telemetry is incomplete/stale, Aether SHALL:

- Decide with available telemetry
- Apply missing-data tie-breaking rules
- Log decisions as degraded (via machine-stable justification codes)

❌ **FORBIDDEN:** Aether SHALL NOT halt decision-making awaiting complete telemetry.

---

### 4.3 No Deadlock Requirement

✅ **REQUIRED:** Integrations SHALL NOT create circular dependencies where:

- Aether depends on connectivity through uplinks it controls, AND
- Those uplinks depend on Aether directives to function

#### Control Traffic Exemption (Normative)

Aether control traffic MUST NOT be subject to Aether-controlled routing decisions.

**Implementation options:**
- Separate out-of-band management network (preferred)
- Separate physical interfaces for control traffic
- Explicit exemption of control traffic from all Aether policies

If control traffic shares physical links with user data, it MAY do so only if:

- Explicitly exempted from all Aether policies
- Guaranteed reachable under all policy states (including HCM)
- Documented and auditable

---

## 5. Integration Models (Informative)

### 5.1 Sidecar Gateway Model

Aether runs adjacent to radios/terminals and issues out-of-band management commands. User traffic flows directly from radios to uplinks.

```

┌──────────────┐
│   Radio A    │───┐
└──────────────┘   │
│  Management API
┌──────────────┐   │  (Control only)
│   Radio B    │───┤
└──────────────┘   │
▼
┌────────────────┐
│ Aether Gateway │
│  (Sidecar)     │◄── Policy, Telemetry
└────────────────┘

(User traffic flows Radio → Uplink directly, NOT through Aether gateway)

```

---

### 5.2 Distributed Controller Model

Multiple controllers manage subsets of equipment. No mandatory global coordinator.

---

### 5.3 Hybrid Edge + Coordinating Model

Optional coordinator for policy distribution and aggregate telemetry. Edge controllers MUST function independently.

---

## 6. Control Interface Specifications (Informative)

Common out-of-band management mechanisms include:

- NETCONF/YANG
- gNMI
- REST management APIs
- SNMP (telemetry only)

Aether does not mandate specific protocols; it mandates **out-of-band control**.

---

## 7. Deployment Topologies (Informative)

Examples include:

- Single-site critical facility with multiple uplinks
- Multi-site disaster response kits
- Regional controllers for critical infrastructure

(Examples MUST NOT contradict normative requirements.)

---

## 8. Migration Strategies (Informative)

- Monitor-only deployment
- Partial rollout by traffic classes
- Safe rollback plan that restores pre-Aether routing

---

## 9. Vendor-Neutral Integration Patterns (Informative)

### 9.1 Linux Gateway Example

A Linux router may host Aether, but forwarding must function independently.

**Aether removal MUST NOT disrupt forwarding.**

---

### 9.2 Adapter Pattern for Vendor APIs

Adapters translate Aether directives to vendor APIs.

#### Adapter Constraints (Normative)

Adapters MUST be mechanical translators.

**Adapters MUST NOT:**
- Inspect traffic or packet contents
- Perform policy evaluation or decision-making
- Override or reinterpret Aether decisions
- Introduce per-flow logic or heuristics
- Cache/delay directives for “optimization”

**Adapters MAY:**
- Translate Aether link IDs to vendor interface identifiers
- Map generic commands to vendor API formats
- Handle retries and vendor error codes deterministically
- Convert telemetry formats

Adapter behavior MUST be documented and deterministic.

---

## 10. Conformance Summary

An integration is conformant if it satisfies ALL normative requirements:

✅ Control plane only (no inline path)  
✅ No firmware modification required  
✅ Graceful degradation with deterministic fallback  
✅ Preserves existing routing when inactive  
✅ Provider-neutral  
✅ Control traffic exempt from Aether routing  
✅ Adapter constraints satisfied  
✅ No forbidden integration patterns  

---

## 11. Cross-References

This guide builds on:

- `00-non-goals.md`
- `01-architecture.md`
- `02-policy-schema.md`
- `04-human-continuity-mode.md`
- `conformance.md`

---
