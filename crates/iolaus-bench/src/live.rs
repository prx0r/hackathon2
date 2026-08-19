use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::time::Instant;

use chrono::Utc;
use iolaus_core::{ComparisonMetrics, PairedTrial, RateEstimate};
use iolaus_hydra::{HydraDbClient, RecallRequest};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{fault_assignment, HermesExtractor, Runner, ScenarioSpec, SuiteFile};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveTaskSuite {
    pub schema: String,
    pub tasks: Vec<LiveTaskSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveTaskSpec {
    pub id: String,
    pub kind: String,
    pub expected_function: String,
    pub tasks: Vec<String>,
    pub expected_params: Value,
}

impl LiveTaskSuite {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let bytes=std::fs::read(path)?;
        let out:Self=serde_json::from_slice(&bytes)?;
        anyhow::ensure!(!out.tasks.is_empty(),"live task suite is empty");
        Ok(out)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEvidence {
    pub request_sha256: String,
    pub response_sha256: String,
    pub raw_response: Value,
    pub selected_function: Option<String>,
    pub expected_function: String,
    pub correct: bool,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEvidence {
    pub baseline_logged: bool,
    pub verified_logged: bool,
    pub baseline_false_positive_learning_signal: bool,
    pub verified_false_positive_learning_signal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveTrial {
    pub scenario_id: String,
    pub trial_index: u64,
    pub task: String,
    pub route: RouteEvidence,
    pub extraction: crate::hermes::ParameterExtraction,
    pub parameter_exact_match: bool,
    pub paired_execution: PairedTrial,
    pub feedback: FeedbackEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveSummary {
    pub trials: u64,
    pub routing_accuracy: RateEstimate,
    pub parameter_schema_validity: RateEstimate,
    pub parameter_exact_match: RateEstimate,
    pub baseline_false_positive_learning_signals: RateEstimate,
    pub verified_false_positive_learning_signals: RateEstimate,
    pub verification: ComparisonMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveRun {
    pub schema: String,
    pub generated_at: String,
    pub seed: u64,
    pub trials_per_scenario: u64,
    pub hydra_feedback_enabled: bool,
    pub live_task_digest: String,
    pub controlled_suite_digest: String,
    pub function_registry_digest: String,
    pub benchmark_design: String,
    pub trials: Vec<LiveTrial>,
    pub summary: LiveSummary,
}

pub struct LiveRunner {
    runner: Runner,
    hydra: HydraDbClient,
    hermes: HermesExtractor,
    functions: BTreeMap<String, Value>,
}

impl LiveRunner {
    pub fn new(runner: Runner, hydra: HydraDbClient, hermes: HermesExtractor, functions: Vec<Value>) -> Self {
        let mut map=BTreeMap::new();
        for f in functions {
            if let Some(id)=f.get("id").and_then(Value::as_str) { map.insert(id.to_string(),f); }
        }
        Self{runner,hydra,hermes,functions:map}
    }

    pub async fn run(
        &self,
        controlled: &SuiteFile,
        live_tasks: &LiveTaskSuite,
        trials_per_scenario: u64,
        seed: u64,
        write_feedback: bool,
    ) -> anyhow::Result<LiveRun> {
        let specs:HashMap<&str,&ScenarioSpec>=controlled.scenario.iter().map(|s|(s.id.as_str(),s)).collect();
        let known:Vec<String>=self.functions.keys().cloned().collect();
        let mut trials=Vec::new();

        for task_spec in &live_tasks.tasks {
            anyhow::ensure!(!task_spec.tasks.is_empty(), "live task {} has no task variants", task_spec.id);
            let spec=*specs.get(task_spec.id.as_str()).ok_or_else(||anyhow::anyhow!("live task {} missing controlled scenario",task_spec.id))?;
            let fn_schema=self.functions.get(&task_spec.expected_function).ok_or_else(||anyhow::anyhow!("function {} missing from registry",task_spec.expected_function))?;
            let params_schema=fn_schema.get("parameters").cloned().unwrap_or(Value::Null);

            for trial_index in 0..trials_per_scenario {
                let task=task_spec.tasks[(trial_index as usize)%task_spec.tasks.len()].clone();
                anyhow::ensure!(!task.to_ascii_lowercase().contains("fault"),"task leaks benchmark fault language");

                let route_start=Instant::now();
                let request=RecallRequest{
                    query:task.clone(),
                    sub_tenant_id:Some("iolaus-functions".into()),
                    max_results:5,
                    mode:"thinking".into(),
                    graph_context:true,
                };
                let request_bytes=serde_json::to_vec(&request)?;
                let response=self.hydra.recall(request).await?;
                let selected=find_first_function_id(&response,&known);
                let route=RouteEvidence{
                    request_sha256:sha256(request_bytes),
                    response_sha256:sha256(serde_json::to_vec(&response)?),
                    raw_response:response.clone(),
                    correct:selected.as_deref()==Some(task_spec.expected_function.as_str()),
                    selected_function:selected,
                    expected_function:task_spec.expected_function.clone(),
                    latency_ms:route_start.elapsed().as_millis() as u64,
                };

                // Parameter extraction is deliberately given the EXPECTED function schema, not the
                // route output. Routing and extraction are evaluated independently; neither is
                // permitted to change the paired verification treatment.
                let extraction=self.hermes.extract(&task,&task_spec.expected_function,&params_schema).await?;
                let parameter_exact_match=canonical_subset_equal(&extraction.parsed,&task_spec.expected_params);

                let fault=fault_assignment(seed,&spec.id,trial_index,spec.fault_probability);
                let paired=self.runner.run_paired(spec,trial_index,seed,fault).await?;

                let mut feedback=FeedbackEvidence{
                    baseline_logged:false,verified_logged:false,
                    baseline_false_positive_learning_signal:false,
                    verified_false_positive_learning_signal:false,
                };
                if write_feedback {
                    self.hydra.log_arm_outcome(&paired.baseline).await?;
                    feedback.baseline_logged=true;
                    self.hydra.log_arm_outcome(&paired.verified).await?;
                    feedback.verified_logged=true;
                }
                feedback.baseline_false_positive_learning_signal=write_feedback && paired.baseline.false_trusted_success();
                feedback.verified_false_positive_learning_signal=write_feedback && paired.verified.false_trusted_success();

                trials.push(LiveTrial{
                    scenario_id:task_spec.id.clone(),trial_index,task,route,extraction,
                    parameter_exact_match,paired_execution:paired,feedback,
                });
            }
        }

        let summary=summarize_live(&trials);
        Ok(LiveRun{
            schema:"iolaus.hydradb.livebench.v1".into(),
            generated_at:Utc::now().to_rfc3339(),seed,trials_per_scenario,hydra_feedback_enabled:write_feedback,
            live_task_digest:sha256(serde_json::to_vec(live_tasks)?),
            controlled_suite_digest:sha256(serde_json::to_vec(controlled)?),
            function_registry_digest:sha256(serde_json::to_vec(&self.functions)?),
            benchmark_design:"HydraDB routing and Hermes parameter extraction are measured on each trial. The Iolaus causal comparison then executes the same intended action and identical seeded fault in baseline and verified arms, preventing decision-plane errors from changing the treatment.".into(),
            trials,summary,
        })
    }
}

pub fn summarize_live(trials:&[LiveTrial])->LiveSummary{
    let n=trials.len() as u64;
    let route=trials.iter().filter(|t|t.route.correct).count() as u64;
    let schema=trials.iter().filter(|t|t.extraction.schema_valid).count() as u64;
    let exact=trials.iter().filter(|t|t.parameter_exact_match).count() as u64;
    let base_fp=trials.iter().filter(|t|t.feedback.baseline_false_positive_learning_signal).count() as u64;
    let ver_fp=trials.iter().filter(|t|t.feedback.verified_false_positive_learning_signal).count() as u64;
    let paired:Vec<PairedTrial>=trials.iter().map(|t|t.paired_execution.clone()).collect();
    LiveSummary{
        trials:n,
        routing_accuracy:iolaus_core::wilson(route,n),
        parameter_schema_validity:iolaus_core::wilson(schema,n),
        parameter_exact_match:iolaus_core::wilson(exact,n),
        baseline_false_positive_learning_signals:iolaus_core::wilson(base_fp,n),
        verified_false_positive_learning_signals:iolaus_core::wilson(ver_fp,n),
        verification:iolaus_core::summarize(&paired),
    }
}

pub fn certify_live(run:&LiveRun)->anyhow::Result<()> {
    anyhow::ensure!(run.schema=="iolaus.hydradb.livebench.v1","unexpected schema");
    anyhow::ensure!(!run.trials.is_empty(),"empty live benchmark");
    let recomputed=summarize_live(&run.trials);
    anyhow::ensure!(serde_json::to_value(&run.summary)?==serde_json::to_value(&recomputed)?,"live summary does not reproduce from raw trials");
    for t in &run.trials{
        if run.hydra_feedback_enabled {
            anyhow::ensure!(t.feedback.baseline_logged && t.feedback.verified_logged,"feedback-enabled trial missing HydraDB arm log");
        }
        anyhow::ensure!(!t.task.to_ascii_lowercase().contains("fault"),"task contains forbidden fault hint");
        anyhow::ensure!(t.paired_execution.baseline.fault_injected==t.paired_execution.verified.fault_injected,"arms did not receive same fault");
        if let Some(r)=&t.paired_execution.verified.receipt{
            iolaus_core::ReceiptSigner::verify(r)?;
            anyhow::ensure!(t.paired_execution.verified.trusted_success,"receipt on untrusted transition");
            anyhow::ensure!(t.paired_execution.verified.postcondition_true,"receipt on false postcondition");
        }
        anyhow::ensure!(!t.feedback.verified_false_positive_learning_signal,"verified arm emitted false-positive learning signal");
    }
    Ok(())
}

fn canonical_subset_equal(actual:&Value,expected:&Value)->bool{
    match (actual.as_object(),expected.as_object()){
        (Some(a),Some(e))=>e.iter().all(|(k,v)|a.get(k)==Some(v)),
        _=>actual==expected,
    }
}

pub fn find_first_function_id(value:&Value, known:&[String])->Option<String>{
    match value{
        Value::String(s)=>{
            if known.iter().any(|k|k==s){return Some(s.clone());}
            for k in known{if s.contains(k){return Some(k.clone());}}
            None
        }
        Value::Array(xs)=>xs.iter().find_map(|x|find_first_function_id(x,known)),
        Value::Object(m)=>{
            for key in ["id","function_id","source_id","title"]{
                if let Some(v)=m.get(key){if let Some(x)=find_first_function_id(v,known){return Some(x);}}
            }
            m.values().find_map(|x|find_first_function_id(x,known))
        }
        _=>None,
    }
}

fn sha256(bytes:impl AsRef<[u8]>)->String{let mut h=Sha256::new();h.update(bytes.as_ref());hex::encode(h.finalize())}

#[cfg(test)]
mod tests{
    use super::*;
    use serde_json::json;
    #[test] fn finds_nested_function(){let x=json!({"chunks":[{"content":"use trigger_deployment"}]});assert_eq!(find_first_function_id(&x,&["trigger_deployment".into()]),Some("trigger_deployment".into()));}
    #[test] fn subset_params(){assert!(canonical_subset_equal(&json!({"id":"a","email":"e","extra":1}),&json!({"id":"a","email":"e"})));}
}
