use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransitionStatus {
    Pending,
    Blocked,
    SucceededUnverified,
    Verified,
    Rejected,
    ExecutionFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerifierKind {
    DbReadback,
    HealthProbe,
    QueueReadback,
    ProcessingStatus,
    EvidenceCardinality,
    MetadataConstraint,
    HumanArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierSpec {
    pub id: String,
    pub kind: VerifierKind,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionContract {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub requires_claims: Vec<String>,
    pub produces_claim_template: Option<String>,
    pub verifier: VerifierSpec,
    #[serde(default = "default_attempts")]
    pub verifier_attempts: u32,
    #[serde(default)]
    pub verifier_backoff_ms: u64,
}

fn default_attempts() -> u32 { 1 }

impl ActionContract {
    pub fn digest(&self) -> String {
        crate::receipt::sha256_json(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionObservation {
    pub invocation_id: Uuid,
    pub action_id: String,
    pub input: Value,
    pub output: Value,
    pub transport_success: bool,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationEvidence {
    pub verifier_id: String,
    pub verifier_kind: VerifierKind,
    pub passed: bool,
    pub attempts: u32,
    pub observed: Value,
    pub reason: String,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub contract_id: String,
    pub invocation_id: Uuid,
    pub status: TransitionStatus,
    pub observation: ExecutionObservation,
    pub verification: Option<VerificationEvidence>,
    pub receipt: Option<SignedReceipt>,
    pub produced_claim: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptPayload {
    pub receipt_id: Uuid,
    pub contract_id: String,
    pub contract_digest: String,
    pub invocation_id: Uuid,
    pub input_hash: String,
    pub output_hash: String,
    pub evidence_hash: String,
    pub verifier_id: String,
    pub verifier_kind: VerifierKind,
    pub verdict: String,
    pub issued_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedReceipt {
    pub payload: ReceiptPayload,
    pub signer_public_key_hex: String,
    pub signature_hex: String,
    pub envelope_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub at_ms: u64,
    pub phase: String,
    pub message: String,
    pub tone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmOutcome {
    pub arm: String,
    pub scenario_id: String,
    pub fault_injected: bool,
    pub reported_success: bool,
    pub trusted_success: bool,
    pub postcondition_true: bool,
    pub downstream_executed: bool,
    pub downstream_safe: bool,
    pub verifier_passed: Option<bool>,
    pub action_latency_ms: u64,
    pub verifier_latency_ms: u64,
    pub total_latency_ms: u64,
    pub cost_units: f64,
    pub receipt: Option<SignedReceipt>,
    pub timeline: Vec<TimelineEvent>,
    #[serde(default)]
    pub raw: Value,
}

impl ArmOutcome {
    pub fn false_trusted_success(&self) -> bool {
        self.trusted_success && !self.postcondition_true
    }

    pub fn contaminated_downstream(&self) -> bool {
        self.downstream_executed && !self.downstream_safe
    }

    pub fn true_completion(&self) -> bool {
        self.trusted_success && self.postcondition_true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedTrial {
    pub scenario_id: String,
    pub trial_index: u64,
    pub seed: u64,
    pub fault_injected: bool,
    pub baseline: ArmOutcome,
    pub verified: ArmOutcome,
}
