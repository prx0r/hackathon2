# Iolaus — verified state transitions for HydraDB agents

> **A tool saying `success=true` is an observation. It is not proof that the intended world-state transition occurred.**

Iolaus is a Rust verification layer plus an auditable benchmark harness for action-capable agents using HydraDB-style function routing, memory, and execution feedback.

HydraDB answers the decision-plane question:

> **What should the agent do?**

Iolaus adds the trust-boundary question:

> **When are we allowed to believe it worked?**

```text
HydraDB route / retrieve function
          ↓
local Hermes extracts arguments
          ↓
application executes tool
          ↓
      tool says success
          ↓
     SUCCEEDED_UNVERIFIED
          ↓
       IOLAUS GATE
          ↓
independent postcondition evidence
      ┌────────┴────────┐
      ↓                 ↓
   VERIFIED          REJECTED
      ↓
positive learning signal / downstream prerequisite
```

The invariant is:

> **No verified receipt → no trusted success transition.**

## What this handoff contains

Four Rust crates remain the executable core:

- `iolaus-core` — action contracts, state-transition model, verifier evidence, signed receipts, paired statistics.
- `iolaus-hydra` — HydraDB HTTP integration and arm-isolated execution/learning logs.
- `iolaus-bench` — controlled benchmark, live HydraDB + Hermes benchmark, result certification.
- `iolaus-demo` — side-by-side browser demonstration of optimistic success vs verified success.

The handoff adds:

- deterministic Hydra fixture generator: customers, functions, memories, execution history;
- `nodes.jsonl` / `edges.jsonl` native graph fixtures;
- 8 natural-language live task families corresponding to the controlled semantic-failure suite;
- local Hermes schema-constrained parameter extractor;
- HydraDB source build discovery/build/start/import adapters designed for `/root`, not `/tmp`;
- two independent Hydra integration gates: API/decision plane and source-built native graph lane;
- live paired benchmark with raw Hydra responses and raw Hermes output retained for audit;
- feedback-contamination metrics showing whether false success is promoted into learning memory;
- fail-closed result certifiers and anti-cheat audits;
- an external coding-agent runbook with exact required evidence.

Start with **`EXTERNAL_AGENT_RUNBOOK.md`**.

## Two benchmark layers

### Layer 1 — controlled verifier causal benchmark

The existing local target is a real HTTP service backed by SQLite. Each baseline/verified pair receives the same scenario, seed, intended operation, and injected semantic fault.

Full preregistered run:

```bash
./scripts/run_controlled_release.sh
```

Default suite: 8 scenarios × 1,000 paired trials = **8,000 paired trials**.

Primary endpoint:

```text
False Trusted-Success Commit Rate (FTSCR)

FTSCR = trusted_success AND postcondition_false / all trials
```

This isolates the causal question: **does the verification gate prevent false state transitions from becoming trusted success?**

### Layer 2 — live HydraDB + Hermes decision/learning benchmark

```bash
./scripts/run_live_pipeline.sh
```

Each live trial separately records:

1. **HydraDB routing** — natural-language task → expected function;
2. **Hermes extraction** — task + expected function schema → structured arguments;
3. **Iolaus treatment** — same intended action + same seeded fault → baseline versus verified arm;
4. **learning-loop integrity** — whether either arm emits a positive HydraDB learning signal while the postcondition is false.

This separation is intentional. A Hydra routing error or Hermes extraction error is **measured**, but it is not allowed to silently change which action/fault the treatment arms receive. Otherwise the benchmark would confound decision quality with verifier quality.

Live metrics include:

- Hydra top-route accuracy;
- Hermes schema-valid extraction rate;
- Hermes exact expected-parameter rate;
- baseline/verified FTSCR;
- false-positive learning-signal rate;
- downstream contamination;
- verifier detection recall and false-block rate;
- latency/overhead/cost units;
- paired McNemar significance and Wilson 95% intervals.

## Why the feedback metric matters

Hydra-style agent systems can store execution outcomes back into memory and use them as future routing/self-improvement signal.

Without a verification boundary:

```text
tool success=true
      ↓
positive execution memory
      ↓
future routing learns from a false outcome
```

With Iolaus:

```text
tool success=true
      ↓
independent postcondition
      ↓
VERIFIED only
      ↓
positive execution memory
```

The live benchmark therefore writes four isolated namespaces:

```text
iolaus-baseline-execution-log
iolaus-baseline-learning
iolaus-verified-execution-log
iolaus-verified-learning
```

Never merge the arms during an experiment.

## HydraDB source-built lane

The environment described by the handoff has a small `/tmp` tmpfs. **Do not build HydraDB there.**

Use:

```bash
export HYDRADB_SOURCE=/root/hydradb-build
export IOLAUS_HYDRA_BUILD_ROOT=/root/iolaus-hydradb-build
./scripts/build_hydradb_source.sh
```

The script:

- checks free disk before starting;
- redirects `TMPDIR` and `CARGO_TARGET_DIR` under `/root`;
- discovers the exact source revision/package/bin/features before building;
- requires literal evidence for `server-runtime` unless explicitly overridden;
- builds release with `--features server-runtime`;
- hashes the produced binary and records `--help` output.

Starting the binary is deliberately revision-specific. `scripts/start_hydradb_source.sh` **refuses to guess arguments**. Supply exact `HYDRADB_RUN_ARGV_JSON` derived from that build's help/source.

Native import is also fail-closed. `scripts/native_import_adapter.py` runs exact argv arrays you supply for that source revision and only prints PASS if a separate verification command reports exactly the fixture node/edge counts.

See `HYDRADB_SOURCE_BUILD.md`.

## Fixture

Generate and validate:

```bash
python scripts/generate_hydra_fixture.py
python scripts/validate_fixture.py
```

The deterministic fixture contains:

- 13 function-schema nodes;
- 6 CRM/customer nodes;
- 6 memory objects;
- 12 execution-history records;
- supporting policy/identity objects;
- relation edges between customers, memories, functions and executions.

No fault assignment is encoded in the fixture or natural-language tasks. Faults are generated only by the seeded benchmark scheduler.

## Local Hermes

The benchmark deliberately keeps the LLM out of the verifier. Hermes is used only for the probabilistic decision-plane task of parameter extraction.

```bash
export HERMES_BIN=hermes
# optional if the local installation requires explicit values:
export HERMES_PROVIDER=...
export HERMES_INFERENCE_MODEL=...

cargo run -p iolaus-bench -- hermes-smoke
```

Every extraction retains the prompt hash, schema hash, raw stdout, raw stdout hash, parsed JSON, validation result, model/provider labels and latency.

## Required live environment

Either spelling of the key is accepted:

```bash
export HYDRA_DB_API_KEY=...
# or HYDRADB_API_KEY

export IOLAUS_HYDRA_TENANT=iolaus-bench-$(date +%s)
# or IOLAUS_HYDRA_DATABASE

# Local source-built endpoint if applicable:
export HYDRA_DB_API_URL=http://127.0.0.1:<port>
# or HYDRADB_BASE_URL
```

Then:

```bash
cargo run -p iolaus-bench -- hydra-smoke
cargo run -p iolaus-bench -- bootstrap-hydra
cargo run --release -p iolaus-bench -- live-run \
  --suite benchmarks/hydradb-cookbooks.toml \
  --tasks fixtures/tasks/hydradb-live-tasks.json \
  --functions fixtures/hydradb/functions.json \
  --trials 25 \
  --seed 20260819 \
  --hydra-feedback \
  --out results/live/final.json
cargo run --release -p iolaus-bench -- live-certify results/live/final.json
python scripts/anti_cheat_audit.py --result results/live/final.json
```

`hydra-smoke` is not a shallow HTTP test: it writes a unique nonce and requires that exact nonce to become observable by recall after bounded retries.

`bootstrap-hydra` is not a shallow ingestion test: it loads the registry/policies and requires a deployment query to retrieve `trigger_deployment` after bounded retries.

## Anti-cheat doctrine

A benchmark result is invalid if any of these occur:

- baseline and verified arms receive different fault assignments;
- scenario text tells Hermes/Hydra what fault will be injected;
- verifier reads the action's optimistic response as its postcondition evidence;
- positive learning signals from baseline and verified arms share a namespace;
- a verified receipt exists when ground truth is false;
- fault generation depends on Hydra/Hermes/model output;
- aggregate JSON is edited without raw trial regeneration;
- external/cloud Hydra is used while claiming the source-built local graph lane passed;
- source build/import PASS is inferred from an API smoke test;
- LLM output is used as the verifier for deterministic state.

See `ANTI_CHEAT.md`.

## One-command release gates

```bash
./scripts/run_all_gates.sh
```

By default this requires:

1. Rust fmt/clippy/tests;
2. controlled 8,000-pair run + certification;
3. fixture/static anti-cheat validation;
4. local Hermes smoke;
5. Hydra write→recall smoke;
6. Hydra function bootstrap→route proof;
7. live paired benchmark + certification;
8. source-built Hydra binary evidence;
9. native graph import count proof.

The source-build and native-import lanes are deliberately not inferred from the API lane.

## Evidence hierarchy

A useful claim must point to an artifact:

```text
"Rust benchmark passes"
→ cargo test/clippy logs

"HydraDB is connected"
→ unique write→recall evidence

"test data loaded"
→ native import count proof + bootstrap routing proof

"Hermes is local"
→ environment record + raw local subprocess evidence

"Iolaus prevents false success"
→ certified paired raw trials

"feedback is cleaner"
→ arm-isolated Hydra logs + false-positive learning metric
```

Do not present the previous reported `13.5% → 0%` result as this release's measured result until the external host regenerates it from this exact source tree.

## External handoff

The intended execution environment has Rust, Docker, HydraDB source, and local Hermes. This packaging environment does **not** contain a Rust toolchain or the private/local HydraDB checkout, so this ZIP is statically validated here but the final compile/native/live gates must execute on that host.

Run `EXTERNAL_AGENT_RUNBOOK.md` from top to bottom and preserve the complete `evidence/`, `logs/`, and `results/` trees.
