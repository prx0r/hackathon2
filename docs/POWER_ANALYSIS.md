# Power and sample-size design

HydraFragileBench uses paired trials, so its primary McNemar comparison will usually be more efficient than an unpaired two-proportion test when the same faults affect both arms.

For planning, however, this document uses the **more conservative unpaired normal approximation** so the sample-size justification does not depend on assuming a favorable within-pair correlation.

Two-sided alpha: **0.05**  
Target power: **0.80**

Approximate per-arm sample sizes:

| Baseline FTSCR | Verified FTSCR | Absolute effect | Conservative n / arm |
|---:|---:|---:|---:|
| 15% | 2% | 13 pp | 72 |
| 10% | 1% | 9 pp | 100 |
| 5% | 0.5% | 4.5 pp | 207 |
| 5% | 1% | 4 pp | 285 |
| 2% | 0.2% | 1.8 pp | 526 |

The release default of **1,000 paired trials per scenario** is therefore intentionally larger than the conservative sample sizes for the effect sizes the benchmark is designed to expose.

It also gives a useful zero-event interpretation: if a verifier arm observes zero false trusted successes in 1,000 trials, the classic rough “rule of three” places the upper 95% event-rate bound near **0.3%**. The result file still reports Wilson intervals rather than presenting zero observations as proof of impossibility.

## Formula used for planning

For two independent proportions `p1` and `p2`:

```text
n ≈ [
  z_(1-α/2) * sqrt(2 p̄ (1-p̄))
  + z_(power) * sqrt(p1(1-p1) + p2(1-p2))
]^2 / (p1-p2)^2
```

where `p̄ = (p1+p2)/2`.

The final hypothesis test remains paired **McNemar**, because each baseline/verified observation shares the exact same scenario and injected fault.

## Multiple scenarios

The pooled primary endpoint is preregistered as the main comparison.

Scenario-level rates and p-values are secondary/exploratory. Do not cherry-pick whichever individual scenario happens to have the smallest p-value.

## Recommended run tiers

- **Video:** 25/scenario = 200 paired trials total.
- **Development:** 200/scenario = 1,600 paired trials total.
- **Submission benchmark:** 1,000/scenario = 8,000 paired trials total.
- **Stress:** 5,000/scenario only if runtime is acceptable.

Every tier uses the same deterministic seeded fault scheduler.
