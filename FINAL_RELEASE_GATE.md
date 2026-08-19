# Final release gate

No benchmark number or HydraDB integration claim is release-ready until the corresponding gate below produces evidence.

## 0. Static handoff sanity

```bash
python scripts/generate_hydra_fixture.py
python scripts/validate_fixture.py
python scripts/static_release_check.py
python scripts/anti_cheat_audit.py
bash -n scripts/*.sh
```

## 1. Rust + controlled verifier benchmark

```bash
./scripts/run_controlled_release.sh
```

Required terminal marker: `CONTROLLED_RELEASE_PASS`.

This runs fmt, clippy with warnings denied, workspace tests, the preregistered 1,000-pairs/scenario suite, result certification and anti-cheat checks.

## 2. Source-built HydraDB proof

Build on real disk, not `/tmp`:

```bash
export HYDRADB_SOURCE=/root/hydradb-build
export IOLAUS_HYDRA_BUILD_ROOT=/root/iolaus-hydradb-build
./scripts/build_hydradb_source.sh
```

Required terminal marker: `HYDRADB_SOURCE_BUILD_PASS`.

Then supply revision-correct server args:

```bash
export HYDRADB_RUN_ARGV_JSON='["...exact argv elements from --help/source..."]'
./scripts/start_hydradb_source.sh
```

Process liveness is **not** Hydra API PASS.

## 3. Native graph import proof

Supply exact import + independent count-verification argv for the built revision:

```bash
export HYDRADB_IMPORT_ARGV_JSON='["/path/to/importer","...","{nodes}","{edges}"]'
export HYDRADB_VERIFY_ARGV_JSON='["/path/to/verifier","..."]'
python scripts/native_import_adapter.py
```

Verification must output JSON containing exactly:

```json
{"nodes":43,"edges":19}
```

Required terminal marker: `NATIVE_HYDRADB_IMPORT_PASS`.

## 4. Local Hermes + Hydra API/decision-plane + learning-loop proof

```bash
export HERMES_BIN=hermes
export HYDRA_DB_API_KEY='...'
export IOLAUS_HYDRA_TENANT="iolaus-bench-$(date +%s)"
export HYDRA_DB_API_URL='http://127.0.0.1:<source-built-port>'

./scripts/run_live_pipeline.sh
```

Required terminal marker: `LIVE_HYDRADB_HERMES_PIPELINE_PASS`.

`hydra-smoke` must prove its unique write can be recalled. `bootstrap-hydra` must prove the function registry actually routes a deployment query.

## 5. All gates

```bash
./scripts/run_all_gates.sh
```

Required terminal marker: `IOLAUS_ALL_GATES_PASS`.

## Reporting rule

Record actual generated values only. Never copy the previous benchmark's `13.5% → 0%` into the final pitch until this exact release has regenerated and certified the result.
