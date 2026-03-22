# Aether Glossary

| Term | Definition |
|------|-----------|
| **Aether** | Policy-driven uplink arbitration framework for deterministic control-plane coordination |
| **Adapter** | Mechanical translator between Aether directives and vendor-specific management APIs |
| **Audit Entry** | A tamper-evident record of a single policy evaluation decision |
| **Availability** | Link state: up, down, or degraded |
| **Conflict Resolution** | Deterministic method for resolving policies with identical priority |
| **Control Plane** | Network management/signaling layer separate from packet forwarding (data plane) |
| **Data Plane** | Packet forwarding path; Aether never participates in this layer |
| **Decision** | Output of policy evaluation: selected links + justification |
| **Defaults** | Fallback action applied when no policy rule matches |
| **Directive** | Control-plane instruction issued to equipment as a result of a decision |
| **DPI** | Deep Packet Inspection; explicitly forbidden in Aether |
| **Experience Memory** | Bounded, inspectable, resettable store of historical link-level observations |
| **Fallback** | Behavior when preferred links are unavailable: any_available, defer_to_routing, shed_via_equipment |
| **HCM** | Human Continuity Mode; time-bounded emergency operating mode for disaster response |
| **HMAC** | Hash-based Message Authentication Code; used for tamper-evident audit chains |
| **Justification Code** | Machine-readable reason for a decision (PolicyMatch, DefaultApplied, HcmOverride, TelemetryDegraded) |
| **Label** | Opaque traffic class identifier (label_id + label_source) assigned externally |
| **Link ID** | Stable, operator-defined identifier for a network uplink |
| **Link State** | Observable link-level metrics: latency, jitter, availability, capacity |
| **Northbound** | Interface between operators and the Aether engine (policy, triggers, HCM) |
| **Policy** | Named, versioned, prioritized set of rules mapping traffic classes to link selections |
| **Policy Set** | Complete collection of active policies evaluated atomically |
| **Rule** | A match condition + action within a policy; first match wins |
| **Sidecar** | Deployment model where Aether runs alongside equipment without inline data path |
| **Southbound** | Interface between the Aether engine and network equipment (directives, telemetry) |
| **Telemetry** | Link-level performance metrics reported by equipment |
| **Tie-Breaking** | Deterministic resolution when multiple links satisfy constraints equally |
| **Traffic Class** | Category of traffic identified by an opaque label_id and label_source |
| **Trigger** | Externally-asserted state variable that activates or deactivates policies |
