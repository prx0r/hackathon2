use std::path::PathBuf;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesConfig {
    pub executable: PathBuf,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_timeout_ms() -> u64 { 120_000 }

impl Default for HermesConfig {
    fn default() -> Self {
        Self {
            executable: std::env::var("HERMES_BIN").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("hermes")),
            model: std::env::var("HERMES_INFERENCE_MODEL").ok(),
            provider: std::env::var("HERMES_PROVIDER").ok(),
            timeout_ms: std::env::var("IOLAUS_HERMES_TIMEOUT_MS").ok().and_then(|x| x.parse().ok()).unwrap_or(default_timeout_ms()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterExtraction {
    pub prompt_sha256: String,
    pub schema_sha256: String,
    pub raw_stdout_sha256: String,
    pub raw_stdout: String,
    pub parsed: Value,
    pub schema_valid: bool,
    pub validation_errors: Vec<String>,
    pub latency_ms: u64,
    pub model: Option<String>,
    pub provider: Option<String>,
}

#[derive(Clone)]
pub struct HermesExtractor {
    cfg: HermesConfig,
}

impl HermesExtractor {
    pub fn new(cfg: HermesConfig) -> Self { Self { cfg } }

    pub async fn smoke(&self) -> anyhow::Result<Value> {
        let prompt = r#"Return exactly one JSON object and nothing else: {"ok":true,"component":"iolaus-hermes"}"#;
        let (stdout, _) = self.run(prompt).await?;
        extract_json_object(&stdout)
    }

    pub async fn extract(&self, task: &str, function_id: &str, schema: &Value) -> anyhow::Result<ParameterExtraction> {
        let prompt = format!(
            "You are the parameter-extraction component of a benchmark.\n\
             Convert the user task into JSON parameters for exactly the supplied function.\n\
             Do not choose another function. Do not discuss the benchmark. Do not infer or mention any injected fault.\n\
             Return ONE JSON object only; no markdown, comments, or prose.\n\n\
             FUNCTION_ID: {function_id}\n\
             JSON_SCHEMA: {}\n\
             USER_TASK: {}",
            serde_json::to_string(schema)?,
            serde_json::to_string(task)?,
        );
        let schema_sha256 = sha256(serde_json::to_vec(schema)?);
        let prompt_sha256 = sha256(prompt.as_bytes());
        let start = Instant::now();
        let (stdout, _) = self.run(&prompt).await?;
        let parsed = extract_json_object(&stdout)?;
        let errors = validate_simple_json_schema(&parsed, schema);
        Ok(ParameterExtraction {
            prompt_sha256,
            schema_sha256,
            raw_stdout_sha256: sha256(stdout.as_bytes()),
            raw_stdout: stdout,
            parsed,
            schema_valid: errors.is_empty(),
            validation_errors: errors,
            latency_ms: start.elapsed().as_millis() as u64,
            model: self.cfg.model.clone(),
            provider: self.cfg.provider.clone(),
        })
    }

    async fn run(&self, prompt: &str) -> anyhow::Result<(String, String)> {
        let mut cmd = Command::new(&self.cfg.executable);
        cmd.arg("-z").arg(prompt).arg("--source").arg("tool").arg("--ignore-rules");
        if let Some(provider) = &self.cfg.provider { cmd.arg("--provider").arg(provider); }
        if let Some(model) = &self.cfg.model { cmd.arg("--model").arg(model); }
        cmd.kill_on_drop(true);

        let output = tokio::time::timeout(
            std::time::Duration::from_millis(self.cfg.timeout_ms),
            cmd.output(),
        ).await.map_err(|_| anyhow::anyhow!("Hermes timed out after {}ms", self.cfg.timeout_ms))??;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        anyhow::ensure!(output.status.success(), "Hermes exited {}: {}", output.status, stderr.trim());
        anyhow::ensure!(!stdout.trim().is_empty(), "Hermes returned empty stdout");
        Ok((stdout, stderr))
    }
}

pub fn extract_json_object(text: &str) -> anyhow::Result<Value> {
    let trimmed = text.trim();
    if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
        anyhow::ensure!(v.is_object(), "Hermes output must be a JSON object");
        return Ok(v);
    }
    let start = trimmed.find('{').ok_or_else(|| anyhow::anyhow!("no JSON object in Hermes output"))?;
    let end = trimmed.rfind('}').ok_or_else(|| anyhow::anyhow!("unterminated JSON object in Hermes output"))?;
    let v: Value = serde_json::from_str(&trimmed[start..=end])?;
    anyhow::ensure!(v.is_object(), "Hermes output must be a JSON object");
    Ok(v)
}

pub fn validate_simple_json_schema(value: &Value, schema: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(obj) = value.as_object() else { return vec!["value is not an object".into()]; };
    let required = schema.get("required").and_then(Value::as_array).cloned().unwrap_or_default();
    for r in required {
        if let Some(name) = r.as_str() {
            if !obj.contains_key(name) { errors.push(format!("missing required field: {name}")); }
        }
    }
    if let Some(props) = schema.get("properties").and_then(Value::as_object) {
        for (name, spec) in props {
            if let Some(v) = obj.get(name) {
                let expected = spec.get("type").and_then(Value::as_str).unwrap_or("");
                let ok = match expected {
                    "string" => v.is_string(),
                    "number" => v.is_number(),
                    "integer" => v.as_i64().is_some() || v.as_u64().is_some(),
                    "boolean" => v.is_boolean(),
                    "array" => v.is_array(),
                    "object" => v.is_object(),
                    _ => true,
                };
                if !ok { errors.push(format!("field {name} has wrong type; expected {expected}")); }
            }
        }
    }
    errors
}

fn sha256(bytes: impl AsRef<[u8]>) -> String {
    let mut h = Sha256::new(); h.update(bytes.as_ref()); hex::encode(h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_wrapped_json() {
        let v = extract_json_object("noise\n{\"id\":\"alice\"}\n").unwrap();
        assert_eq!(v["id"], serde_json::json!("alice"));
    }

    #[test]
    fn validates_required_types() {
        let schema=serde_json::json!({"type":"object","properties":{"id":{"type":"string"}},"required":["id"]});
        assert!(validate_simple_json_schema(&serde_json::json!({"id":"alice"}),&schema).is_empty());
        assert_eq!(validate_simple_json_schema(&serde_json::json!({"id":2}),&schema).len(),1);
    }
}
