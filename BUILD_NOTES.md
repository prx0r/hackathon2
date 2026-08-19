# Build notes — external HydraDB/Hermes handoff

Artifact assembled on 2026-08-19.

## Verified in the packaging environment

- all committed TOML parses with Python `tomllib`;
- all committed JSON parses;
- deterministic Hydra fixture regenerated and validated;
- fixture counts: 43 nodes / 19 edges / 13 functions;
- no injected fault labels appear in natural-language live task fixtures;
- anti-cheat static audit passes;
- all shell scripts pass `bash -n`;
- Hydra source-discovery script self-tested on a synthetic Cargo tree exposing `server-runtime`;
- native import adapter self-tested with independent dummy import/count commands;
- no `TODO` / `FIXME` markers in Rust source;
- source tree receives a SHA-256 manifest before packaging;
- ZIP integrity is tested after creation.

## Deliberate non-claims

This packaging environment does **not** have:

- `cargo` / `rustc`;
- the external machine's local Hermes installation;
- the external machine's HydraDB source checkout;
- HydraDB runtime/API credentials.

Therefore this handoff does **not** claim:

- the newly extended Rust workspace compiled here;
- source-built HydraDB `server-runtime` passed here;
- native Hydra graph import passed here;
- `hydra-smoke`, `bootstrap-hydra`, or local Hermes passed here;
- any new live benchmark aggregate number.

Those are explicit external release gates, not missing footnotes. See `EXTERNAL_AGENT_RUNBOOK.md` and `SUCCESS_GATES.md`.

## Required external compile gate

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Only after that passes should the controlled and live benchmarks be run.

## Prior benchmark number

The external brief reports that an earlier executable state produced an 8,000-pair controlled result around `13.5% → 0%` false trusted success with `p<0.001`.

That is **input context**, not a result certified by this packaged source revision. Regenerate and quote the new result file.
