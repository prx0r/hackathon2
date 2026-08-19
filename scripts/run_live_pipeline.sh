#!/usr/bin/env bash
set -euo pipefail
trials="${IOLAUS_LIVE_TRIALS:-5}"
seed="${IOLAUS_SEED:-20260819}"
out="${IOLAUS_LIVE_OUT:-results/live/hydradb-hermes.json}"
mkdir -p "$(dirname "$out")" evidence/live

: "${HYDRA_DB_API_KEY:=${HYDRADB_API_KEY:-}}"
: "${IOLAUS_HYDRA_TENANT:=${IOLAUS_HYDRA_DATABASE:-}}"
[[ -n "$HYDRA_DB_API_KEY" ]] || { echo "HYDRA_DB_API_KEY/HYDRADB_API_KEY required" >&2; exit 2; }
[[ -n "$IOLAUS_HYDRA_TENANT" ]] || { echo "IOLAUS_HYDRA_TENANT/IOLAUS_HYDRA_DATABASE required" >&2; exit 2; }
export HYDRA_DB_API_KEY IOLAUS_HYDRA_TENANT

scripts/capture_environment.sh evidence/live/environment.txt
cargo run --release -p iolaus-bench -- hermes-smoke | tee evidence/live/hermes-smoke.log
cargo run --release -p iolaus-bench -- hydra-smoke | tee evidence/live/hydra-smoke.log
cargo run --release -p iolaus-bench -- bootstrap-hydra --functions fixtures/hydradb/functions.json | tee evidence/live/bootstrap-hydra.log
cargo run --release -p iolaus-bench -- live-run \
  --suite benchmarks/hydradb-cookbooks.toml \
  --tasks fixtures/tasks/hydradb-live-tasks.json \
  --functions fixtures/hydradb/functions.json \
  --trials "$trials" --seed "$seed" --hydra-feedback --out "$out" | tee evidence/live/live-run.log
cargo run --release -p iolaus-bench -- live-certify "$out" | tee evidence/live/live-certify.log
python3 scripts/anti_cheat_audit.py --root . --result "$out" | tee evidence/live/anti-cheat.log
sha256sum "$out" | tee evidence/live/result.sha256

echo "LIVE_HYDRADB_HERMES_PIPELINE_PASS"
