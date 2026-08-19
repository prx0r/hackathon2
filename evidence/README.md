# Evidence directory

This directory is intentionally empty of release PASS claims in the packaged handoff.

The external execution host should populate it with:

```text
evidence/environment-before.txt
evidence/controlled/*
evidence/live/*
evidence/native-import/PASS.json
evidence/hydradb-source-discovery.json
```

The source-build script writes its native build evidence under the configured build root, normally:

```text
/root/iolaus-hydradb-build/evidence/
```

Do not add a PASS file manually. PASS artifacts must be emitted by the corresponding verification script.
