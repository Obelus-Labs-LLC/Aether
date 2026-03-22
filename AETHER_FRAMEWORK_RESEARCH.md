# Aether — Policy-Driven Uplink Arbitration Framework — Research Assessment

**Project:** `_backlog/specs/Aether`
**Repo:** `https://github.com/Obelus-Labs-LLC/Aether`
**Version:** 1.0
**Status:** Normative specification + Rust reference implementation

---

## 1. What Aether Is

A specification for deterministic, auditable control-plane coordination in multi-provider networks. In environments with multiple uplinks (satellite, terrestrial, cellular), Aether decides which uplink to use based on declared policies.

### Core Design Properties
| Property | Description |
|----------|-------------|
| Control-plane only | Never touches data-plane packets. Remove Aether and traffic keeps flowing |
| Deterministic | Same inputs always produce same outputs. No ML, no real-time optimization |
| No DPI | No deep packet inspection, no application identification |
| No per-flow state | No flow/session/user tracking |
| No routing protocol participation | Does not originate or modify routing advertisements |
| Tamper-evident audit logging | Every decision logged with complete metadata |
| Graceful self-degradation | Loss of Aether does not cause traffic interruption |
| Human Continuity Mode (HCM) | Disaster response override with time-bounded activation |

### Specification Components
- `conformance.md` — Binary conformance model (10 core requirements, 8 forbidden behaviors)
- `docs/01-architecture.md` — Control-plane-only architecture, sidecar deployment model
- `docs/02-policy-schema.md` — Policy evaluation: triggers, rules, ordering, conflict resolution
- `docs/03-integration-guide.md` — Adapter model for vendor-agnostic uplink control
- `docs/04-human-continuity-mode.md` — Emergency override with authorization, scoping, expiration
- `docs/00-non-goals.md` — Explicit exclusions (no DPI, no flow state, no routing)
- `schema/` — JSON schemas for decision logs, HCM activation events, telemetry

---

## 2. Is It Accurate?

### Architecture — Sound
- Control-plane-only is the correct design for critical infrastructure. If the arbitration framework dies, packets continue flowing on whatever path was last configured. This is fundamentally safer than inline architectures where framework failure causes traffic blackholes
- The sidecar/adapter model is proven in service mesh architectures (Envoy/Istio) and translates well to network control
- Determinism constraint is unusual but correct for regulated environments where decisions must be provably reproducible after-the-fact

### Conformance Model — Rigorous
- Binary compliance (no "mostly compliant") prevents gaming and ambiguity
- 10 core requirements (AETH-C-001 through AETH-C-010) are well-defined and verifiable
- 8 forbidden behaviors (AETH-F-001 through AETH-F-008) draw clear lines
- Deterministic replay test requirement is strong — export state, reset, replay, outputs must match

### Policy Schema — Solid Foundation
- Trigger → Rule → Action evaluation chain is well-structured
- Conflict resolution and tie-breaking are explicitly defined
- Label source specificity prevents ambiguous classification

### What's Questionable
- **Strict determinism may be too rigid**: Real uplinks have continuously changing quality (latency, loss, jitter). A system that ignores real-time measurements in favor of pure policy determinism may make provably-correct but practically-wrong decisions
- **No formal verification model**: The spec demands determinism but provides no formal model (TLA+, Alloy, Z) to verify that a given policy set produces intended behavior for all possible input states. This is a missed opportunity
- **No API contracts**: Northbound (operator policy input) and southbound (uplink control) interfaces are described conceptually but not specified as concrete APIs

---

## 3. Competitive Landscape

### SD-WAN (Most Similar Commercial Technology)
VMware VeloCloud, Cisco Catalyst SD-WAN (Viptela), Fortinet, Palo Alto Prisma, Versa. $7.9B market (2025) → $21B+ by 2030. SD-WAN provides policy-driven path selection across multiple uplinks but fundamentally differs from Aether:

| Property | SD-WAN | Aether |
|----------|--------|--------|
| Plane | Control + Data | Control only |
| DPI | Core feature | Forbidden |
| Per-flow state | Yes (session tracking) | Forbidden |
| Determinism | No (real-time metrics, ML) | Yes (same inputs = same outputs) |
| Audit trail | Troubleshooting logs | Tamper-evident compliance trail |
| Human override mode | No | Yes (HCM) |

SD-WAN optimizes enterprise WAN performance. Aether ensures auditable, deterministic uplink selection where wrong decisions have catastrophic consequences (grid outage, emergency comms failure). An organization could run SD-WAN on individual uplinks for traffic optimization while using Aether above it for uplink selection.

### 3GPP Standards
- **ATSSS (Release 16+)**: Access Traffic Steering, Switching, and Splitting. Closest standardized equivalent — defines steering modes (Active-Standby, Priority-Based, Smallest Delay). But deeply coupled to 3GPP core network functions, not a standalone control-plane spec
- **NTN (Release 17-18)**: Non-Terrestrial Networks. Formalizes satellite integration into 5G. Focuses on radio access and core network, not uplink arbitration
- **ITU-T Y.3219**: Fixed, Mobile, Satellite Convergence deterministic networking. Framework standard, not a policy arbitration spec

### IETF RFCs
- **RFC 5394**: Policy-Enabled Path Computation Framework. Most conceptually aligned — policy-based decision-making within PCE architecture
- **RFC 9256**: Segment Routing Policy Architecture. Policy-based routing and multi-path steering
- **RFC 8684**: Multipath TCP v1. Multi-path at transport layer

### Other
- **ONAP Policy Framework**: Comprehensive policy design/deployment/execution for network automation. But ONAP is a massive orchestration platform — the opposite of Aether's lightweight spec-only approach
- **MEF 70**: SD-WAN service attributes. Defines policy-driven optimization terminology
- **MIL-STD-188 series**: Military SATCOM standards (EHF, SHF, UHF, MUOS). Physical/link layer interoperability, not policy-driven control-plane arbitration

### The Gap Aether Fills
No existing framework combines: control-plane-only + deterministic decisions + no DPI/per-flow state + tamper-evident audit + human override mode + graceful self-degradation. SD-WAN is too heavy (data plane, DPI, flow state). 3GPP ATSSS is too coupled (requires 3GPP core). ONAP is too big (full MANO stack). Military standards are too low-level (physical/link layer).

---

## 4. Is the Concept Novel?

**Yes.** The specific combination is novel:

- **Determinism + auditability** is the key differentiator. Most network orchestration systems deliberately embrace non-determinism (real-time metrics, ML) as a feature. Aether's insistence that identical inputs produce identical outputs is the opposite philosophy, driven by regulatory auditability
- **Control-plane-only uplink arbitration** is distinct from SD-WAN's full-stack approach and 3GPP's integrated architecture
- **Human Continuity Mode** is novel at the uplink arbitration level. Emergency priority systems exist (GETS, WPS, FirstNet) but they provide priority *within* a single network. HCM overrides which *uplink/provider* is used — a different layer entirely

---

## 5. Human Continuity Mode — Deep Dive

### Existing Emergency Network Mechanisms
- **GETS (Government Emergency Telecommunications Service)**: Priority wireline calling for NS/EP personnel. Dial special access number with PIN for >95% call completion during congestion
- **WPS (Wireless Priority Service)**: Wireless complement to GETS. Dial *272 for priority cellular. Priority categories 1-5, >90% completion during congestion
- **TSP (Telecommunications Service Priority)**: Prioritizes repair/installation of critical circuits during disasters
- **FirstNet**: Dedicated 5G for first responders. Always-on priority + preemption on Band 14. 190+ deployable assets. Mandated by Congress to never throttle

### What Makes HCM Different
All existing mechanisms provide priority *within* a single network. HCM operates at a different level — it overrides the uplink selection policy itself, potentially switching which provider or transport medium is used. This is not priority within a network but arbitration *between* networks.

Additionally, HCM changes the entire decision-making framework (potentially relaxing determinism constraints in favor of any-available-link survival), whereas GETS/WPS are overlay priority schemes within normal operation.

The combination of uplink-level override + auditable disaster-mode transitions + defined degradation behavior + time-bounded activation with monotonic clocks is novel. No standard currently addresses this intersection.

---

## 6. Who Would Use This

### Tier 1 — Direct Fit
- **Regulated Critical Infrastructure**: Power grid (NERC CIP compliance requires auditable network decisions), water/wastewater, oil/gas pipelines. Already running satellite-terrestrial hybrid SCADA links with triple-redundant comms (2x satellite + 1x cellular)
- **Military/Defense**: Tactical networks managing multiple satellite and terrestrial links (MIL-STD-188 series). Deterministic + auditable aligns with military verification requirements
- **Disaster Response**: FEMA, state emergency management, first responder networks. HCM directly addresses infrastructure collapse scenarios

### Tier 2 — Strong Fit
- **Maritime/Aviation**: Mixed satellite providers (Iridium, VSAT, Starlink, cellular when in range) need policy-driven arbitration
- **Government/Regulatory Bodies**: SOX, NERC CIP, FedRAMP compliance where network decisions must be auditable and reproducible
- **Remote Operations**: Mining, energy extraction in areas with unreliable terrestrial connectivity depending on satellite failover

### Tier 3 — Emerging
- **Satellite Network Operators**: Starlink, OneWeb, Telesat offering enterprise connectivity could use Aether as standardized arbitration between their service and terrestrial alternatives
- **5G Private Networks**: Enterprises with private 5G + public carrier + satellite backup need arbitration

---

## 7. How to Make It Better

### Critical Gaps
1. ~~No implementation exists~~ — **Resolved**: Rust reference implementation in `aether-ref/` with 43 tests, mock adapter, and conformance test coverage
2. **No API/interface definition** — Northbound (policy input) and southbound (uplink control) interfaces are conceptual only. No OpenAPI spec, no protobuf definitions, no message formats
3. **No formal verification model** — The determinism guarantee makes this a perfect candidate for TLA+ or Alloy. Operators should be able to verify their policy sets produce intended behavior for all possible input states

### High-Value Improvements
4. **Hybrid telemetry mode** — Allow bounded, auditable reactions to measured link conditions (latency, loss, jitter). Every adaptation logged and reproducible given the same measurements. This preserves the audit trail while allowing practical link quality awareness
5. **Multi-stakeholder policy conflict resolution** — During disaster response, multiple agencies (FEMA, state, local, utility) may have conflicting HCM policies. Define precedence and conflict resolution
6. **3GPP ATSSS interoperability** — Map Aether uplink decisions to ATSSS steering modes (Active-Standby, Priority-Based). "Satellite primary, cellular standby" should translate directly to an ATSSS Active-Standby rule
7. **LEO satellite handling** — LEO satellite handover occurs every ~10 seconds. Define whether each handover triggers re-evaluation or satellite is treated as one logical uplink

### Strategic Improvements
8. **Compliance mapping** — Explicitly map audit logging requirements to NERC CIP-005/010/015-1, SOX, FedRAMP. This makes adoption easier for regulated industries
9. **MEF 70 alignment** — Use MEF 70 SD-WAN service attribute terminology where possible for vendor familiarity
10. **Explicit degradation levels** — Level 1: full arbitration → Level 2: last-known-good policy → Level 3: static priority failover → Level 4: any-available-link survival
11. **Performance Measurement Function** — Similar to 3GPP ATSSS's PMF/PMFP, even if measurements are only inputs to deterministic policy evaluation

---

## 8. Open Source Landscape

No open-source project directly replicates Aether's design:

- **flexiWAN**: First open-source SD-WAN/SASE. Policy-driven routing but full SD-WAN stack (control + data plane, DPI). Most mature open-source option
- **vtrunkd**: Linux daemon for multi-uplink aggregation at tunnel level. Data-plane focused
- **nante-wan**: Minimal open-source SD-WAN. Community project, less mature
- **EveryWAn**: Academic SD-WAN using SDN principles
- **NetBird**: Open-source zero-trust networking with policy-driven connectivity via WireGuard. Not SD-WAN but relevant for decentralized approach

**The whitespace is real.** No open-source project implements control-plane-only, deterministic, auditable uplink arbitration. If Aether were implemented, it would occupy a unique position — lighter than SD-WAN, more focused than ONAP/OSM, more rigorous on determinism and auditability than anything available.

---

## 9. Where It Could Be Used Across Obelus Labs

- **Dell ↔ Legion networking**: Currently using basic Tailscale mesh. Aether could define the arbitration policy for Tailscale vs LAN vs potential future cellular failover
- **Guardian**: Mobile app deployment will need Funnel or similar for public access. Aether policies could arbitrate between Tailscale Funnel and direct connectivity
- **As a product/standard**: Position as an open standard for auditable uplink arbitration. Submit to relevant standards bodies or industry groups focused on critical infrastructure networking. The spec-first approach is already correct

---

## 10. Summary Assessment

| Dimension | Rating | Notes |
|-----------|--------|-------|
| Concept | Strong | Fills genuine whitespace between SD-WAN and standards bodies |
| Specification | Strong | Rigorous conformance model, clear forbidden behaviors |
| Implementation | Complete | Rust reference implementation with 43 tests |
| Novelty | High | No existing framework combines all six design properties |
| Market Fit | Strong in niche | Regulated critical infrastructure, military, disaster response |
| Readiness | Spec-only | Needs reference implementation, API definitions, formal verification |
| HCM Novelty | High | Uplink-level emergency override is distinct from existing GETS/WPS/FirstNet |
