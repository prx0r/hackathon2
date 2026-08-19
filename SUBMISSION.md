# Submission draft — Iolaus

## One line

**Iolaus prevents action-capable agents from teaching HydraDB that a task succeeded until an independent verifier confirms the intended world-state postcondition.**

## The problem

HydraDB's Chief-of-Staff architecture is compelling because it turns execution history into future routing intelligence. The orchestrator executes a function, records an outcome, and HydraDB learns from accumulated successes, failures, latency, rejection and user preference.

But a production tool can return a successful transport response without achieving its semantic goal:

- CRM returns 200 but the row is absent;
- deployment command returns success but the service is unhealthy;
- upload returns accepted but indexing is incomplete;
- support escalation returns queued but no human handoff exists;
- a research/financial answer is generated despite zero or mismatched evidence.

If these are stored as `outcome=success`, the system does not only make one wrong statement. It can **poison the memory that future autonomous routing learns from**.

## The primitive

Every action has a small contract:

```text
requires:
  verified prerequisite claims

produces:
  a candidate postcondition

verifier:
  independent readback / health probe / indexing status /
  evidence constraint / human artifact

policy:
  retries, backoff, timeout, verifier class
```

Execution has two phases:

```text
1. EXECUTE
   tool result = observation
   status = SUCCEEDED_UNVERIFIED

2. VERIFY
   independent check of postcondition
   PASS → VERIFIED + signed receipt
   FAIL → REJECTED
```

Only a verified result may:

- satisfy a downstream precondition;
- be recorded as a positive execution outcome;
- become a positive self-improvement signal.

## Why HydraDB

This is not a generic wrapper pasted onto a random agent.

HydraDB's own cookbooks explicitly build:

1. function selection;
2. multi-step planning;
3. persistent execution outcome memory;
4. feedback-driven self-improvement;
5. automated/proactive agents.

That makes **trustworthy outcome semantics** unusually important. The stronger HydraDB's memory loop becomes, the more expensive a false positive becomes because it persists.

## Demo

Two agents receive the exact same plan and exact same injected fault.

### Baseline agent

```text
create_customer
→ HTTP 200 {"success": true}
→ writes outcome=success
→ sends welcome message
→ independent ground truth: customer does NOT exist
→ FALSE TRUSTED SUCCESS
```

### Iolaus agent

```text
create_customer
→ HTTP 200 {"success": true}
→ SUCCEEDED_UNVERIFIED
→ GET /customers/{id}
→ 404
→ REJECTED
→ no positive outcome memory
→ welcome step BLOCKED
```

Then rerun with a healthy target:

```text
create_customer
→ HTTP 200
→ readback exists
→ VERIFIED
→ signed receipt
→ positive learning signal
→ welcome step executes
```

## Benchmark

`HydraFragileBench` uses paired fault injection across cookbook-derived scenarios. The same input/fault is applied to both arms.

Primary endpoint:

**False Trusted-Success Commit Rate.**

Secondary endpoints:

- downstream contamination;
- failure detection recall;
- false block rate;
- verified completion;
- latency overhead;
- cost per verified success.

The output includes Wilson 95% intervals and a paired McNemar test.

## Future

Once verified outcomes exist as first-class signals:

- HydraDB routing can optimise **verified success probability** rather than optimistic tool success;
- flaky functions can be down-weighted from verified empirical evidence;
- verification class can be routed by risk;
- high-risk actions can require stronger/human verifiers;
- autonomous agents can exchange receipt-backed state;
- self-healing plans can route around actions that repeatedly fail postconditions.

The contribution is intentionally narrow:

> **No receipt → no trusted transition.**
