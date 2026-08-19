use anyhow::Context;
use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use iolaus_core::ArmOutcome;

#[derive(Clone)]
pub struct HydraDbClient {
    http: Client,
    pub base_url: String,
    pub api_key: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallRequest {
    pub query: String,
    pub sub_tenant_id: Option<String>,
    pub max_results: u32,
    pub mode: String,
    pub graph_context: bool,
}

impl HydraDbClient {
    pub fn from_env() -> anyhow::Result<Self> {
        // Keep compatibility with the hackathon environment while accepting the
        // newer HYDRADB_* spelling used by the public CLI/docs.
        let api_key = std::env::var("HYDRA_DB_API_KEY")
            .or_else(|_| std::env::var("HYDRADB_API_KEY"))
            .context("HYDRA_DB_API_KEY (or HYDRADB_API_KEY) is required")?;
        let tenant_id = std::env::var("IOLAUS_HYDRA_TENANT")
            .or_else(|_| std::env::var("IOLAUS_HYDRA_DATABASE"))
            .context("IOLAUS_HYDRA_TENANT (or IOLAUS_HYDRA_DATABASE) is required")?;
        let base_url = std::env::var("HYDRA_DB_API_URL")
            .or_else(|_| std::env::var("HYDRADB_BASE_URL"))
            .unwrap_or_else(|_| "https://api.hydradb.com".to_string());

        Ok(Self {
            http: Client::builder().timeout(std::time::Duration::from_secs(60)).build()?,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            tenant_id,
        })
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.bearer_auth(&self.api_key)
    }

    pub async fn create_tenant_if_supported(&self) -> anyhow::Result<Value> {
        let response = self.auth(self.http.post(format!("{}/tenants/create", self.base_url)))
            .json(&json!({"tenant_id": self.tenant_id}))
            .send().await?;
        if response.status().as_u16() == 409 {
            return Ok(json!({"ok":true,"already_exists":true}));
        }
        Ok(response.error_for_status()?.json().await.unwrap_or_else(|_|json!({"ok":true})))
    }

    pub async fn add_memory(
        &self,
        sub_tenant_id: &str,
        text: &str,
        user_name: &str,
        infer: bool,
    ) -> anyhow::Result<Value> {
        let body = json!({
            "memories": [{
                "text": text,
                "user_name": user_name,
                "infer": infer
            }],
            "tenant_id": self.tenant_id,
            "sub_tenant_id": sub_tenant_id,
            "upsert": true
        });
        let response = self.auth(
            self.http.post(format!("{}/memories/add_memory", self.base_url))
        )
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

        Ok(response.json().await.unwrap_or_else(|_| json!({"ok": true})))
    }

    pub async fn recall(&self, req: RecallRequest) -> anyhow::Result<Value> {
        let mut body = json!({
            "tenant_id": self.tenant_id,
            "query": req.query,
            "max_results": req.max_results,
            "mode": req.mode,
            "graph_context": req.graph_context
        });
        if let Some(sub) = req.sub_tenant_id {
            body["sub_tenant_id"] = json!(sub);
        }

        let response = self.auth(
            self.http.post(format!("{}/recall/full_recall", self.base_url))
        )
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

        Ok(response.json().await?)
    }

    pub async fn upload_app_knowledge(
        &self,
        sub_tenant_id: &str,
        app_knowledge: &Value,
    ) -> anyhow::Result<Value> {
        let form = multipart::Form::new()
            .text("tenant_id", self.tenant_id.clone())
            .text("sub_tenant_id", sub_tenant_id.to_string())
            .text("app_knowledge", serde_json::to_string(app_knowledge)?);

        let response = self.auth(
            self.http.post(format!("{}/ingestion/upload_knowledge", self.base_url))
        )
        .multipart(form)
        .send()
        .await?
        .error_for_status()?;

        Ok(response.json().await.unwrap_or_else(|_| json!({"ok": true})))
    }

    pub async fn verify_processing(&self, file_id: &str) -> anyhow::Result<Value> {
        let response = self.auth(
            self.http.post(format!("{}/ingestion/verify_processing", self.base_url))
        )
        .query(&[
            ("tenant_id", self.tenant_id.as_str()),
            ("file_ids", file_id),
        ])
        .json(&json!({}))
        .send()
        .await?
        .error_for_status()?;

        Ok(response.json().await?)
    }

    pub async fn log_arm_outcome(&self, outcome: &ArmOutcome) -> anyhow::Result<()> {
        let prefix = if outcome.arm == "baseline" { "baseline" } else { "verified" };
        let exact_sub = format!("iolaus-{}-execution-log", prefix);
        let learn_sub = format!("iolaus-{}-learning", prefix);

        let exact = serde_json::to_string(outcome)?;
        self.add_memory(&exact_sub, &exact, "iolaus-benchmark", false).await?;

        // This deliberately mirrors a naïve application feedback loop in the baseline:
        // trusted_success is promoted as a positive signal even if the hidden world-state
        // later proves false. The verified arm can only trusted-success after the gate.
        if outcome.trusted_success {
            let signal = format!(
                "scenario={} trusted_success=true postcondition_true={} total_latency_ms={} cost_units={:.3}",
                outcome.scenario_id,
                outcome.postcondition_true,
                outcome.total_latency_ms,
                outcome.cost_units
            );
            self.add_memory(&learn_sub, &signal, "iolaus-benchmark", true).await?;
        } else if !outcome.postcondition_true {
            let signal = format!(
                "scenario={} verified_or_detected_failure=true postcondition_true=false total_latency_ms={}",
                outcome.scenario_id,
                outcome.total_latency_ms
            );
            self.add_memory(&learn_sub, &signal, "iolaus-benchmark", true).await?;
        }

        Ok(())
    }
}
