# Integration into the current Iolaus repository

This handoff is an extension of the existing Rust workspace, not a request to replace it with another prototype.

Recommended coding-agent workflow:

1. Diff this handoff against the current Rust workspace on the target machine.
2. Preserve any newer fixes already made there.
3. Integrate the new live-benchmark modules, Hydra client hardening, fixtures, scripts and docs.
4. Run `cargo fmt`, clippy with warnings denied, and all workspace tests before resolving conflicts in favor of this handoff.
5. Run the controlled benchmark and certify it.
6. Build/start/import the local HydraDB source revision on `/root` and preserve exact evidence.
7. Run local Hermes + Hydra semantic smoke/bootstrap.
8. Run the live benchmark and certify/audit it.
9. Return a patch/diff plus all evidence using `agent_handoff/RETURN_TEMPLATE.md`.

Do not overwrite a newer working implementation merely because this ZIP contains a file with the same name. Reconcile by semantics and rerun every gate.

Do not preserve historical claims that a source-built graph lane or live feedback run passed unless evidence from the exact final revision exists.
