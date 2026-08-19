use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteFile {
    pub name: String,
    pub trials_per_scenario: u64,
    pub seed: u64,
    pub scenario: Vec<ScenarioSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioSpec {
    pub id: String,
    pub kind: String,
    pub cookbook: String,
    pub fault_probability: f64,
    pub verifier_attempts: u32,
    pub verifier_backoff_ms: u64,
}

impl SuiteFile {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let suite: Self = toml::from_str(&text)?;
        anyhow::ensure!(!suite.scenario.is_empty(), "suite must contain scenarios");
        for s in &suite.scenario {
            anyhow::ensure!(
                (0.0..=1.0).contains(&s.fault_probability),
                "invalid fault probability for {}",
                s.id
            );
        }
        Ok(suite)
    }
}
