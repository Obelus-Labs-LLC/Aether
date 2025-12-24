
⸻

Aether

A policy-driven uplink arbitration framework for deterministic, auditable control-plane coordination

Version: 1.0
Status: Published Specification (Draft / RFC-style)
Target: Critical Infrastructure, Satellite/Terrestrial Hybrid Networks, Disaster Response

⸻


The Core Thesis

Aether is not designed to outperform SDN, routing protocols, or orchestration systems.
It is designed to constrain them.

Modern network control planes suffer from scope creep: over time they accumulate hidden responsibilities—DPI, heuristics, vendor-specific state, implicit inference—until they become difficult to audit and unpredictable under failure.

Aether deliberately trades capability for predictability.

It defines a strict interaction model between intent and execution that remains auditable and valid even when links flap, telemetry degrades, or controllers disappear. Aether explicitly assumes that normal routing, forwarding, and header processing behave exactly as they do today.

⸻

The Aether Constraint

To remain deterministic and auditable, Aether deliberately refuses to do the following:
	•	No Payload Inspection
Traffic classification is assumed to occur upstream at the edge.
	•	No Path Computation
Aether does not replace BGP, OSPF, IS-IS, or MPLS. It dictates which existing path should be preferred, not how paths are computed.
	•	No Per-Flow State
Aether manages uplink and provider state, not individual sessions or users.
	•	No Inline Presence
Aether provides control-plane directives only. It never processes, forwards, proxies, buffers, or inspects packets.

These constraints are intentional. Violating any of them breaks determinism and auditability.

⸻

Skeptic’s Map

If you are coming from a networking background and think “this is just [X]”, use this map to find the specific technical answer to your objection.

If you think…	Read this document…	To understand…
“This is just SDN.”	docs/01-architecture.md	Why Aether is a governance layer over SDN, not a replacement
“This can’t scale.”	docs/00-non-goals.md	How Aether avoids scaling issues by refusing flow state
“How would I deploy this?”	docs/03-integration-guide.md	The boundary between Aether and your hardware
“Isn’t this just QoS?”	docs/02-policy-schema.md	Why policy arbitration ≠ packet prioritization


⸻

How It Works

Aether functions as a state referee, not a signaling protocol or path computation engine.

It takes two inputs and produces one auditable output:
	•	Input A — Policy
Explicit, operator-defined constraints
(e.g., “Critical telemetry must use the most stable link regardless of latency”)
	•	Input B — Observed Telemetry
Link-level performance data (loss, latency, availability), not per-flow or packet-level data
	•	Output — Directive
A deterministic control-plane instruction sent to existing equipment

Interaction Model

[ Business Logic ] → [ Aether Policy ] → [ Aether Decision Engine ]
                                              |
         [ Observed Link Telemetry ] -----------+
                                              |
                                    [ Deterministic Directive ]
                                              |
                                [ Hardware / SDN / BGP / IGP ]

If Aether fails or is removed, forwarding continues using existing behavior.

⸻

Repository Structure

1. The Specification (docs/)
	•	Non-Goals
The defensive perimeter — what Aether will never do
	•	Architecture
Formal definition of the control-plane boundary
	•	Policy Schema
Deterministic grammar for expressing intent
	•	Human Continuity Mode (HCM)
A tightly bounded emergency state for disaster response
	•	Conformance Specification
Binary, testable requirements and audit criteria

2. The Contract (schema/)

Canonical JSON schemas defining the only interfaces Aether speaks:
	•	telemetry-v1.json — what the network is allowed to report
	•	decision-log-v1.json — tamper-evident decision records
	•	hcm-activation-v1.json — emergency activation events

3. Scenario Walkthroughs (examples/)

Conceptual walkthroughs demonstrating behavior under failure scenarios such as:
	•	Flapping satellite links
	•	Congested cellular backhaul
	•	Partial telemetry loss

These are illustrative, not benchmarks.

⸻

Intended Adopters
	•	Satellite Operators
Offering resilience-focused services to government and NGO customers
	•	Critical Infrastructure Operators
Managing hybrid terrestrial–satellite connectivity for power, water, and emergency services
	•	Procurement & Oversight Bodies
Enforcing non–black-box behavior via conformance requirements

⸻

Status & License

This repository contains the published v1.0 draft specification of Aether.

It is intentionally specification-only:
	•	No reference implementation
	•	No hosted service
	•	No vendor affiliation

The goal is to define clear, enforceable constraints for a control-plane coordination layer and invite critique from people who have operated real networks.

Feedback is explicitly welcome on:
	•	Failure-mode realism
	•	Boundary erosion in real deployments
	•	Whether the constraints defined here are enforceable in practice

License:
	•	Code: Apache 2.0
	•	Documentation: Creative Commons Attribution 4.0 (CC BY 4.0)




⸻
