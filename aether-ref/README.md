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

### Start the HTTP server

```bash
aether serve --bind 0.0.0.0:8080 --audit-key-hex <64-char-hex-key>
```

## Architecture

```
src/
  types/       Type definitions (link, policy, decision, HCM, audit)
  engine/      Core policy evaluator (pure, deterministic)
  telemetry/   Link state ingestion, experience memory, staleness detection
  audit/       HMAC-SHA256 tamper-evident audit logging
  hcm/         Human Continuity Mode lifecycle management
  adapter/     Southbound adapter trait, mock adapter, Linux netlink adapter, adapter trust registry
  state/       State snapshot for deterministic replay
  api/         HTTP server (Axum), northbound (operator), southbound (equipment) interfaces
  config.rs    Engine configuration (missing telemetry action, staleness threshold)
```

## HTTP API

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/policies` | Load policy set |
| GET | `/api/v1/policies` | Get current policy set |
| POST | `/api/v1/evaluate` | Evaluate traffic class and issue directive |
| POST | `/api/v1/telemetry` | Ingest telemetry record |
| GET | `/api/v1/audit` | Query audit log (supports `?from=&to=` time range) |
| GET | `/api/v1/audit/{id}` | Get specific audit entry |
| POST | `/api/v1/hcm/activate` | Activate Human Continuity Mode |
| POST | `/api/v1/hcm/deactivate` | Deactivate HCM |
| GET | `/api/v1/hcm/state` | Get HCM state |
| GET | `/api/v1/health` | Health check |

Full specification: [`openapi.yaml`](openapi.yaml)

## Telemetry Trust Model

Adapters can be registered with shared secrets for HMAC-SHA256 telemetry verification:
- **Heartbeat detection**: adapters that stop reporting are flagged
- **Sequence monotonicity**: replayed or out-of-order records are rejected
- **HMAC verification**: tampered telemetry records are rejected

## Deployment

```bash
cd deploy
docker compose up
```

See [`deploy/`](deploy/) for Dockerfile, docker-compose, simulated topology, and telemetry poller.

## Formal Verification

TLA+ model in [`formal/`](formal/) verifying policy completeness, conflict determinism, HCM mutual exclusion, and HCM bounded duration.

## Example Policies

- `examples/policies/critical-infrastructure.yaml` — Power grid SCADA telemetry
- `examples/policies/disaster-response.yaml` — HCM activation for emergency medical/public safety
- `examples/policies/multi-provider.yaml` — LEO + LTE + MSS with latency-based selection

## Conformance

This implementation targets Level 1 (Core) and Level 2 (HCM) conformance per `conformance.md`.

## License

Apache 2.0
