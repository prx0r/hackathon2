use serde::{Deserialize, Serialize};
use statrs::distribution::{ChiSquared, ContinuousCDF};

use crate::{ArmOutcome, PairedTrial};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateEstimate {
    pub numerator: u64,
    pub denominator: u64,
    pub rate: f64,
    pub lower_95: f64,
    pub upper_95: f64,
}

pub fn wilson(numerator: u64, denominator: u64) -> RateEstimate {
    if denominator == 0 {
        return RateEstimate {
            numerator,
            denominator,
            rate: 0.0,
            lower_95: 0.0,
            upper_95: 0.0,
        };
    }
    let z = 1.959963984540054_f64;
    let n = denominator as f64;
    let phat = numerator as f64 / n;
    let denom = 1.0 + z * z / n;
    let center = (phat + z * z / (2.0 * n)) / denom;
    let half = z * ((phat * (1.0 - phat) / n + z * z / (4.0 * n * n)).sqrt()) / denom;
    RateEstimate {
        numerator,
        denominator,
        rate: phat,
        lower_95: (center - half).max(0.0),
        upper_95: (center + half).min(1.0),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencySummary {
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub mean_ms: f64,
}

fn percentile(mut values: Vec<u64>, q: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    let idx = ((values.len() - 1) as f64 * q).round() as usize;
    values[idx]
}

fn latency(values: Vec<u64>) -> LatencySummary {
    let mean = if values.is_empty() {
        0.0
    } else {
        values.iter().map(|x| *x as f64).sum::<f64>() / values.len() as f64
    };
    LatencySummary {
        p50_ms: percentile(values.clone(), 0.50),
        p95_ms: percentile(values, 0.95),
        mean_ms: mean,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmMetrics {
    pub trials: u64,
    pub false_trusted_success: RateEstimate,
    pub false_positive_fraction: RateEstimate,
    pub downstream_contamination: RateEstimate,
    pub true_completion: RateEstimate,
    pub failure_detection_recall: RateEstimate,
    pub false_block_rate: RateEstimate,
    pub latency: LatencySummary,
    pub total_cost_units: f64,
    pub cost_per_true_completion: Option<f64>,
}

fn arm_metrics<'a>(outcomes: impl Iterator<Item = &'a ArmOutcome>) -> ArmMetrics {
    let xs: Vec<&ArmOutcome> = outcomes.collect();
    let trials = xs.len() as u64;
    let false_success = xs.iter().filter(|x| x.false_trusted_success()).count() as u64;
    let trusted_successes = xs.iter().filter(|x| x.trusted_success).count() as u64;
    let contaminated = xs.iter().filter(|x| x.contaminated_downstream()).count() as u64;
    let true_completed = xs.iter().filter(|x| x.true_completion()).count() as u64;

    let false_world = xs.iter().filter(|x| !x.postcondition_true).count() as u64;
    let detected_false = xs.iter()
        .filter(|x| !x.postcondition_true && !x.trusted_success)
        .count() as u64;

    let true_world = xs.iter().filter(|x| x.postcondition_true).count() as u64;
    let false_block = xs.iter()
        .filter(|x| x.postcondition_true && !x.trusted_success)
        .count() as u64;

    let total_cost = xs.iter().map(|x| x.cost_units).sum::<f64>();

    ArmMetrics {
        trials,
        false_trusted_success: wilson(false_success, trials),
        false_positive_fraction: wilson(false_success, trusted_successes),
        downstream_contamination: wilson(contaminated, trials),
        true_completion: wilson(true_completed, trials),
        failure_detection_recall: wilson(detected_false, false_world),
        false_block_rate: wilson(false_block, true_world),
        latency: latency(xs.iter().map(|x| x.total_latency_ms).collect()),
        total_cost_units: total_cost,
        cost_per_true_completion: if true_completed == 0 {
            None
        } else {
            Some(total_cost / true_completed as f64)
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McNemar {
    pub baseline_false_verified_safe: u64,
    pub baseline_safe_verified_false: u64,
    pub chi_square_cc: f64,
    pub p_value: f64,
}

fn mcnemar(trials: &[PairedTrial]) -> McNemar {
    let mut b = 0_u64;
    let mut c = 0_u64;

    for t in trials {
        let x = t.baseline.false_trusted_success();
        let y = t.verified.false_trusted_success();
        if x && !y { b += 1; }
        if !x && y { c += 1; }
    }

    let n = b + c;
    let statistic = if n == 0 {
        0.0
    } else {
        let diff = (b as f64 - c as f64).abs();
        ((diff - 1.0).max(0.0)).powi(2) / n as f64
    };

    let chi = ChiSquared::new(1.0).expect("df=1");
    let p = if n == 0 { 1.0 } else { 1.0 - chi.cdf(statistic) };

    McNemar {
        baseline_false_verified_safe: b,
        baseline_safe_verified_false: c,
        chi_square_cc: statistic,
        p_value: p,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonMetrics {
    pub baseline: ArmMetrics,
    pub verified: ArmMetrics,
    pub false_success_absolute_delta: f64,
    pub downstream_contamination_absolute_delta: f64,
    pub latency_p95_overhead_ms: i64,
    pub mcnemar: McNemar,
}

pub fn summarize(trials: &[PairedTrial]) -> ComparisonMetrics {
    let baseline = arm_metrics(trials.iter().map(|t| &t.baseline));
    let verified = arm_metrics(trials.iter().map(|t| &t.verified));

    ComparisonMetrics {
        false_success_absolute_delta:
            verified.false_trusted_success.rate - baseline.false_trusted_success.rate,
        downstream_contamination_absolute_delta:
            verified.downstream_contamination.rate - baseline.downstream_contamination.rate,
        latency_p95_overhead_ms:
            verified.latency.p95_ms as i64 - baseline.latency.p95_ms as i64,
        mcnemar: mcnemar(trials),
        baseline,
        verified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wilson_bounds_are_valid() {
        let x = wilson(0, 1000);
        assert_eq!(x.rate, 0.0);
        assert!(x.upper_95 > 0.0);
        assert!(x.upper_95 < 0.01);
    }
}
