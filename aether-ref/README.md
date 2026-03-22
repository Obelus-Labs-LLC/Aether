# Aether Reference Implementation

Rust reference implementation of the [Aether specification](../README.md) — a policy-driven uplink arbitration framework for deterministic, auditable control-plane coordination.

## Build

```bash
cargo build --release
```

Requires Rust 1.75+.

## Test

```bash
cargo test
```

## CLI Usage

### Evaluate a traffic class

```bash
aether evaluate \
  --policy examples/policies/critical-infrastructure.yaml \
  --telemetry telemetry.json \
  --label-id scada_telemetry \
  --label-source DSCP
```

### Validate a policy set

```bash
aether validate --policy examples/policies/disaster-response.yaml
```

### Verify audit chain integrity

```bash
# Via environment variable (recommended)
export AETHER_AUDIT_KEY=<hex-encoded-key>
aether audit-verify --log audit.json

# Via key file
aether audit-verify --log audit.json --key-file /path/to/key

# Via command line (not recommended for production)
aether audit-verify --log audit.json --key-hex <hex-encoded-key>
```

## Architecture

```
src/
  types/       Type definitions (link, policy, decision, HCM, audit)
  engine/      Core policy evaluator (pure, deterministic)
  telemetry/   Link state ingestion and experience memory
  audit/       HMAC-SHA256 tamper-evident audit logging
  hcm/         Human Continuity Mode lifecycle management
  adapter/     Southbound adapter trait + mock implementation
  state/       State snapshot for deterministic replay
  api/         Northbound (operator) and southbound (equipment) interfaces
```

## Example Policies

- `examples/policies/critical-infrastructure.yaml` — Power grid SCADA telemetry
- `examples/policies/disaster-response.yaml` — HCM activation for emergency medical/public safety
- `examples/policies/multi-provider.yaml` — LEO + LTE + MSS with latency-based selection

## Conformance

This implementation targets Level 1 (Core) and Level 2 (HCM) conformance per `conformance.md`.

## License

Apache 2.0
