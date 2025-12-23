# Aether

**Version:** 1.0  
**Status:** Published Specification  
**License:** Apache 2.0 (code), CC BY 4.0 (documentation)

## Overview

Aether is a policy-driven uplink arbitration framework designed for resilient connectivity in multi-provider environments. It coordinates which uplink is used, when, and for what class of traffic—optimized for resilience and predictability under failure, not speed or profit.

**Aether is:**
- A neutral control framework for existing networks
- Policy-first, AI-second (AI advises, policy decides)
- Designed to fail gracefully, not magically
- Built for disaster response, critical infrastructure, and humanitarian networks

**Aether is NOT:**
- An ISP, satellite constellation, or telecommunications provider
- A consumer broadband or Wi-Fi product
- A surveillance or content-inspection system
- A replacement for existing routing protocols

## Key Features

- **Neutral Host Arbitration:** Coordinates across multiple uplinks/providers without owning infrastructure
- **Policy-Driven Decisions:** Deterministic, auditable link selection based on explicit policies
- **Experience-Based Link Memory:** Learns link behavior over time without inspecting payloads or tracking users
- **Human Continuity Mode:** Strictly bounded emergency mode for disaster response with time limits and audit trails
- **Provider-Agnostic:** Works with any satellite, terrestrial, or cellular provider
- **Graceful Degradation:** Continues operating when components fail

## Quick Start

### For Operators

1. **Review the specification:**
   - Start with [`docs/00-non-goals.md`](docs/00-non-goals.md) to understand scope boundaries
   - Read [`docs/01-architecture.md`](docs/01-architecture.md) for system design
   - Study [`docs/02-policy-schema.md`](docs/02-policy-schema.md) to write policies

2. **Understand integration requirements:**
   - Review [`docs/03-integration-guide.md`](docs/03-integration-guide.md)
   - Check if your equipment meets minimum capabilities
   - Plan sidecar/gateway deployment

3. **Deploy:**
   - Use example policies in `examples/policies/`
   - Configure integration adapters for your equipment
   - Test in monitor-only mode before enabling directives

### For Implementers

1. **Read conformance requirements:**
   - [`docs/conformance.md`](docs/conformance.md) defines testable requirements
   - Review forbidden behaviors (you must avoid these)
   - Understand binary conformance model

2. **Study schemas:**
   - `schema/decision-log-v1.json` - Audit log format
   - `schema/hcm-activation-v1.json` - Emergency mode events
   - `schema/telemetry-v1.json` - Link state telemetry

3. **Build to specification:**
   - Control plane only (no inline data path)
   - Deterministic decisions (same inputs = same outputs)
   - External classification (no payload inspection)
   - Complete audit logging (tamper-evident)

### For Procurement

Aether provides procurement-safe conformance language. See [`docs/conformance.md`](docs/conformance.md) Section 11 for RFP-ready text.

**Conformance levels:**
- **Level 1 (Core):** Basic uplink arbitration, policy evaluation, graceful degradation
- **Level 2 (Core + HCM):** Adds Human Continuity Mode for emergency response

## Documentation

### Core Specification Documents

| Document | Purpose | Audience |
|----------|---------|----------|
| [Non-Goals](docs/00-non-goals.md) | Scope boundaries, what Aether is NOT | All |
| [Architecture](docs/01-architecture.md) | System design, control plane constraints | Implementers, Architects |
| [Policy Schema](docs/02-policy-schema.md) | Policy language specification | Policy Authors, Operators |
| [Integration Guide](docs/03-integration-guide.md) | Deployment patterns, vendor integration | Operators, Integrators |
| [Human Continuity Mode](docs/04-human-continuity-mode.md) | Emergency mode specification | Emergency Response, Policy Authors |
| [Conformance](docs/conformance.md) | Testing requirements, certification | Implementers, Procurement |
| [Glossary](docs/glossary.md) | Canonical term definitions | All |

### Schemas

- `schema/decision-log-v1.json` - Decision log entry format
- `schema/hcm-activation-v1.json` - HCM activation event format
- `schema/telemetry-v1.json` - Link state telemetry format

## Example Policies

See `examples/policies/` for:
- Disaster response prioritization
- Cost-optimized routine traffic
- Low-latency application routing

## Design Principles

1. **Policy first, AI second:** AI advises, policy decides
2. **No payload inspection, no identity tracking**
3. **Deterministic behavior with logged decisions**
4. **Designed to fail gracefully, not magically**
5. **Framework, not hosted service**
6. **Provider-agnostic and neutral**

## Intended Adopters

- Satellite operators (government/resilience offerings)
- Telecom vendors building sat-terrestrial convergence
- Governments and critical infrastructure operators
- Disaster response and humanitarian networks

## Getting Help

- **Issues:** Open an issue for bugs, questions, or feature requests
- **Discussions:** Use GitHub Discussions for architecture questions
- **Contributions:** See CONTRIBUTING.md (if you choose to add this)

## License

- **Code and reference implementations:** Apache 2.0
- **Documentation:** Creative Commons Attribution 4.0 (CC BY 4.0)
- **Policy examples:** Public Domain (CC0)

## Version History

- **v1.0** (2024-12-23): Initial publication
  - Core specification (Non-Goals, Architecture, Policy Schema, Integration, HCM)
  - Conformance specification with binary testable requirements
  - JSON schemas for logs and events
  - Example policies

## Citation

If you reference Aether in academic or technical work:
```
Aether: Policy-Driven Uplink Arbitration Framework
Version 1.0 (2024)
https://github.com/[your-username]/aether
```

## Status

This is the **published v1.0 specification**. It is suitable for:
- Implementation by vendors
- Deployment by operators
- Inclusion in procurement requirements
- Reference in technical proposals

The specification has undergone adversarial review and hardening to ensure:
- Internal consistency across all documents
- Binary, testable conformance requirements
- Implementability without vendor lock-in
- Resilience under hostile interpretation

---

**Aether is designed to be a missing layer in connectivity infrastructure—not a product, but a framework that existing operators can adopt to coordinate multi-provider uplinks under failure conditions.**