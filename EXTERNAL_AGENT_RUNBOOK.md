# External coding-agent runbook — HydraDB + Hermes + Iolaus

This directory is a **handoff release**, not a claim that the source-built HydraDB lane has already run on this machine.

The target external machine described in `agent_handoff/EXTERNAL_AGENT_BRIEF.md` has:

- Rust/Cargo installed;
- Docker installed;
- ~18 GB free under `/root`;
- `/tmp` on a ~3.8 GB tmpfs and therefore unsuitable for a HydraDB source build;
- HydraDB source already cloned at `/tmp/hydradb-build/` or movable to `/root`;
- local Hermes available;
- no required external LLM API key.

## Definition of done

All five statements must be true and backed by files, not terminal memory:

1. **Iolaus controlled benchmark passes**: Rust tests, 8,000 paired trials, certification.
2. **HydraDB source build passes** with the checked-out revision's `server-runtime` feature, using a build root on `/root`, never `/tmp`.
3. **Native HydraDB fixture import passes** and a query/count command proves all expected fixture nodes/edges are present.
4. **HydraDB documented API lane passes**: `hydra-smoke`, `bootstrap-hydra`, recall of registered functions.
5. **Live HydraDB + Hermes benchmark passes**: HydraDB routing, Hermes parameter extraction, paired Iolaus verification, separate feedback namespaces, `live-certify`, anti-cheat audit.

A process merely staying alive is not a PASS. An import process returning zero is not a PASS. A `200` response is not a semantic PASS. That is the entire thesis of Iolaus; the benchmark must obey its own rule.

---

## 0. Put builds on real disk

Recommended:

```bash
mkdir -p /root/hackathon2 /root/iolaus-hydradb-build
cp -a /path/to/Iolaus-hydradb-livebench-handoff /root/hackathon2/Iolaus

# If HydraDB was cloned under /tmp, move or reclone it to real storage.
rsync -a --delete /tmp/hydradb-build/ /root/hydradb-build/
export HYDRADB_SRC=/root/hydradb-build
export HYDRADB_BUILD_ROOT=/root/iolaus-hydradb-build
```

Do not set `CARGO_TARGET_DIR` under `/tmp`. `scripts/build_hydradb_source.sh` sets both `TMPDIR` and `CARGO_TARGET_DIR` under `/root`.

Capture the machine before changing it:

```bash
cd /root/hackathon2/Iolaus
scripts/capture_environment.sh evidence/environment-before.txt
```

---

## 1. Validate the handoff before touching HydraDB

```bash
python3 scripts/generate_hydra_fixture.py
python3 scripts/validate_fixture.py
python3 scripts/static_release_check.py .
python3 scripts/anti_cheat_audit.py --root .
```

Expected fixture counts:

- 43 nodes;
- 19 edges;
- 13 registered functions;
- 6 CRM customer context nodes;
- 12 execution-history nodes;
- 6 memory/policy nodes.

The fixture does **not** contain per-trial fault assignments. Fault assignment is deterministic inside Rust from `(seed, scenario_id, trial_index)`.

---

## 2. Build HydraDB from source on `/root`

Install native dependencies if necessary:

```bash
export IOLAUS_INSTALL_BUILD_DEPS=1
scripts/build_hydradb_source.sh "$HYDRADB_SRC"
```

Or install packages manually, then leave the flag unset:

```bash
sudo apt-get update
sudo apt-get install -y \
  libcypher-parser-dev libgraphblas-dev cmake pkg-config clang build-essential

scripts/build_hydradb_source.sh "$HYDRADB_SRC"
```

The script:

- checks free space before compiling;
- verifies `libcypher-parser` and GraphBLAS are detectable;
- records the HydraDB Git SHA;
- searches the exact source revision for the `server-runtime` feature;
- sets `TMPDIR=/root/iolaus-hydradb-build/tmp`;
- sets `CARGO_TARGET_DIR=/root/iolaus-hydradb-build/target`;
- runs `cargo build --release --features server-runtime`;
- records build logs, executable candidates, binary hash, and `--help` output.

Required evidence:

```text
/root/iolaus-hydradb-build/evidence/source-discovery.json
/root/iolaus-hydradb-build/evidence/build-env.txt
/root/iolaus-hydradb-build/evidence/cargo-build.log
/root/iolaus-hydradb-build/evidence/server-binary.sha256
/root/iolaus-hydradb-build/evidence/server-help.txt
```

If the workspace requires `-p <package>`, inspect `source-discovery.json` and set:

```bash
export HYDRADB_PACKAGE=<exact package from this revision>
```

Do not guess package or binary names in the final evidence.

---

## 3. Start the source-built server

The handoff intentionally does **not** invent HydraDB server CLI arguments because source revisions can differ.

Inspect:

```bash
cat /root/iolaus-hydradb-build/evidence/server-help.txt
rg -n "listen|bind|port|server|serve|api|cypher|import" "$HYDRADB_SRC"
```

Then set the exact arguments for this revision:

```bash
export HYDRADB_RUN_ARGV_JSON='["serve","--config","/exact/path"]'  # example SHAPE only; use this revision's actual argv
scripts/start_hydradb_source.sh
```

That script only proves process liveness. It prints no `HYDRA_SMOKE_PASS`.

Set the API endpoint exposed by the server/gateway:

```bash
export HYDRA_DB_API_URL=http://127.0.0.1:<actual-port>
export HYDRA_DB_API_KEY=<local/dev key required by this revision>
export IOLAUS_HYDRA_TENANT=iolaus-live-$(date +%s)
```

If the source-built graph node does not itself expose the documented HydraDB memory/recall REST surface, run the exact local gateway from the HydraDB repo and point `HYDRA_DB_API_URL` at that gateway. Record its Git SHA/process command too. **Do not silently fall back to `https://api.hydradb.com` while claiming a local-source run.**

---

## 4. Import the native graph fixture

The fixture is portable Iolaus JSONL. The checked-out HydraDB revision is the authority for native import syntax and field mapping.

Inspect the source first:

```bash
python3 scripts/discover_hydradb_source.py "$HYDRADB_SRC" \
  --out evidence/hydradb-source-discovery.json

rg -n "nodes\.jsonl|edges\.jsonl|import|bulk|loader|GraphBLAS|Cypher" "$HYDRADB_SRC"
```

If HydraDB's native loader accepts the committed fixture shape directly, use it. If its field names differ, write a deterministic converter under `scripts/` and test the converter. Do not edit the source fixture by hand between arms.

`native_import_adapter.py` executes only exact argv arrays supplied by the coding agent:

```bash
export HYDRADB_IMPORT_ARGV_JSON='["/exact/binary","...","{nodes}","...","{edges}"]'
export HYDRADB_VERIFY_ARGV_JSON='["/exact/query-command","..."]'
python3 scripts/native_import_adapter.py
```

The verification command must print JSON only:

```json
{"nodes":43,"edges":19}
```

Only then does the script write:

```text
evidence/native-import/PASS.json
```

This avoids an import-command exit code being misreported as proof the graph is present.

---

## 5. Prove local Hermes

HydraDB's Chief-of-Staff architecture delegates parameter extraction to an app-layer LLM and says that provider can be replaced. This benchmark uses local Hermes.

First verify Hermes itself:

```bash
hermes --version
cargo run --release -p iolaus-bench -- hermes-smoke
```

Optional exact model/provider pin:

```bash
export HERMES_PROVIDER=<configured local provider>
export HERMES_INFERENCE_MODEL=<configured model>
```

The Rust adapter uses one-shot `hermes -z`, asks for one JSON object, hashes the prompt/schema/raw response, parses the object, then validates required fields and primitive JSON types.

Fault state is never provided to Hermes.

---

## 6. HydraDB API smoke + bootstrap

```bash
cargo run --release -p iolaus-bench -- hydra-smoke
cargo run --release -p iolaus-bench -- bootstrap-hydra \
  --functions fixtures/hydradb/functions.json
```

`hydra-smoke` writes a unique exact memory then performs recall. `bootstrap-hydra` uploads the 13 function schemas to `iolaus-functions`, writes benchmark policies/preferences, and runs a deploy-routing probe.

Required markers:

```text
HYDRA_SMOKE_PASS
BOOTSTRAP_HYDRA_PASS
```

A server `200` by itself is insufficient.

---

## 7. Controlled release benchmark

Run the causal verification study without LLM/routing noise:

```bash
IOLAUS_CONTROLLED_TRIALS=1000 scripts/run_controlled_release.sh
```

This is 1,000 paired trials × 8 scenarios = 8,000 pairs.

Both arms receive the same seeded semantic fault. The treatment is only whether postcondition evidence is required before success becomes trusted.

`certify` recomputes aggregate statistics from raw trials and verifies every Ed25519 receipt.

Do not paste the existing 13.5% → 0% figure into a final submission unless this machine's generated result reproduces it. Use the actual result file.

---

## 8. Live HydraDB + Hermes benchmark

Start small:

```bash
IOLAUS_LIVE_TRIALS=2 scripts/run_live_pipeline.sh
```

Then a presentation-quality run:

```bash
IOLAUS_LIVE_TRIALS=25 scripts/run_live_pipeline.sh
```

25 × 8 = 200 live decision trials. Every trial measures:

1. **HydraDB route**: task → ranked function knowledge;
2. **Hermes extraction**: task + expected JSON schema → parameters;
3. **Iolaus paired execution**: same intended action + same fault, baseline vs verified;
4. **HydraDB feedback**: exact logs and learning signals in arm-separated collections.

The design deliberately gives Hermes the expected function schema rather than whichever route HydraDB returned. This keeps routing accuracy and extraction accuracy separately measurable and prevents a routing error from changing the causal treatment in the verifier comparison.

The raw result contains:

- task text;
- Hydra request/response hashes plus the raw Hydra response;
- selected and expected function;
- routing correctness;
- Hermes prompt/schema/output hashes plus raw local Hermes stdout;
- parsed parameters;
- schema-valid and exact-match flags;
- paired fault assignment;
- baseline/verified outcomes;
- signed verified receipts;
- learning-signal contamination flags.

Certification:

```bash
cargo run --release -p iolaus-bench -- live-certify results/live/hydradb-hermes.json
python3 scripts/anti_cheat_audit.py --root . --result results/live/hydradb-hermes.json
```

---

## 9. Final all-gates command

After native source build/import evidence exists:

```bash
IOLAUS_REQUIRE_NATIVE_HYDRA=1 scripts/run_all_gates.sh
```

No final “working with real HydraDB” claim before this succeeds.

---

## What to report

Report three distinct classes of results:

### A. Decision plane

- HydraDB top-1 route accuracy;
- Hermes schema-valid parameter rate;
- Hermes exact parameter match rate;
- HydraDB route latency;
- Hermes extraction latency.

### B. Verification plane

- False Trusted-Success Commit Rate (FTSCR);
- false-positive fraction among trusted successes;
- failure-detection recall;
- false-block rate;
- downstream contamination;
- true completion rate;
- P50/P95 latency and verifier overhead;
- McNemar p-value;
- Wilson intervals.

### C. Learning-loop integrity

- baseline positive learning signals written for false postconditions;
- verified positive learning signals written for false postconditions;
- exact/audit log write success;
- separated baseline/verified collections.

The core expected qualitative result is not “HydraDB bad.” It is:

> HydraDB can improve function selection from execution history; therefore applications should make the feedback event stronger than `tool_return.success == true`. Iolaus promotes only independently verified state transitions into trusted success/positive learning signal.
