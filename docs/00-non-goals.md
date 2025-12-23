# Aether Non-Goals

**Document ID:** 00-non-goals  
**Version:** 1.0  
**Status:** Normative  
**Applies To:** Aether Specification v1.0

---

## 1. Purpose

This document defines **explicit non-goals** of the Aether specification.

Non-goals are **normative constraints**, not aspirational statements.  
They exist to:

- Prevent architectural drift
- Eliminate ambiguous implementation claims
- Block feature creep that undermines determinism, auditability, or operator control
- Make conformance binary and enforceable

If an implementation violates any non-goal, it is **non-conformant**, regardless of functionality or performance.

---

## 2. Out of Scope by Design

Aether is intentionally limited. The following capabilities are **explicitly out of scope** and SHALL NOT be implemented as part of Aether.

### 2.1 Inline Data Path Components

Aether SHALL NOT:

- Forward packets
- Proxy traffic
- Buffer user data
- Act as a gateway, firewall, or NAT
- Sit inline between traffic sources and uplinks

Aether operates **exclusively** in the control plane.  
Any deployment where user traffic must traverse an Aether-controlled process, container, VM, or appliance is **non-conformant**.

---

### 2.2 Payload Inspection or Surveillance

Aether SHALL NOT:

- Inspect packet payloads
- Perform deep packet inspection (DPI)
- Parse application-layer protocols
- Infer application identity, user identity, or content
- Use TLS metadata (SNI, ALPN, certificates) for decision-making

All traffic classification occurs **upstream** via externally-assigned labels.  
Aether treats labels as **opaque identifiers**.

---

### 2.3 Traffic Classification Logic

Aether SHALL NOT:

- Define traffic classes internally
- Reclassify traffic
- Override or reinterpret labels
- Learn classifications from traffic behavior
- Apply heuristics to guess intent or priority

Aether consumes classifications; it does not create them.

---

### 2.4 Global Network Control or Omniscience

Aether SHALL NOT:

- Require a global, synchronized network view
- Depend on centralized controllers for correctness
- Assume complete telemetry coverage
- Require perfect state convergence to operate

Distributed Aether controllers MUST function independently with local state.

---

### 2.5 Routing Protocol Participation

Aether SHALL NOT:

- Participate in routing protocols (BGP, OSPF, IS-IS, RIP, etc.)
- Originate or inject routing advertisements
- Modify inter-provider routing relationships
- Act as a routing authority

Aether MAY configure **local forwarding behavior** via existing equipment but SHALL NOT function as a router.

---

### 2.6 Firmware, Kernel, or Hardware Modification

Aether SHALL NOT require:

- Custom firmware on radios, terminals, or gateways
- Kernel modules
- Hardware accelerators
- Vendor-specific silicon features

Conformant integrations use **existing management interfaces** only.

---

## 3. Explicitly Forbidden Architectural Patterns

The following patterns are **explicitly forbidden** and invalidate conformance claims.

### 3.1 Inline “Sidecar” Gateways

A deployment is **non-conformant** if:

- Traffic must traverse an Aether gateway to reach an uplink
- Aether failure interrupts traffic forwarding
- Aether is required for packet delivery

A control-plane sidecar that forwards traffic is **not** a sidecar — it is an inline proxy and is forbidden.

---

### 3.2 Centralized Mandatory Controllers

A deployment is **non-conformant** if:

- A single controller is required for operation
- Controllers cannot operate independently
- Global state synchronization is required for correctness

Optional coordination is allowed; mandatory centralization is not.

---

### 3.3 Machine Learning as Decision Authority

Aether SHALL NOT:

- Use machine learning models as primary decision logic
- Allow non-deterministic inference to influence decisions
- Rely on opaque or uninspectable model state

Determinism is mandatory.  
All decision-influencing state MUST be inspectable, exportable, and resettable.

---

### 3.4 Hidden Policy or Shadow Logic

Aether SHALL NOT:

- Embed policy logic outside the policy schema
- Implement “fallback intelligence” not defined in policy
- Allow adapters to reinterpret or override decisions
- Apply undocumented heuristics

All behavior MUST be traceable to policy, triggers, or explicitly documented fallback configuration.

---

#
