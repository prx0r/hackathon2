#!/usr/bin/env bash
set -euo pipefail
trials="${IOLAUS_CONTROLLED_TRIALS:-1000}"
seed="${IOLAUS_SEED:-20260819}"
out="${IOLAUS_CONTROLLED_OUT:-results/controlled-final.json}"
mkdir -p "$(dirname "$out")" evidence
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 scripts/validate_fixture.py
python3 scripts/anti_cheat_audit.py --root .
cargo run --release -p iolaus-bench -- run --suite benchmarks/hydradb-cookbooks.toml --trials "$trials" --seed "$seed" --out "$out"
cargo run --release -p iolaus-bench -- certify "$out"
python3 scripts/anti_cheat_audit.py --root . --result "$out"
sha256sum "$out" | tee evidence/controlled-result.sha256
echo "CONTROLLED_RELEASE_PASS"
