# Live HydraDB + local Hermes integration

This is the **decision/learning-plane experiment**. The controlled verifier experiment remains separate so routing/LLM errors cannot confound the Iolaus treatment effect.

## Environment

```bash
export HYDRA_DB_API_KEY='...'
# HYDRADB_API_KEY is accepted too.

export IOLAUS_HYDRA_TENANT="iolaus-bench-$(date +%s)"
# IOLAUS_HYDRA_DATABASE is accepted too.

# Required for a source-built local run:
export HYDRA_DB_API_URL='http://127.0.0.1:<actual-port>'
# HYDRADB_BASE_URL is accepted too.

export HERMES_BIN=hermes
```

Never leave the endpoint unset and then claim the hosted/default service was the source-built local HydraDB run.

## Gate 1 — local Hermes

```bash
cargo run --release -p iolaus-bench -- hermes-smoke
```

Required: `HERMES_SMOKE_PASS`.

Hermes is used for parameter extraction only. The postcondition verifier is deterministic.

## Gate 2 — semantic Hydra smoke

```bash
cargo run --release -p iolaus-bench -- hydra-smoke
```

This writes a unique nonce to an exact audit memory and repeatedly recalls it. It only prints `HYDRA_SMOKE_PASS` if the same nonce becomes observable.

Thus an HTTP 200 alone cannot pass this gate.

## Gate 3 — bootstrap/routing proof

```bash
cargo run --release -p iolaus-bench -- bootstrap-hydra \
  --functions fixtures/hydradb/functions.json
```

This uploads the function registry, writes benchmark policies/preferences, then repeatedly asks for a deployment route. It only prints `BOOTSTRAP_HYDRA_PASS` if `trigger_deployment` is recovered from the response.

## Gate 4 — live paired run

Start small:

```bash
cargo run --release -p iolaus-bench -- live-run \
  --trials 2 \
  --seed 20260819 \
  --hydra-feedback \
  --out results/live/smoke.json
```

Then:

```bash
cargo run --release -p iolaus-bench -- live-run \
  --trials 25 \
  --seed 20260819 \
  --hydra-feedback \
  --out results/live/hydradb-hermes.json
```

25 trials × 8 scenario families = 200 live decision trials.

Each trial keeps raw evidence for:

- Hydra request hash;
- Hydra raw response + hash;
- selected/expected function;
- Hydra route latency;
- Hermes prompt/schema hashes;
- Hermes raw local stdout + hash;
- parsed JSON and schema validation;
- exact expected-parameter match;
- paired baseline/verified execution and fault assignment;
- signed receipt where verification passes;
- whether each Hydra feedback arm was written;
- whether a false postcondition was promoted as positive learning signal.

## Gate 5 — certify

```bash
cargo run --release -p iolaus-bench -- live-certify \
  results/live/hydradb-hermes.json
python scripts/anti_cheat_audit.py \
  --result results/live/hydradb-hermes.json
```

The certifier recomputes aggregate metrics from raw trials, verifies paired fault equality and receipts, and rejects verified false-positive learning signals. The Python audit additionally hashes the retained raw Hydra/Hermes evidence.

## Arm isolation

```text
iolaus-baseline-execution-log
iolaus-baseline-learning
iolaus-verified-execution-log
iolaus-verified-learning
```

Exact observations may be recorded in either audit log. Positive learning signal is condition-specific and must never cross namespaces.

## Reporting

Report separately:

1. Hydra routing accuracy/latency;
2. Hermes parameter accuracy/latency;
3. Iolaus FTSCR and verification metrics;
4. Hydra learning-loop false-positive contamination.

Do not collapse all four into one score.
