# Aether

A policy-driven uplink arbitration framework for deterministic, auditable control-plane coordination in multi-provider networks.

**Version:** 1.0-Draft
**Status:** Specification + Reference Implementation
**Target Environments:** Critical infrastructure, satellite-terrestrial hybrid networks, disaster response

-----

## Overview

Aether defines a constrained control-plane coordination layer for environments with multiple uplinks and heterogeneous providers (Satellite, Terrestrial, Cellular).

It does not replace routing protocols, forwarding behavior, or existing control systems. Instead, it defines how policy intent can be translated into auditable, deterministic directives without introducing inline dependencies or opaque decision logic.

The specification is intentionally limited in scope.

-----

## The Problem

Modern network control planes often suffer from scope creep. Orchestration systems frequently accumulate hidden responsibilities — Deep Packet Inspection (DPI), heuristic path computation, vendor-specific state — until they become opaque and unpredictable during failure.

Aether addresses this by constraining the control plane. It defines a strict interaction boundary between intent and execution that remains valid even when links flap, telemetry degrades, or controllers disappear.

**Core Thesis:** Aether deliberately trades capability for predictability.

-----

## The Aether Constraint

To preserve determinism and auditability, Aether explicitly refuses to perform the following functions:

- **No Payload Inspection:** Traffic classification must occur upstream; Aether operates on pre-tagged traffic classes.
- **No Path Computation:** Aether does not replace BGP, OSPF, IS-IS, or MPLS. It selects between existing, valid paths — not how paths are computed.
- **No Per-Flow State:** Arbitration is performed at the provider/uplink level, not at the session or user level.
- **No Inline Presence:** Aether provides out-of-band directives only. It never processes, forwards, proxies, buffers, or inspects data-plane packets.

-----

## How It Works

Aether functions as a **policy arbiter**, not a signaling protocol or path computation engine.

**Inputs:**

- **Policy (Constraints):** Operator-defined rules (e.g., "Critical Telemetry: Path Stability > Latency")
- **Telemetry (Observed State):** Link-level performance metrics (loss, latency, jitter) defined in `schema/telemetry-v1.json`

**Output:**

- **Directive:** A control-plane instruction for the underlying hardware or SDN controller

**Fail-Safe:** If the Aether engine fails, the underlying hardware maintains its last-known valid state or reverts to local routing protocol defaults.

-----

## Repository Structure

### Specification (`docs/`)

| Document | Description |
|----------|-------------|
| `docs/00-non-goals.md` | Explicit scope boundaries and what Aether will never do |
| `docs/01-architecture.md` | Control-plane enforcement boundary and decision flow |
| `docs/02-policy-schema.md` | Deterministic grammar for expressing policy intent |
| `docs/03-integration-guide.md` | Adapter model and equipment integration requirements |
| `docs/04-human-continuity-mode.md` | Time-bounded emergency operating mode for disaster response |
| `docs/glossary.md` | Term definitions |
| `conformance.md` | Binary conformance model: 10 core requirements, 8 forbidden behaviors |

### Research

| Document | Description |
|----------|-------------|
| `AETHER_FRAMEWORK_RESEARCH.md` | Background research, design rationale, and open questions |

### Schemas (`schema/`)

| Schema | Description |
|--------|-------------|
| `schema/telemetry-v1.json` | Link-state reporting format |
| `schema/decision-log-v1.json` | Tamper-evident audit log schema |
| `schema/hcm-activation-v1.json` | Human Continuity Mode activation contract |

### Reference Implementation (`aether-ref/`)

Rust reference implementation with HTTP API server. See [`aether-ref/README.md`](aether-ref/README.md) for build, usage, and deployment instructions.

| Component | Path | Description |
|-----------|------|-------------|
| Policy engine | `src/engine/` | Deterministic evaluation (49 tests) |
| HTTP API | `src/api/http.rs`, `openapi.yaml` | Axum server, 10 endpoints, OpenAPI 3.1 |
| Netlink adapter | `src/adapter/netlink.rs` | Linux `ip route` with idempotent apply and rollback |
| Telemetry trust | `src/adapter/registry.rs` | HMAC verification, sequence monotonicity, heartbeat liveness |
| Audit logging | `src/audit/` | HMAC-SHA256 tamper-evident chain |
| HCM | `src/hcm/` | Human Continuity Mode lifecycle management |
| Formal model | `formal/AetherSpec.tla` | TLA+ — policy completeness, HCM correctness |
| Deployment | `deploy/` | Docker Compose with simulated multi-link topology |
| Failure modes | `docs/failure-modes.md` | Explicit outputs for degraded/missing/expired scenarios |
| Examples | `examples/policies/` | Critical infrastructure, disaster response, multi-provider |

-----

## Intended Audience

- Satellite and hybrid connectivity operators
- Critical infrastructure network operators
- Humanitarian and disaster response coordination
- Procurement and oversight bodies seeking auditable, non-proprietary requirements

-----

## License

- **Code/Schemas:** Apache 2.0
- **Documentation:** CC BY 4.0
