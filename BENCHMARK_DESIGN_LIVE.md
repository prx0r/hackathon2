# HydraFragileBench Live — benchmark design

## Research question

When an action-capable agent uses HydraDB for function routing and persistent execution memory, how often does a successful tool response correspond to the intended world-state transition, and how much does an independent Iolaus postcondition gate reduce false trusted successes and contaminated downstream/learning signals?

## Why the benchmark is decomposed

The system has three logically separate correctness problems:

1. **Routing** — did HydraDB select the intended function?
2. **Parameter extraction** — did the LLM produce valid intended parameters?
3. **State transition** — after execution returned success, did the world actually satisfy the postcondition?

A single end-to-end success number cannot tell which layer failed.

Therefore LiveBench records all three, but the baseline-vs-Iolaus causal treatment is isolated at layer 3.

## Paired design

For each `(scenario, trial_index)`:

```text
natural-language task
        |
        +---- HydraDB recall ------> route record
        |
        +---- Hermes extraction ---> parameter record
        |
        v
fixed intended action for causal comparison
        |
seeded semantic fault assignment
        |
   +----+------------------+
   |                       |
baseline                Iolaus
same action              same action
same fault               same fault
same target semantics    same target semantics
   |                       |
trust success=true       verify postcondition
   |                       |
feedback                 feedback only after gate
```

Hydra/Hermes are called **once per paired trial**, not separately per arm. Otherwise different model/retrieval randomness could destroy the pairing.

## Primary endpoint

False Trusted-Success Commit Rate:

```text
FTSCR = count(trusted_success && !postcondition_true) / trials
```

This is evaluated separately for baseline and Iolaus.

## Secondary verification endpoints

- false-positive fraction among trusted successes;
- downstream contamination;
- true completion;
- failure-detection recall;
- false-block rate;
- verifier overhead;
- cost unit per true completion;
- McNemar paired significance test;
- Wilson 95% intervals.

## Decision-plane endpoints

Hydra routing accuracy:

```text
selected function == preregistered expected function
```

Hermes schema validity:

```text
all required fields present && declared primitive types match
```

Hermes exact match:

```text
all preregistered expected parameter key/value pairs are present exactly
```

Extra harmless keys do not fail exact-match subset comparison, but schema-invalid output always fails schema validity.

## Learning-loop integrity endpoints

For every arm:

```text
false-positive learning signal =
  trusted_success && !postcondition_true
```

The exact execution record is always written with `infer=false` when feedback is enabled.

Positive/negative learning signal goes to a distinct arm collection with `infer=true`.

The benchmark explicitly expects baseline to sometimes pollute its positive feedback if a tool lies optimistically. Iolaus must emit zero positive learning signals for false world-state transitions in the deterministic verifier suite.

## Scenario families

1. CRM silent write — request succeeded but requested customer absent.
2. Deployment false green — trigger succeeded but health readback unhealthy.
3. Multi-step cascade — welcome step must depend on verified customer creation.
4. Human handoff — response says queued but queue item absent.
5. Ingestion — accepted is not indexed/recall-ready.
6. Financial answer — output exists with zero evidence chunks.
7. Temporal evidence — answer uses wrong fiscal period.
8. Competitive briefing — publication must wait for source readiness.

These are deliberately injected integration-seam failures, not observed HydraDB failure rates.

## Fault schedule

The fault schedule is deterministic:

```text
SHA256(seed || scenario_id || trial_index)
  -> ChaCha20Rng
  -> sample < preregistered probability
```

The natural-language task, HydraDB query, and Hermes prompt never contain the fault assignment.

## Certified controlled run

Default:

```text
8 scenarios × 1,000 paired trials = 8,000 pairs
seed = 20260819
```

The 8,000-pair run isolates Iolaus itself and should be the primary statistical claim.

## Live integration run

Recommended:

- smoke: 2/scenario = 16 trials;
- video: 5/scenario = 40 trials;
- presentation-quality: 25/scenario = 200 trials;
- stronger live measurement: 100/scenario = 800 trials if local Hydra/Hermes throughput is acceptable.

Do not make the live LLM run huge just to create a larger N. Its purpose is integration evidence and decision-plane measurement; the deterministic controlled run supplies statistical power for the verification treatment.
