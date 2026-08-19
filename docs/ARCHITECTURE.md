# Architecture

## Trust state machine

```text
             execute
PENDING ───────────────► SUCCEEDED_UNVERIFIED
                              │
                  verifier ┌──┴──┐
                           │     │
                          PASS  FAIL
                           │     │
                           ▼     ▼
                      VERIFIED REJECTED
```

`SUCCEEDED_UNVERIFIED` is deliberate. A successful HTTP call may be useful observation evidence while still being insufficient for trusted state.

## Contract

```rust
ActionContract {
    id,
    requires_claims,
    produces_claim,
    verifier,
    attempts,
    backoff,
}
```

Verifier classes:

- `DbReadback`
- `HealthProbe`
- `QueueReadback`
- `ProcessingStatus`
- `EvidenceCardinality`
- `MetadataConstraint`
- `HumanArtifact` (schema supported; not used in deterministic benchmark)

## Receipt

A PASS receipt binds:

- contract id + digest;
- invocation id;
- canonical input hash;
- tool output hash;
- verifier evidence hash;
- postcondition result;
- timestamp;
- verifier identity/class.

Receipts are Ed25519-signed.

A signature proves **integrity and signer identity**, not semantic truth by itself. Semantic trust comes from the verifier policy and evidence.

## HydraDB seam

The public HydraDB feedback pattern writes exact execution history using `infer:false` and learned patterns using `infer:true`.

Iolaus policy:

```text
exact audit:
  always allowed to record observation/status

positive learning signal:
  VERIFIED only
```

Rejected/unverified outcomes may still be stored as negative/diagnostic signal.

## Benchmark target lab

The target lab is a separate HTTP boundary backed by SQLite:

```text
agent → POST action
             │
             ├── returned JSON
             │
             └── SQLite world state

verifier → independent GET/readback
```

This prevents the verifier from merely re-reading the same optimistic response.

## Paired benchmark

```text
                fault schedule
                     │
              ┌──────┴──────┐
              ▼             ▼
         baseline       verified
              │             │
          same HTTP       same HTTP
          same input      same input
              │             │
      trust tool result   verify truth
              └──────┬──────┘
                     ▼
               paired metrics
```

## Future routing

Verified history enables a cleaner objective:

```text
P(verified success | task, function, context)
------------------------------------------------
expected cost
```

This is more useful than routing from raw `success=true` rates.

That extension is not required for the current benchmark, but the result schema is designed to support it.
