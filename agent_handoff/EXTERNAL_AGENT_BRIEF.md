# Task for the coding agent

You are finishing the Iolaus HydraDB integration on a machine with Rust, Docker, local Hermes, and a HydraDB source checkout.

## Goal

Produce machine-verifiable evidence for:

```text
HYDRA source build with server-runtime: PASS
native fixture import + count query: PASS
hermes-smoke: PASS
hydra-smoke: PASS
bootstrap-hydra: PASS
controlled 8,000-pair benchmark + certify: PASS
live HydraDB + Hermes benchmark + live-certify: PASS
anti-cheat audit: PASS
```

## Important machine constraint

`/tmp` is a small tmpfs. Do not compile HydraDB there. Use `/root/iolaus-hydradb-build` for `TMPDIR` and `CARGO_TARGET_DIR`.

## Work order

1. Read `EXTERNAL_AGENT_RUNBOOK.md`, `ANTI_CHEAT.md`, `SUCCESS_GATES.md`.
2. Run static fixture/audit checks.
3. Move/reclone HydraDB to `/root/hydradb-build` if needed.
4. Run `scripts/build_hydradb_source.sh`.
5. Inspect the exact revision's server help/source; configure and start it.
6. Discover the native import contract; adapt fixture deterministically if required.
7. Prove post-import graph counts using `native_import_adapter.py`.
8. Configure the documented API endpoint to the local source-built service/gateway. Never silently use cloud HydraDB.
9. Run Hermes smoke, Hydra smoke, bootstrap.
10. Run controlled benchmark and certify.
11. Run live benchmark with `IOLAUS_LIVE_TRIALS=2`, fix integration issues, then 25.
12. Run all gates.
13. Return the evidence directory, result JSONs, Git SHAs, binary hashes, and a concise discrepancy report.

## Do not change benchmark semantics to make it pass

Fix adapters/environment/source integration. Do not:

- lower fault probabilities after seeing results;
- expose fault labels to Hermes;
- skip failed calls;
- replace Hermes output with expected params;
- share feedback collections between arms;
- turn verifier errors into passes;
- remove the count/readback checks;
- report source-build success from the old Docker image.

If the checked-out HydraDB revision's local API differs from the public REST API, write a small explicit adapter and document it. Do not fake endpoint compatibility.

## Return format

Use `agent_handoff/RETURN_TEMPLATE.md`. Attach/preserve raw evidence and discrepancy notes; do not return only a prose success summary.
