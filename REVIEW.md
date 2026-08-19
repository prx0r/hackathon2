# Peer review of the previous Iolaus/HydraBite state

## What was good

- The core insight was strong: tool success is not semantic success.
- Precondition gates + postcondition checks are the right abstraction.
- Tamper-evident receipts are useful.
- The demo failure mode was easy to understand.

## What was not release-ready

1. **Python policy tests were stronger than the HydraDB integration evidence.**
2. The demo used an in-memory `MockHydra`.
3. The public pitch implied durable graph paths that the checked-in engine did not fully write.
4. Packaging/CI still contained inherited unrelated-project state.
5. The benchmark did not have a preregistered statistical protocol.
6. The demo showed three hand-authored cases rather than repeated paired trials.
7. The claim “graph-native” depended on an OpenCypher/graph-node surface not established from the current public hosted HydraDB docs.

## Corrections in this Rust release

- no in-memory benchmark target;
- real HTTP boundary;
- real SQLite ground truth;
- deterministic seeded fault injection;
- paired baseline vs verified arms;
- direct client for documented hosted HydraDB endpoints;
- exact audit vs positive learning signals separated;
- signed PASS receipts;
- full raw-trial result format;
- certification recomputes all summaries;
- cookbook-derived scenario matrix;
- explicit statistical design;
- side-by-side browser demo;
- no fabricated benchmark results.

## Important product correction

The strongest sponsor-native story is not:

> “We added a second graph database inside HydraDB.”

It is:

> “We harden the exact execution-feedback seam in the public HydraDB agent architecture, so HydraDB learns from independently verified outcomes rather than optimistic tool acknowledgements.”

That is both more defensible and more useful.
