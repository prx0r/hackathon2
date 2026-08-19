use std::time::{Duration, Instant};

use chrono::Utc;
use iolaus_core::{
    ActionContract, ArmOutcome, ExecutionObservation, PairedTrial, ReceiptSigner,
    TimelineEvent, VerificationEvidence, VerifierKind, VerifierSpec,
};
use iolaus_hydra::HydraDbClient;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::time::sleep;
use uuid::Uuid;

use crate::suite::{ScenarioSpec, SuiteFile};

#[derive(Clone)]
pub struct Runner {
    http: Client,
    base_url: String,
    signer: ReceiptSigner,
    hydra: Option<HydraDbClient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRun {
    pub schema: String,
    pub suite_name: String,
    pub suite_digest: String,
    pub seed: u64,
    pub trials_per_scenario: u64,
    pub generated_at: String,
    pub trials: Vec<PairedTrial>,
    pub summary: iolaus_core::ComparisonMetrics,
}

impl Runner {
    pub fn new(base_url: String, hydra: Option<HydraDbClient>) -> Self {
        Self {
            http: Client::new(),
            base_url,
            signer: ReceiptSigner::generate(),
            hydra,
        }
    }

    pub async fn run_suite(
        &self,
        suite: &SuiteFile,
        trials_per_scenario: u64,
        seed: u64,
    ) -> anyhow::Result<BenchmarkRun> {
        let mut trials = Vec::new();

        for spec in &suite.scenario {
            for trial_index in 0..trials_per_scenario {
                let fault = fault_assignment(seed, &spec.id, trial_index, spec.fault_probability);
                let paired = self.run_paired(spec, trial_index, seed, fault).await?;
                if let Some(hydra) = &self.hydra {
                    hydra.log_arm_outcome(&paired.baseline).await?;
                    hydra.log_arm_outcome(&paired.verified).await?;
                }
                trials.push(paired);
            }
        }

        let suite_digest = sha256_hex(serde_json::to_vec(suite)?);
        let summary = iolaus_core::summarize(&trials);

        Ok(BenchmarkRun {
            schema: "iolaus.hydrafragilebench.v1".into(),
            suite_name: suite.name.clone(),
            suite_digest,
            seed,
            trials_per_scenario,
            generated_at: Utc::now().to_rfc3339(),
            trials,
            summary,
        })
    }

    pub async fn run_paired(
        &self,
        spec: &ScenarioSpec,
        trial_index: u64,
        seed: u64,
        fault: bool,
    ) -> anyhow::Result<PairedTrial> {
        let base_ns = format!("b-{}-{}", sanitize(&spec.id), trial_index);
        let verified_ns = format!("v-{}-{}", sanitize(&spec.id), trial_index);

        let baseline = self.run_arm(spec, &base_ns, false, fault).await?;
        let verified = self.run_arm(spec, &verified_ns, true, fault).await?;

        Ok(PairedTrial {
            scenario_id: spec.id.clone(),
            trial_index,
            seed,
            fault_injected: fault,
            baseline,
            verified,
        })
    }

    async fn run_arm(
        &self,
        spec: &ScenarioSpec,
        ns: &str,
        gated: bool,
        fault: bool,
    ) -> anyhow::Result<ArmOutcome> {
        match spec.kind.as_str() {
            "crm_silent_write" => self.crm(spec, ns, gated, fault, false).await,
            "cascade_welcome" => self.crm(spec, ns, gated, fault, true).await,
            "deploy_false_green" => self.deploy(spec, ns, gated, fault).await,
            "support_false_handoff" => self.handoff(spec, ns, gated, fault).await,
            "accepted_not_indexed" => self.ingestion(spec, ns, gated, fault, false).await,
            "unverified_briefing" => self.ingestion(spec, ns, gated, fault, true).await,
            "empty_evidence" => self.empty_evidence(spec, ns, gated, fault).await,
            "wrong_period" => self.wrong_period(spec, ns, gated, fault).await,
            other => anyhow::bail!("unknown scenario kind: {}", other),
        }
    }

    async fn crm(
        &self,
        spec: &ScenarioSpec,
        ns: &str,
        gated: bool,
        fault: bool,
        cascade: bool,
    ) -> anyhow::Result<ArmOutcome> {
        let arm = if gated { "verified" } else { "baseline" };
        let mut timeline = vec![ev(0, "ROUTE", "create_customer selected", "neutral")];
        let input = json!({"ns":ns,"id":"alice","email":"alice@example.test",
                           "fault": if fault {"silent_noop"} else {""}});
        let start = Instant::now();
        let t0 = Utc::now();
        let response = self.http
            .post(format!("{}/lab/crm/customers", self.base_url))
            .json(&input).send().await?;
        let transport_success = response.status().is_success();
        let output: Value = response.json().await?;
        let action_latency = start.elapsed().as_millis() as u64;
        let reported_success = transport_success && output.get("success").and_then(Value::as_bool) == Some(true);
        timeline.push(ev(action_latency, "EXECUTE", "tool returned success=true", "neutral"));

        let observation = ExecutionObservation {
            invocation_id: Uuid::new_v4(),
            action_id: "create_customer".into(),
            input: input.clone(),
            output: output.clone(),
            transport_success,
            started_at: t0,
            finished_at: Utc::now(),
            latency_ms: action_latency,
        };

        let contract = ActionContract {
            id: "create_customer".into(),
            description: "Create the requested customer record".into(),
            requires_claims: vec![],
            produces_claim_template: Some("verified_customer:{id}".into()),
            verifier: VerifierSpec {
                id: "crm_readback".into(),
                kind: VerifierKind::DbReadback,
                params: json!({"id":"alice"}),
            },
            verifier_attempts: spec.verifier_attempts,
            verifier_backoff_ms: spec.verifier_backoff_ms,
        };

        let verify_start = Instant::now();
        let evidence = self.verify_bool(
            &contract,
            || async {
                let v: Value = self.http.get(format!(
                    "{}/lab/crm/customers/{}/alice", self.base_url, ns
                )).send().await?.json().await?;
                Ok::<(bool, Value), anyhow::Error>((v.get("exists").and_then(Value::as_bool) == Some(true), v))
            }
        ).await?;
        let verifier_latency = verify_start.elapsed().as_millis() as u64;
        let postcondition = evidence.passed;

        let (trusted_success, receipt) = if gated {
            timeline.push(ev(action_latency + 1, "STATE", "SUCCEEDED_UNVERIFIED", "warn"));
            if evidence.passed {
                timeline.push(ev(action_latency + verifier_latency, "VERIFY", "CRM readback PASS", "good"));
                (true, Some(self.signer.sign_pass(&contract, &observation, &evidence)))
            } else {
                timeline.push(ev(action_latency + verifier_latency, "VERIFY", "CRM readback FAIL", "bad"));
                (false, None)
            }
        } else {
            timeline.push(ev(action_latency + 1, "TRUST", "tool response promoted directly", "warn"));
            (reported_success, None)
        };

        let mut downstream_executed = false;
        if cascade && trusted_success {
            downstream_executed = true;
            let _ = self.http.post(format!("{}/lab/welcome", self.base_url))
                .json(&json!({"ns":ns,"customer_id":"alice","message_id":Uuid::new_v4().to_string()}))
                .send().await?;
            timeline.push(ev(action_latency + verifier_latency + 1, "CHAIN", "send_welcome executed", if postcondition {"good"} else {"bad"}));
        } else if cascade && gated && !trusted_success {
            timeline.push(ev(action_latency + verifier_latency + 1, "BLOCK", "send_welcome requires verified_customer:alice", "good"));
        }

        let downstream_safe = !downstream_executed || postcondition;
        let total = start.elapsed().as_millis() as u64;

        if !postcondition && trusted_success {
            timeline.push(ev(total, "RESULT", "FALSE TRUSTED SUCCESS", "bad"));
        } else if gated && !postcondition {
            timeline.push(ev(total, "RESULT", "semantic failure detected", "good"));
        } else {
            timeline.push(ev(total, "RESULT", "true completion", "good"));
        }

        Ok(ArmOutcome {
            arm: arm.into(),
            scenario_id: spec.id.clone(),
            fault_injected: fault,
            reported_success,
            trusted_success,
            postcondition_true: postcondition,
            downstream_executed,
            downstream_safe,
            verifier_passed: if gated { Some(evidence.passed) } else { None },
            action_latency_ms: action_latency,
            verifier_latency_ms: if gated { verifier_latency } else { 0 },
            total_latency_ms: total,
            cost_units: 1.0 + if gated { 0.1 * evidence.attempts as f64 } else { 0.0 }
                + if downstream_executed { 1.0 } else { 0.0 },
            receipt,
            timeline,
            raw: json!({"observation": observation, "verification": if gated {Some(evidence)} else {None}}),
        })
    }

    async fn deploy(
        &self,
        spec: &ScenarioSpec,
        ns: &str,
        gated: bool,
        fault: bool,
    ) -> anyhow::Result<ArmOutcome> {
        let arm = if gated { "verified" } else { "baseline" };
        let mut timeline = vec![ev(0,"ROUTE","trigger_deployment selected","neutral")];
        let input = json!({"ns":ns,"service":"api","fault":if fault {"false_green"} else {""}});
        let start = Instant::now();
        let t0 = Utc::now();
        let response = self.http.post(format!("{}/lab/deployments",self.base_url))
            .json(&input).send().await?;
        let transport_success = response.status().is_success();
        let output: Value = response.json().await?;
        let action_latency = start.elapsed().as_millis() as u64;
        let reported_success = transport_success && output.get("success").and_then(Value::as_bool)==Some(true);
        timeline.push(ev(action_latency,"EXECUTE","deployment API returned success","neutral"));

        let observation = ExecutionObservation{
            invocation_id:Uuid::new_v4(), action_id:"trigger_deployment".into(),
            input:input.clone(), output:output.clone(), transport_success,
            started_at:t0, finished_at:Utc::now(), latency_ms:action_latency
        };
        let contract = ActionContract{
            id:"trigger_deployment".into(),
            description:"Deploy service and reach healthy state".into(),
            requires_claims:vec![], produces_claim_template:Some("healthy_deployment:{service}".into()),
            verifier:VerifierSpec{id:"health_probe".into(),kind:VerifierKind::HealthProbe,params:json!({"service":"api"})},
            verifier_attempts:spec.verifier_attempts, verifier_backoff_ms:spec.verifier_backoff_ms
        };
        let verify_start=Instant::now();
        let evidence=self.verify_bool(&contract,||async{
            let v:Value=self.http.get(format!("{}/lab/deployments/{}/api/health",self.base_url,ns))
                .send().await?.json().await?;
            Ok::<(bool,Value),anyhow::Error>((v.get("healthy").and_then(Value::as_bool)==Some(true),v))
        }).await?;
        let verifier_latency=verify_start.elapsed().as_millis() as u64;
        let postcondition=evidence.passed;
        let (trusted_success,receipt)=if gated{
            timeline.push(ev(action_latency+1,"STATE","SUCCEEDED_UNVERIFIED","warn"));
            if evidence.passed{
                timeline.push(ev(action_latency+verifier_latency,"VERIFY","health probe PASS","good"));
                (true,Some(self.signer.sign_pass(&contract,&observation,&evidence)))
            }else{
                timeline.push(ev(action_latency+verifier_latency,"VERIFY","health probe FAIL","bad"));
                (false,None)
            }
        }else{
            timeline.push(ev(action_latency+1,"TRUST","deployment marked successful from tool output","warn"));
            (reported_success,None)
        };
        let downstream_executed=trusted_success;
        let downstream_safe=!downstream_executed||postcondition;
        if downstream_executed{
            timeline.push(ev(action_latency+verifier_latency+1,"CHAIN","announce_deployment_complete executed",if postcondition{"good"}else{"bad"}));
        }else if gated{
            timeline.push(ev(action_latency+verifier_latency+1,"BLOCK","completion announcement blocked","good"));
        }
        let total=start.elapsed().as_millis() as u64;
        Ok(ArmOutcome{
            arm:arm.into(),scenario_id:spec.id.clone(),fault_injected:fault,reported_success,trusted_success,
            postcondition_true:postcondition,downstream_executed,downstream_safe,
            verifier_passed:if gated{Some(evidence.passed)}else{None},
            action_latency_ms:action_latency,verifier_latency_ms:if gated{verifier_latency}else{0},total_latency_ms:total,
            cost_units:1.0+if gated{0.1*evidence.attempts as f64}else{0.0}+if downstream_executed{0.2}else{0.0},
            receipt,timeline,raw:json!({"observation":observation,"verification":if gated{Some(evidence)}else{None}})
        })
    }

    async fn handoff(
        &self,
        spec: &ScenarioSpec,
        ns: &str,
        gated: bool,
        fault: bool,
    ) -> anyhow::Result<ArmOutcome> {
        let arm=if gated{"verified"}else{"baseline"};
        let mut timeline=vec![ev(0,"ROUTE","escalate_to_human selected","neutral")];
        let input=json!({"ns":ns,"ticket_id":"t-42","fault":if fault{"silent_noop"}else{""}});
        let start=Instant::now(); let t0=Utc::now();
        let response=self.http.post(format!("{}/lab/support/escalations",self.base_url)).json(&input).send().await?;
        let transport_success=response.status().is_success();
        let output:Value=response.json().await?;
        let action_latency=start.elapsed().as_millis() as u64;
        let reported_success=transport_success&&output.get("queued").and_then(Value::as_bool)==Some(true);
        timeline.push(ev(action_latency,"EXECUTE","handoff API returned queued=true","neutral"));
        let observation=ExecutionObservation{invocation_id:Uuid::new_v4(),action_id:"escalate_to_human".into(),input:input.clone(),output:output.clone(),transport_success,started_at:t0,finished_at:Utc::now(),latency_ms:action_latency};
        let contract=ActionContract{id:"escalate_to_human".into(),description:"Ticket exists in human queue".into(),requires_claims:vec![],produces_claim_template:Some("human_handoff:{ticket}".into()),verifier:VerifierSpec{id:"queue_readback".into(),kind:VerifierKind::QueueReadback,params:json!({"ticket":"t-42"})},verifier_attempts:spec.verifier_attempts,verifier_backoff_ms:spec.verifier_backoff_ms};
        let verify_start=Instant::now();
        let evidence=self.verify_bool(&contract,||async{
            let v:Value=self.http.get(format!("{}/lab/support/escalations/{}/t-42",self.base_url,ns)).send().await?.json().await?;
            Ok::<(bool,Value),anyhow::Error>((v.get("queued").and_then(Value::as_bool)==Some(true),v))
        }).await?;
        let verifier_latency=verify_start.elapsed().as_millis() as u64;
        let postcondition=evidence.passed;
        let (trusted_success,receipt)=if gated{
            if evidence.passed{timeline.push(ev(action_latency+verifier_latency,"VERIFY","human queue readback PASS","good"));(true,Some(self.signer.sign_pass(&contract,&observation,&evidence)))}
            else{timeline.push(ev(action_latency+verifier_latency,"VERIFY","human queue readback FAIL","bad"));(false,None)}
        }else{timeline.push(ev(action_latency+1,"TRUST","handoff stored as successful history","warn"));(reported_success,None)};
        let total=start.elapsed().as_millis() as u64;
        Ok(ArmOutcome{arm:arm.into(),scenario_id:spec.id.clone(),fault_injected:fault,reported_success,trusted_success,postcondition_true:postcondition,downstream_executed:false,downstream_safe:true,verifier_passed:if gated{Some(evidence.passed)}else{None},action_latency_ms:action_latency,verifier_latency_ms:if gated{verifier_latency}else{0},total_latency_ms:total,cost_units:1.0+if gated{0.1*evidence.attempts as f64}else{0.0},receipt,timeline,raw:json!({"observation":observation,"verification":if gated{Some(evidence)}else{None}})})
    }

    async fn ingestion(
        &self,
        spec: &ScenarioSpec,
        ns: &str,
        gated: bool,
        fault: bool,
        briefing: bool,
    ) -> anyhow::Result<ArmOutcome> {
        let arm=if gated{"verified"}else{"baseline"};
        let mut timeline=vec![ev(0,"INGEST","upload source","neutral")];
        let input=json!({"ns":ns,"source_id":"source-1","fault":if fault{"accepted_not_indexed"}else{""}});
        let start=Instant::now(); let t0=Utc::now();
        let response=self.http.post(format!("{}/lab/ingestion/upload",self.base_url)).json(&input).send().await?;
        let transport_success=response.status().is_success();
        let output:Value=response.json().await?;
        let action_latency=start.elapsed().as_millis() as u64;
        let reported_success=transport_success&&output.get("accepted").and_then(Value::as_bool)==Some(true);
        timeline.push(ev(action_latency,"EXECUTE","upload returned accepted=true","neutral"));
        let observation=ExecutionObservation{invocation_id:Uuid::new_v4(),action_id:"upload_knowledge".into(),input:input.clone(),output:output.clone(),transport_success,started_at:t0,finished_at:Utc::now(),latency_ms:action_latency};
        let contract=ActionContract{id:"upload_knowledge".into(),description:"Source must be indexed before recall".into(),requires_claims:vec![],produces_claim_template:Some("indexed_source:{id}".into()),verifier:VerifierSpec{id:"verify_processing".into(),kind:VerifierKind::ProcessingStatus,params:json!({"expected":"indexed"})},verifier_attempts:spec.verifier_attempts,verifier_backoff_ms:spec.verifier_backoff_ms};
        let verify_start=Instant::now();
        let evidence=self.verify_bool(&contract,||async{
            let v:Value=self.http.get(format!("{}/lab/ingestion/status/{}/source-1",self.base_url,ns)).send().await?.json().await?;
            Ok::<(bool,Value),anyhow::Error>((v.get("status").and_then(Value::as_str)==Some("indexed"),v))
        }).await?;
        let verifier_latency=verify_start.elapsed().as_millis() as u64;
        let postcondition=evidence.passed;
        let (trusted_success,receipt)=if gated{
            timeline.push(ev(action_latency+1,"STATE","ACCEPTED_UNVERIFIED","warn"));
            if evidence.passed{timeline.push(ev(action_latency+verifier_latency,"VERIFY","processing status indexed","good"));(true,Some(self.signer.sign_pass(&contract,&observation,&evidence)))}
            else{timeline.push(ev(action_latency+verifier_latency,"VERIFY","processing not ready","bad"));(false,None)}
        }else{timeline.push(ev(action_latency+1,"TRUST","upload acceptance treated as ready","warn"));(reported_success,None)};
        let downstream_executed=trusted_success;
        let downstream_safe=!downstream_executed||postcondition;
        if downstream_executed{
            timeline.push(ev(action_latency+verifier_latency+1,"CHAIN",if briefing{"weekly briefing generated/sent"}else{"recall attempted"},if postcondition{"good"}else{"bad"}));
        }else if gated{
            timeline.push(ev(action_latency+verifier_latency+1,"BLOCK",if briefing{"briefing blocked until source indexed"}else{"recall blocked until source indexed"},"good"));
        }
        let total=start.elapsed().as_millis() as u64;
        Ok(ArmOutcome{arm:arm.into(),scenario_id:spec.id.clone(),fault_injected:fault,reported_success,trusted_success,postcondition_true:postcondition,downstream_executed,downstream_safe,verifier_passed:if gated{Some(evidence.passed)}else{None},action_latency_ms:action_latency,verifier_latency_ms:if gated{verifier_latency}else{0},total_latency_ms:total,cost_units:1.0+if gated{0.1*evidence.attempts as f64}else{0.0}+if downstream_executed{0.3}else{0.0},receipt,timeline,raw:json!({"observation":observation,"verification":if gated{Some(evidence)}else{None}})})
    }

    async fn empty_evidence(
        &self,
        spec: &ScenarioSpec,
        _ns: &str,
        gated: bool,
        fault: bool,
    ) -> anyhow::Result<ArmOutcome> {
        let arm=if gated{"verified"}else{"baseline"};
        let start=Instant::now();
        let evidence_count=if fault{0}else{3};
        let input=json!({"question":"What was gross margin in Q4?","evidence_count":evidence_count});
        let output=json!({"success":true,"answer":"Gross margin was 74.2%."});
        let action_latency=1;
        let reported_success=true;
        let observation=ExecutionObservation{invocation_id:Uuid::new_v4(),action_id:"answer_financial_question".into(),input:input.clone(),output:output.clone(),transport_success:true,started_at:Utc::now(),finished_at:Utc::now(),latency_ms:action_latency};
        let contract=ActionContract{id:"answer_financial_question".into(),description:"Answer must have at least one evidence chunk".into(),requires_claims:vec![],produces_claim_template:Some("evidence_backed_answer".into()),verifier:VerifierSpec{id:"evidence_cardinality".into(),kind:VerifierKind::EvidenceCardinality,params:json!({"min":1})},verifier_attempts:spec.verifier_attempts,verifier_backoff_ms:spec.verifier_backoff_ms};
        let verify_start=Instant::now();
        let evidence=VerificationEvidence{verifier_id:"evidence_cardinality".into(),verifier_kind:VerifierKind::EvidenceCardinality,passed:evidence_count>=1,attempts:1,observed:json!({"evidence_count":evidence_count}),reason:if evidence_count>=1{"evidence present".into()}else{"zero evidence chunks".into()},latency_ms:0};
        let verifier_latency=verify_start.elapsed().as_millis() as u64;
        let mut timeline=vec![ev(0,"GENERATE","answer produced","neutral")];
        let postcondition=evidence.passed;
        let (trusted_success,receipt)=if gated{
            if evidence.passed{timeline.push(ev(1,"VERIFY","evidence cardinality PASS","good"));(true,Some(self.signer.sign_pass(&contract,&observation,&evidence)))}
            else{timeline.push(ev(1,"BLOCK","zero chunks: answer not promoted","good"));(false,None)}
        }else{timeline.push(ev(1,"TRUST","generated answer treated as successful output","warn"));(reported_success,None)};
        let total=start.elapsed().as_millis() as u64;
        Ok(ArmOutcome{arm:arm.into(),scenario_id:spec.id.clone(),fault_injected:fault,reported_success,trusted_success,postcondition_true:postcondition,downstream_executed:trusted_success,downstream_safe:!trusted_success||postcondition,verifier_passed:if gated{Some(evidence.passed)}else{None},action_latency_ms:action_latency,verifier_latency_ms:if gated{verifier_latency}else{0},total_latency_ms:total,cost_units:1.0+if gated{0.02}else{0.0},receipt,timeline,raw:json!({"observation":observation,"verification":if gated{Some(evidence)}else{None}})})
    }

    async fn wrong_period(
        &self,
        spec: &ScenarioSpec,
        _ns: &str,
        gated: bool,
        fault: bool,
    ) -> anyhow::Result<ArmOutcome> {
        let arm=if gated{"verified"}else{"baseline"};
        let requested="Q4-2025";
        let observed=if fault{"Q1-2025"}else{"Q4-2025"};
        let input=json!({"question":"What was gross margin in Q4 2025?","requested_period":requested});
        let output=json!({"success":true,"answer":"74.2%","evidence_period":observed});
        let observation=ExecutionObservation{invocation_id:Uuid::new_v4(),action_id:"answer_financial_question".into(),input:input.clone(),output:output.clone(),transport_success:true,started_at:Utc::now(),finished_at:Utc::now(),latency_ms:1};
        let contract=ActionContract{id:"answer_financial_question".into(),description:"Evidence period must match requested fiscal period".into(),requires_claims:vec![],produces_claim_template:Some("period_correct_answer".into()),verifier:VerifierSpec{id:"period_constraint".into(),kind:VerifierKind::MetadataConstraint,params:json!({"requested":requested})},verifier_attempts:spec.verifier_attempts,verifier_backoff_ms:spec.verifier_backoff_ms};
        let passed=requested==observed;
        let evidence=VerificationEvidence{verifier_id:"period_constraint".into(),verifier_kind:VerifierKind::MetadataConstraint,passed,attempts:1,observed:json!({"requested":requested,"observed":observed}),reason:if passed{"period matches".into()}else{"evidence period mismatch".into()},latency_ms:0};
        let mut timeline=vec![ev(0,"GENERATE",format!("answer cites {}",observed),"neutral")];
        let (trusted_success,receipt)=if gated{
            if passed{timeline.push(ev(1,"VERIFY","period constraint PASS","good"));(true,Some(self.signer.sign_pass(&contract,&observation,&evidence)))}
            else{timeline.push(ev(1,"REJECT","wrong fiscal period","good"));(false,None)}
        }else{timeline.push(ev(1,"TRUST","answer accepted without period constraint","warn"));(true,None)};
        Ok(ArmOutcome{arm:arm.into(),scenario_id:spec.id.clone(),fault_injected:fault,reported_success:true,trusted_success,postcondition_true:passed,downstream_executed:trusted_success,downstream_safe:!trusted_success||passed,verifier_passed:if gated{Some(passed)}else{None},action_latency_ms:1,verifier_latency_ms:0,total_latency_ms:1,cost_units:1.0+if gated{0.02}else{0.0},receipt,timeline,raw:json!({"observation":observation,"verification":if gated{Some(evidence)}else{None}})})
    }

    async fn verify_bool<F, Fut>(
        &self,
        contract: &ActionContract,
        mut f: F,
    ) -> anyhow::Result<VerificationEvidence>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<(bool, Value)>>,
    {
        let start = Instant::now();
        let mut observed = json!({});
        let mut passed = false;
        let attempts = contract.verifier_attempts.max(1);

        for i in 0..attempts {
            let (ok, value) = f().await?;
            observed = value;
            if ok {
                passed = true;
                return Ok(VerificationEvidence {
                    verifier_id: contract.verifier.id.clone(),
                    verifier_kind: contract.verifier.kind.clone(),
                    passed,
                    attempts: i + 1,
                    observed,
                    reason: "declared postcondition satisfied".into(),
                    latency_ms: start.elapsed().as_millis() as u64,
                });
            }
            if i + 1 < attempts && contract.verifier_backoff_ms > 0 {
                sleep(Duration::from_millis(contract.verifier_backoff_ms)).await;
            }
        }

        Ok(VerificationEvidence {
            verifier_id: contract.verifier.id.clone(),
            verifier_kind: contract.verifier.kind.clone(),
            passed,
            attempts,
            observed,
            reason: "declared postcondition not satisfied".into(),
            latency_ms: start.elapsed().as_millis() as u64,
        })
    }
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric(){c}else{'_'}).collect()
}

fn ev(at_ms:u64, phase:impl Into<String>, message:impl Into<String>, tone:impl Into<String>)->TimelineEvent{
    TimelineEvent{at_ms,phase:phase.into(),message:message.into(),tone:tone.into()}
}

fn sha256_hex(bytes: impl AsRef<[u8]>) -> String {
    let mut h=Sha256::new(); h.update(bytes.as_ref()); hex::encode(h.finalize())
}

pub fn fault_assignment(seed:u64, scenario_id:&str, trial_index:u64, p:f64)->bool{
    let mut h=Sha256::new();
    h.update(seed.to_le_bytes());
    h.update(scenario_id.as_bytes());
    h.update(trial_index.to_le_bytes());
    let digest=h.finalize();
    let mut seed_bytes=[0u8;32];
    seed_bytes.copy_from_slice(&digest[..32]);
    let mut rng=ChaCha20Rng::from_seed(seed_bytes);
    rng.gen::<f64>() < p
}
