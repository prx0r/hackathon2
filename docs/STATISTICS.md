# Statistical design notes

## Why paired trials

If one arm sees more hard faults than the other, a difference in failure rate is uninterpretable.

Iolaus generates one fault bit per `(scenario, trial_index)` and reuses it in both namespaces.

The benchmark therefore estimates the effect of the verification policy conditional on identical external conditions.

## Wilson interval

For a binomial rate `p = x/n`, we report the Wilson 95% interval instead of the normal/Wald interval, because several desired rates can be near zero.

## McNemar comparison

For the primary endpoint, each paired trial yields a boolean:

```text
false_trusted_success_baseline
false_trusted_success_verified
```

Only discordant pairs carry evidence about the policy difference.

The runner reports continuity-corrected McNemar chi-square and p-value.

Scenario-level p-values are exploratory; the pooled primary endpoint is the main comparison.

## Zero-event interpretation

Do not say “zero observed means impossible.”

If zero events are observed in 1,000 independent-like trials, the classic rough 95% upper bound is `3/1000 = 0.3%`.

The actual report also includes Wilson intervals.

## Latency

Do not compare the local benchmark's latency directly to HydraDB's published hosted latency. They measure different systems.

Use local latency only to quantify the incremental verification overhead of the two-arm benchmark.

## Cost units

The local benchmark uses configurable abstract cost units:

- tool call: 1.0
- verifier readback: 0.1
- retry: additional call cost
- downstream action: 1.0

In live deployments these can be replaced by:
- API prices;
- inference token cost;
- wall-clock compute;
- business cost.

Primary efficiency metric:

```text
total cost units / true completed tasks
```

This avoids making cheap false successes look efficient.
