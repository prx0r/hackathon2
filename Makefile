.PHONY: static test bench certify hermes-smoke hydra-smoke bootstrap live live-certify controlled-release live-release all-gates demo

static:
	python scripts/generate_hydra_fixture.py
	python scripts/validate_fixture.py
	python scripts/static_release_check.py
	python scripts/anti_cheat_audit.py
	bash -n scripts/*.sh

test:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace

bench:
	cargo run --release -p iolaus-bench -- run --suite benchmarks/hydradb-cookbooks.toml --trials 1000 --seed 20260819 --out results/full.json

certify:
	cargo run --release -p iolaus-bench -- certify results/full.json

hermes-smoke:
	cargo run -p iolaus-bench -- hermes-smoke

hydra-smoke:
	cargo run -p iolaus-bench -- hydra-smoke

bootstrap:
	cargo run -p iolaus-bench -- bootstrap-hydra

live:
	cargo run --release -p iolaus-bench -- live-run --trials 25 --seed 20260819 --hydra-feedback --out results/live/final.json

live-certify:
	cargo run --release -p iolaus-bench -- live-certify results/live/final.json

controlled-release:
	./scripts/run_controlled_release.sh

live-release:
	./scripts/run_live_pipeline.sh

all-gates:
	./scripts/run_all_gates.sh

demo:
	cargo run --release -p iolaus-demo -- --port 8080
