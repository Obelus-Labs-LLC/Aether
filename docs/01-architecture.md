# Aether Architecture Specification

**Document ID:** 01-architecture  
**Version:** 1.0  
**Status:** Normative  
**Applies To:** Aether Specification v1.0

---

## 1. System Overview and Scope

Aether is a policy-driven control framework for uplink arbitration in multi-provider connectivity environments. It coordinates which uplink is used, when, and for what traffic class of traffic, optimizing for resilience and predictability under failure rather than cost or throughput.

### 1.1 What Aether Is

Aether is:

- A **control plane framework** that issues directives to existing network equipment
- A **policy evaluation engine** that selects uplinks based on explicit rules and observable link state
- A **neutral arbitrator** that operates without ownership of network infrastructure
- A **deterministic system** that produces identical outputs given identical inputs and state

### 1.2 What Aether Is Not

Aether is not:

- A data plane component (does not forward, route, inspect, buffer, or modify packets)
- A routing protocol or traffic engineering system (does not replace BGP/OSPF/MPLS-TE/SD-WAN)
- A centralized orchestrator requiring global synchronized state
- A surveillance or content-inspection system
- A hosted service or managed platform

See `docs/00-non-goals.md` for binding constraints.

---

## 2. System Context

The following diagram establishes Aether’s position in the stack and the separation between control plane and data plane.

> **Illustrative only:** Technologies shown do not imply endorsement, dependency, or required compatibility.

```

┌─────────────────────────────────────────────────────────────┐
│                    Policy Authors / Operators                │
│     (Define policy, manage triggers, authorize HCM, audit)    │
└────────────────────────────┬────────────────────────────────┘
│ Policy Set (out-of-band)
▼
┌─────────────────┐
│     Aether      │◄──── Link Telemetry (link-level)
│ Control Plane   │      (latency, availability, capacity)
│   Framework     │
└────────┬────────┘
│ Out-of-band directives (control only)
▼
┌───────────────────────────────────────────┐
│   Radios / Terminals / Routers / Gateways  │
│   (Execute selection via existing systems) │
└──────────┬───────────────────────────┬────┘
│                           │
Data Path │                           │ Data Path
(Aether does not │                           │ (Aether does not
participate)    │                           │  participate)
▼                           ▼
┌──────────────┐              ┌──────────────┐
│    Uplink A  │              │    Uplink B  │
│ (e.g., LEO)  │              │ (e.g., LTE)  │
└──────────────┘              └──────────────┘
│                           │
└──────────┬────────────────┘
▼
Existing Networks
(Internet / MPLS / Private / Gov)

```

**Key boundaries:**

- Aether operates entirely in the **control plane**
- User traffic flows between equipment and uplinks without traversing Aether
- Policies and triggers are provided out-of-band
- Telemetry is link-level and MUST NOT require per-flow or payload inspection

---

## 3. Control Plane Architecture

### 3.1 Core Principle

Aether issues directives via management/control interfaces **separate from packet forwarding paths**. Aether does not touch, buffer, inspect, or relay user data.

### 3.2 Directive Execution Model

Aether issues directives that may include:

- Link selection preference for a traffic class label
- Priority adjustments applied by existing equipment (e.g., queue/QoS profiles)
- Failover selection within policy-defined bounds
- Time-bounded operational modes (e.g., HCM) via externally asserted triggers

Directives are executed by existing equipment using management protocols/APIs such as:

- NETCONF/YANG
- gNMI/gNOI where applicable
- Vendor management REST APIs
- SNMP for read-only telemetry (if necessary)

> **Out of scope:** Data-plane programming interfaces (e.g., P4, eBPF, DPDK hooks) are not part of Aether. Aether uses out-of-band management/control only.

### 3.3 Prohibited Control Behaviors

Aether SHALL NOT:

- Originate, modify, or inject routing protocol advertisements (BGP, OSPF, IS-IS, etc.)
- Install arbitrary routes or modify inter-domain/intra-domain routing policy
- Require packet inspection or packet modification
- Depend on synchronized global state as a precondition for correctness
- Issue commands that make Aether itself a data-path prerequisite

---

## 4. Classification Boundaries

This section is normative and non-optional.

### 4.1 Classification Is External

Traffic classification MUST occur upstream of Aether-controlled systems. Classification is performed by external systems (routers, gateways, tagging engines, operators) and is provided to Aether as an input.

### 4.2 Labels Are Opaque

Aether reads externally assigned labels and treats them as opaque identifiers. Aether does not interpret their semantic meaning.

Aether SHALL NOT:

- Validate label correctness
- Infer intent from traffic properties
- Enrich labels
- Reclassify or rewrite labels

### 4.3 Prohibited Classification Behaviors

Aether SHALL NOT:

- Inspect packet payloads
- Parse protocol headers beyond reading externally assigned labels
- Infer class from ports, timing patterns, traffic volume, or TLS metadata
- Use per-flow inspection for any classification purpose

### 4.4 Misclassification Responsibility

If traffic is misclassified upstream, Aether applies the policy associated with the received label. Misclassification is out of scope and is the responsibility of the classification system and operator governance.

### 4.5 Example Classification Flow

> **Illustrative only:** Labels are not interpreted by Aether; policy authors define semantics.

```

┌─────────────────┐
│  Application    │
└────────┬────────┘
▼
┌───────────────────────────┐
│ Upstream Classifier/Edge  │  ◄── Classification occurs HERE
│ (Assigns label externally)│
└────────┬──────────────────┘
│ Structured label presented to Aether-controlled system
▼
┌───────────────────────────┐
│           Aether           │  ◄── Reads label, matches policy
│ (No inspection, no inference)│
└────────┬──────────────────┘
│ Out-of-band directive to equipment
▼
┌───────────────────────────┐
│ Equipment selects uplink   │  ◄── Packet flows directly to uplink
└───────────────────────────┘

````

### 4.6 Traffic Class Label Structure

A traffic class label SHALL be provided to Aether as a structured input:

- `label_id`: opaque identifier value (e.g., `"EF"`, `"routine"`)
- `label_source`: classification source (e.g., `"DSCP"`, `"VLAN"`, `"policy_engine"`)

Aether treats `label_id` as opaque and does not interpret semantics. Policies define actions for specific labels and sources.

Example:

```json
{
  "label_id": "EF",
  "label_source": "DSCP"
}
````

Multiple sources may coexist; policies define matching rules, including explicit wildcards as specified in the Policy Schema.

---

## 5. Decision Flow and Determinism

### 5.1 Deterministic Decision Principle

Given identical inputs and decision-influencing state, Aether SHALL produce identical decisions.

Non-deterministic behavior is a specification violation.

### 5.2 Decision Flow

```
Inputs:
- Policy Set (versioned)
- Trigger State (externally asserted)
- Link Telemetry (link-level, timestamped)
- Experience Memory (bounded, inspectable)
- Traffic Class Label (label_id, label_source)

┌──────────────────┐
│ Traffic Label In │
└────────┬─────────┘
         ▼
┌──────────────────────────┐
│ Policy Match + Evaluation │  ◄── deterministic
└────────┬─────────────────┘
         ▼
┌──────────────────────────┐
│ Link Selection Decision   │
└────────┬─────────────────┘
         ▼
┌──────────────────────────┐
│ Directive + Audit Logging │  ◄── tamper-evident
└────────┬─────────────────┘
         ▼
┌──────────────────────────┐
│ Equipment Executes Change │
└──────────────────────────┘
```

### 5.3 Policy Evaluation Rules

Policy evaluation is defined by `docs/02-policy-schema.md`. At a minimum:

* Policies are evaluated deterministically
* Priority order is explicit
* Rule ordering is preserved (first match wins)
* Tie-breaking is stable and documented
* Default behavior is explicit and bounded

### 5.4 Link State Inputs

Aether MAY consider link-level observable attributes including:

* `latency_ms` (and jitter if provided)
* `availability` (up/down/degraded)
* `capacity_mbps` (aggregate, link-level)
* Operator-defined enumerations (e.g., `provider_tier`), explicitly configured

**Telemetry derivation constraints (normative):**

Telemetry MUST be derived from:

* Interface/link metrics (e.g., counters, device telemetry)
* Provider-reported aggregates

Telemetry MUST NOT require:

* Per-flow inspection
* Payload-derived analysis
* DPI or application parsing

All telemetry must be link- or provider-level aggregates.

### 5.5 Determinism Definition

**Decision inputs SHALL include:**

1. Policy set contents and `policy_set_version`
2. Trigger state values
3. Link telemetry values used at evaluation time (with timestamps)
4. Experience memory state that influences decisions
5. Traffic class label (`label_id`, `label_source`)

**Decision inputs SHALL NOT include:**

* Wall-clock time, except where policies explicitly define time windows
* Telemetry arrival order
* Randomized tie-breaking
* Non-audited hidden state

**Tie-breaking:**
When multiple links satisfy policy equally, implementations MUST use stable tie-breaking defined in policy schema (preference list → metrics → lexicographic ordering by stable link identifier).

### 5.6 Trigger State Clarification

Triggers are externally asserted state variables, treated as explicit inputs.

Aether SHALL NOT infer triggers from:

* Traffic patterns
* Link behavior patterns
* Payload characteristics

Trigger activation/deactivation events MUST be logged with timestamp and authorization context where applicable (especially for HCM).

### 5.7 Link Identifier Stability

Link identifiers MUST be stable and canonicalized for the lifetime of a deployment.

Requirements:

* Identifiers MUST be operator-defined stable names or UUIDs
* Identifiers MUST NOT be derived from transient attributes (timestamps, IPs, interface indices)
* Identifiers MUST remain consistent across restarts and reloads

Stable IDs are required for tie-breaking, memory association, and audit correlation.

---

## 6. Experience-Based Link Memory

### 6.1 Purpose

Experience-based memory allows Aether to incorporate past link behavior into decisions without payload inspection and without requiring global state.

### 6.2 Memory Constraints

Experience-based memory MUST be:

* **Bounded:** fixed maximum size; defined eviction/aging
* **Inspectable:** operators can view memory state
* **Resettable:** operators can clear memory without restart
* **Exportable:** memory state can be exported for audit/migration

### 6.3 Memory Scope

Memory MAY store only link- or provider-level aggregates. It SHALL NOT store or infer:

* User identity
* Per-flow or per-session state
* Application fingerprints
* Payload-derived heuristics

---

## 7. Failure Modes and Graceful Degradation

### 7.1 Design Principle

Aether is designed to fail gracefully, not magically. Under degraded visibility or component failure, behavior must remain predictable and logged.

### 7.2 Failure Scenarios and Responses (Illustrative)

| Failure condition           | Aether behavior                     | Equipment fallback                                       |
| --------------------------- | ----------------------------------- | -------------------------------------------------------- |
| Policy undefined            | Apply explicit default or defer     | Continues forwarding                                     |
| Preferred links unavailable | Apply policy fallback               | Continues forwarding                                     |
| Telemetry missing/stale     | Use last-known + missing-data rules | Continues forwarding                                     |
| Aether unreachable          | No new directives                   | Continue last directives or revert to pre-Aether routing |

### 7.3 No Global State Requirement

Aether SHALL NOT require complete synchronized network-wide state to function correctly.

Implementations MUST support:

* Local-first operation
* Bounded synchronization assumptions
* Partial visibility (decisions degrade, system does not deadlock)

### 7.4 No Intentional Traffic Dropping by Aether

Aether SHALL NOT intentionally drop traffic in the data plane.

If traffic shedding is required, it MUST be:

* Explicitly configured in operator policy
* Executed by existing equipment (`shed_via_equipment`)
* Logged with authorization context where required

---

## 8. Explicitly Forbidden Architectures

Any architecture exhibiting the following SHALL NOT claim conformance:

1. Inline proxying or gateways requiring packet traversal through Aether-controlled components
2. Payload/DPI or deep header inspection (TLS metadata, SNI parsing, etc.)
3. Routing protocol manipulation or route advertisement injection by Aether
4. Centralized mandatory global-state orchestration
5. Firmware modification requirements for integration
6. Unbounded/opaque ML models influencing decisions
7. Adapter-side policy logic or decision overrides

---

## 9. Integration Points and Interfaces

Aether SHALL be deployable via sidecar/gateway architectures without firmware modifications.

### 9.1 Sidecar/Gateway Pattern (Illustrative)

```
┌─────────────┐     control only     ┌──────────────┐
│ Equipment A │◄────────────────────►│   Aether Ctrl │
└─────────────┘                     └──────────────┘
       │  data path                          ▲
       ▼                                     │ telemetry (link-level)
   (uplinks)                                 │
```

Control and telemetry channels MUST be out-of-band and MUST NOT be made dependent on Aether-managed routing decisions.

---

## 10. Embedded Conformance Requirements

Implementations MUST:

* Maintain control-plane-only operation (no data path)
* Enforce classification boundaries (labels only; no inspection)
* Provide deterministic decisions and stable tie-breaking
* Provide inspectable/exportable/resettable memory
* Log decisions in tamper-evident audit logs

---

## 11. Cross-References

* `docs/00-non-goals.md` — Binding scope boundaries and forbidden patterns
* `docs/02-policy-schema.md` — Policy evaluation semantics and schema rules
* `docs/04-human-continuity-mode.md` — HCM activation, scope, time bounds
* `conformance.md` — Testable criteria and evidence requirements

---

