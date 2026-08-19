# Required return from the coding agent

Do not answer only “done”. Return this evidence summary plus the directories/files.

## Revisions

```text
Iolaus git SHA:
HydraDB git SHA:
HydraDB binary SHA-256:
Rust/Cargo versions:
Hermes CLI version:
Hermes provider/model label:
Hydra endpoint used:
```

Never include the API key itself.

## Gate status

```text
STATIC_RELEASE_CHECK: PASS/FAIL
RUST_FMT: PASS/FAIL
RUST_CLIPPY_-D_WARNINGS: PASS/FAIL
RUST_TESTS: PASS/FAIL
CONTROLLED_RELEASE: PASS/FAIL
HYDRADB_SOURCE_BUILD: PASS/FAIL
HYDRADB_NATIVE_IMPORT: PASS/FAIL
HERMES_SMOKE: PASS/FAIL
HYDRA_SMOKE_WRITE_RECALL: PASS/FAIL
BOOTSTRAP_ROUTE_PROBE: PASS/FAIL
LIVE_HYDRADB_HERMES: PASS/FAIL
LIVE_CERTIFY: PASS/FAIL
ANTI_CHEAT: PASS/FAIL
ALL_GATES: PASS/FAIL
```

A failed gate stays failed. Do not relabel it SKIP/PASS.

## Controlled benchmark

```text
paired trials:
baseline FTSCR [Wilson 95% CI]:
verified FTSCR [Wilson 95% CI]:
paired McNemar p:
verifier detection recall:
false-block rate:
downstream contamination baseline/verified:
p50/p95 verifier overhead:
result SHA-256:
```

## Live decision plane

```text
live trials:
Hydra routing accuracy [CI]:
Hydra route latency:
Hermes schema-valid extraction [CI]:
Hermes expected-parameter exact-match [CI]:
Hermes latency:
```

## Live verification / learning plane

```text
baseline live FTSCR:
verified live FTSCR:
baseline false-positive learning signals:
verified false-positive learning signals:
result SHA-256:
```

## Native graph proof

```text
expected nodes/edges: 43 / 19
observed nodes/edges:
import command evidence file:
verification command evidence file:
PASS.json SHA-256:
```

## Discrepancies

List every place where the checked-out HydraDB revision differed from assumptions in the handoff: server feature/package/bin, launch args, local gateway, import format, REST request shape, tenant/database terminology, indexing delays, etc.

## Attach/preserve

```text
evidence/
results/
/root/iolaus-hydradb-build/evidence/
any deterministic HydraDB fixture converter added by the agent
patch/diff to this handoff
```
