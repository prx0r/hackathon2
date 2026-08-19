# Success gates

| Gate | Command/evidence | Required result |
|---|---|---|
| Static fixture | `python3 scripts/validate_fixture.py` | `PASS`, 43 nodes, 19 edges |
| Static benchmark audit | `python3 scripts/anti_cheat_audit.py --root .` | `ANTI_CHEAT_AUDIT_PASS` |
| Rust quality | fmt + clippy `-D warnings` + workspace tests | all pass |
| Controlled benchmark | `scripts/run_controlled_release.sh` | controlled certification pass |
| Hydra source discovery | `discover_hydradb_source.py` | `server-runtime` found |
| Hydra source build | `build_hydradb_source.sh` | binary + SHA + build log |
| Native import | `native_import_adapter.py` | post-import counts exactly 43/19 |
| Hermes | `iolaus-bench hermes-smoke` | JSON `ok=true` |
| Hydra API | `iolaus-bench hydra-smoke` | write + recall + marker |
| Hydra bootstrap | `iolaus-bench bootstrap-hydra` | functions uploaded + routing probe |
| Live benchmark | `scripts/run_live_pipeline.sh` | raw result + live certification |
| Final | `scripts/run_all_gates.sh` | `IOLAUS_ALL_GATES_PASS` |

## Release-claim mapping

Only claim **“Rust benchmark works”** after controlled gate.

Only claim **“integrates with HydraDB API”** after API smoke/bootstrap/live gate.

Only claim **“runs against source-built HydraDB with full Cypher runtime”** after source build + native import + local endpoint evidence.

Only claim **“uses local Hermes for parameter extraction”** after Hermes smoke and a live result containing non-null extraction records.
