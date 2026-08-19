# HydraFragileBench — preregistered benchmark protocol

## Research question

When an agent action reports transport/application success, does verifier-gated promotion reduce **false trusted-success commits** and **unsafe downstream actions** compared with trusting the tool response directly?

The benchmark does **not** test whether HydraDB's retrieval or routing is better than Iolaus. It holds routing/planning constant and isolates outcome verification.

## Experimental unit

One paired trial:

```text
same scenario
same semantic input
same injected fault
same target implementation
same declared postcondition

        ┌──────── baseline arm
trial ──┤
        └──────── verified arm
```

Namespaces are separate so one arm cannot modify the other's ground truth.

## Primary endpoint

### False Trusted-Success Commit Rate (FTSCR)

For arm `a`:

```text
FTSCR(a) =
count(trusted_success = true AND postcondition_true = false)
-----------------------------------------------------------
total_trials
```

This measures the exact event that can poison execution history.

### False Positive Fraction among trusted successes

```text
FPF(a) =
false_trusted_successes
-----------------------
all_trusted_successes
```

This answers: “Of all positive execution outcomes written by this arm, how many were actually false?”

## Secondary endpoints

### Downstream contamination rate

A downstream action executes even though a declared prerequisite is false.

```text
DCR =
count(downstream_executed AND NOT downstream_safe)
---------------------------------------------------
total_trials
```

### Failure detection recall

Among trials where the semantic postcondition is false:

```text
recall =
count(rejected_or_blocked)
--------------------------
count(postcondition_false)
```

### False block rate

Among healthy trials:

```text
FBR =
count(not_trusted_success AND postcondition_true)
-------------------------------------------------
count(postcondition_true)
```

### True completion rate

```text
TCR =
count(trusted_success AND postcondition_true)
---------------------------------------------
total_trials
```

### Efficiency

- action latency;
- verifier latency;
- P50 / P95 end-to-end latency;
- attempts/retries;
- configurable cost units;
- cost per true completion.

## Statistical analysis

### Pairing

Fault assignment is generated once and applied to both arms. This controls for scenario difficulty.

### Confidence intervals

Every rate is reported with a Wilson 95% interval.

### Hypothesis test

Primary paired comparison uses McNemar's test on discordant false-success outcomes:

```text
b = baseline false-success, verified safe
c = baseline safe, verified false-success
```

The implementation reports the continuity-corrected chi-square statistic and p-value.

### Latency

Latency differences are descriptive because local scheduling noise is environment-dependent. Report median, P95 and the paired per-trial delta. Do not turn microbenchmark noise into a product claim.

## Default sample size

`1,000 paired trials per scenario`.

Why:

- if an arm observes zero false successes in 1,000 trials, the rough “rule of three” upper 95% bound is about 0.3%;
- it gives stable estimates across low-probability semantic failures;
- paired deterministic fault assignment greatly reduces variance for the primary endpoint.

The browser demo runs 25 trials per each of 8 scenarios (**200 paired trials total**) for speed. It should not replace the full 1,000-per-scenario benchmark.

## Scenario families

| ID | Cookbook seam | Injected failure | Independent truth |
|---|---|---|---|
| `chief.crm_silent_write` | function action + outcome memory | 200/success without row | CRM readback |
| `chief.deploy_false_green` | deployment action | success while unhealthy | health probe |
| `chief.cascade_welcome` | multi-step plan | false prerequisite followed by side effect | prerequisite row + message ledger |
| `support.false_handoff` | human escalation | queued response without queue record | escalation queue readback |
| `onboarding.accepted_not_indexed` | ingestion | accepted but not indexed | processing status |
| `finance.empty_evidence` | answer generation | answer with zero chunks | evidence cardinality |
| `finance.wrong_period` | temporal answer | answer cites wrong fiscal period | metadata constraint |
| `intel.unverified_briefing` | scheduled proactive briefing | source accepted but not indexed | processing status |

## Fault injection

Each scenario has a configured semantic-failure probability. Faults are deterministic from:

```text
suite_digest + seed + scenario_id + trial_index
```

The raw assignment is saved in the results file.

## Anti-cheat properties

The benchmark must never accept an aggregate-only result.

A valid result contains every raw paired trial with:

- fault assignment;
- baseline action output;
- baseline independent truth;
- verified action output;
- verifier evidence;
- final state;
- downstream execution state;
- timestamps and latency;
- receipt digest if verified.

`iolaus-bench certify` recomputes the summary from raw trials and fails on disagreement.

## Fairness

The baseline is intentionally strong in every way except postcondition verification:

- same route;
- same parameters;
- same retry budget unless the contract explicitly assigns verifier polling;
- same target;
- same fault;
- same local environment.

We are not benchmarking a bad planner against a good planner.

## Claims discipline

Do not say:

> HydraDB fails X% of actions.

The failure frequencies are injected benchmark conditions.

Say:

> Under controlled semantic-failure injection at cookbook integration seams, the benchmark measures how often each architecture commits a false success and lets that success propagate downstream.
