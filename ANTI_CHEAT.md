# Anti-cheat and claims discipline

The benchmark must be harder to fake than the result is to obtain.

## MUST

- Preregister the scenario file, seed, task fixture, and function registry before the final run.
- Give baseline and verified arms the exact same fault assignment.
- Query HydraDB and call Hermes once per paired trial; share those decision-plane observations across arms.
- Keep the semantic fault hidden from HydraDB and Hermes.
- Determine ground truth through a separate state/readback path, not by rereading `success=true`.
- Store raw per-trial observations before aggregating.
- Recompute summary statistics from raw trials in `certify`/`live-certify`.
- Verify every signed receipt.
- Keep baseline and verified HydraDB exact logs and learning signals in different collections.
- Record source Git SHAs, binary hashes, result hashes, environment details, and server commands.
- Verify native import by querying counts/state after import.
- Call source failure/degraded status a failure/unknown state; never coerce it to “zero results.”
- Label cookbook-derived faults as injected fault models, never empirical HydraDB bug rates.

## MUST NOT

- Do not mutate fault probability after looking at results.
- Do not discard hard trials or failed Hermes/Hydra calls from the denominator.
- Do not retry only the verified arm.
- Do not expose strings such as `silent_noop` or `false_green` in the task/LLM prompt.
- Do not use the same optimistic tool output as both execution result and verifier evidence.
- Do not issue a receipt when `postcondition_true=false`.
- Do not count a process exit code as proof that graph data imported.
- Do not call cloud HydraDB while describing the run as source-built local HydraDB.
- Do not silently replace Hermes outputs with expected parameters and then report 100% parameter accuracy.
- Do not merge baseline and verified feedback into the same Hydra collection.
- Do not report `0%` as a universal failure probability; report the Wilson interval and the controlled fixture scope.
- Do not claim the benchmark proves HydraDB is defective. It tests an application-level semantic boundary common to action systems.

## Fail-closed conditions

The release is invalid if any are true:

- arms have different `fault_injected` values;
- a verified receipt exists on a false postcondition;
- the verified arm emits a positive learning signal for a false postcondition;
- native graph counts do not match the fixture manifest;
- source Git revision is unknown;
- live result summary cannot be exactly reproduced from raw trials;
- the final task fixture contains fault mechanism names;
- external LLM/API output was manually edited before certification.

## Evidence tree

A serious final run should contain:

```text
evidence/
  environment-before.txt
  controlled-result.sha256
  live/
    environment.txt
    hermes-smoke.log
    hydra-smoke.log
    bootstrap-hydra.log
    live-run.log
    live-certify.log
    anti-cheat.log
    result.sha256
  native-import/
    import-argv.json
    import.stdout.log
    import.stderr.log
    verify-argv.json
    verify.stdout.log
    verify.stderr.log
    PASS.json

/root/iolaus-hydradb-build/evidence/
  source-discovery.json
  build-env.txt
  cargo-build.log
  server-help.txt
  server-binary.sha256
```
