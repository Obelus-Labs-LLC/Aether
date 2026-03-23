# Aether TLA+ Formal Model

Formal verification of the Aether policy evaluation engine using TLA+ and the TLC model checker.

## Verified Invariants

1. **PolicyCompleteness**: Every traffic class evaluation produces a defined decision (policy match or default)
2. **HcmMutualExclusion**: No concurrent HCM activations with different activation IDs
3. **HcmBoundedDuration**: Active HCM duration never exceeds the configured maximum
4. **HcmCumulativeBound**: Cumulative HCM duration across all activations never exceeds the configured maximum

## Running

Requires Java and the TLA+ tools:

```bash
# Install TLC (TLA+ model checker)
# Download from https://github.com/tlaplus/tlaplus/releases

# Run model checker
java -jar tla2tools.jar -config AetherSpec.cfg AetherSpec.tla
```

## Model Scope

The model uses a finite state space:
- 2 policies, 2 links, 3 traffic classes
- HCM duration limit of 5 ticks, cumulative limit of 15 ticks

This is sufficient to verify the core invariants. Larger models can be configured by adjusting the constants in `AetherSpec.cfg`.
