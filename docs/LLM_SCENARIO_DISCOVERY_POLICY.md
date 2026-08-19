# LLM-assisted scenario discovery — non-scored lane

Local Hermes may be useful for proposing additional fragile integration cases, but it must **not** dynamically generate faults during the scored benchmark.

Safe workflow:

```text
cookbook / API contract
      ↓
Hermes proposes candidate semantic failure
      ↓
human/coding-agent converts it into:
  precondition
  optimistic action response
  independent ground truth
  postcondition verifier
      ↓
freeze scenario in source control
      ↓
assign ID + deterministic fault model
      ↓
run tests on both healthy and faulty worlds
      ↓
only then add to a future benchmark version
```

Do not let the same LLM see a scored trial and choose whether/how it should fail. That would make scenario difficulty model-dependent and destroy reproducibility.

An LLM-generated candidate must be rejected unless an independent deterministic oracle can label the final postcondition for that scenario.
