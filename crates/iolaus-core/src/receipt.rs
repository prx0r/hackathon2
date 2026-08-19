use chrono::Utc;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{ActionContract, ExecutionObservation, ReceiptPayload, SignedReceipt, VerificationEvidence};

pub fn canonical_json<T: Serialize>(value: &T) -> Vec<u8> {
    // serde_json preserves struct field order; all receipt-bearing structures are structs,
    // not arbitrary maps. For Value inputs we hash their serialized representation as observed.
    serde_json::to_vec(value).expect("serializable")
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

pub fn sha256_json<T: Serialize>(value: &T) -> String {
    sha256_bytes(&canonical_json(value))
}

#[derive(Clone)]
pub struct ReceiptSigner {
    signing: SigningKey,
}

impl ReceiptSigner {
    pub fn generate() -> Self {
        let mut secret = [0u8; 32];
        OsRng.fill_bytes(&mut secret);
        Self { signing: SigningKey::from_bytes(&secret) }
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(self.signing.verifying_key().to_bytes())
    }

    pub fn sign_pass(
        &self,
        contract: &ActionContract,
        observation: &ExecutionObservation,
        evidence: &VerificationEvidence,
    ) -> SignedReceipt {
        assert!(evidence.passed, "receipts are only issued for PASS evidence");

        let payload = ReceiptPayload {
            receipt_id: Uuid::new_v4(),
            contract_id: contract.id.clone(),
            contract_digest: contract.digest(),
            invocation_id: observation.invocation_id,
            input_hash: sha256_json(&observation.input),
            output_hash: sha256_json(&observation.output),
            evidence_hash: sha256_json(evidence),
            verifier_id: evidence.verifier_id.clone(),
            verifier_kind: evidence.verifier_kind.clone(),
            verdict: "PASS".to_string(),
            issued_at: Utc::now(),
        };

        let payload_bytes = canonical_json(&payload);
        let signature: Signature = self.signing.sign(&payload_bytes);
        let signer_public_key_hex = self.public_key_hex();
        let signature_hex = hex::encode(signature.to_bytes());

        let envelope_hash = sha256_json(&serde_json::json!({
            "payload": &payload,
            "signer_public_key_hex": &signer_public_key_hex,
            "signature_hex": &signature_hex,
        }));

        SignedReceipt {
            payload,
            signer_public_key_hex,
            signature_hex,
            envelope_hash,
        }
    }

    pub fn verify(receipt: &SignedReceipt) -> anyhow::Result<()> {
        let pk_bytes: [u8; 32] = hex::decode(&receipt.signer_public_key_hex)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("public key must be 32 bytes"))?;
        let sig_bytes: [u8; 64] = hex::decode(&receipt.signature_hex)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("signature must be 64 bytes"))?;

        let key = VerifyingKey::from_bytes(&pk_bytes)?;
        let sig = Signature::from_bytes(&sig_bytes);
        key.verify(&canonical_json(&receipt.payload), &sig)?;

        let expected = sha256_json(&serde_json::json!({
            "payload": &receipt.payload,
            "signer_public_key_hex": &receipt.signer_public_key_hex,
            "signature_hex": &receipt.signature_hex,
        }));
        anyhow::ensure!(expected == receipt.envelope_hash, "envelope hash mismatch");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{VerifierKind, VerifierSpec};

    #[test]
    fn signed_receipt_roundtrip() {
        let signer = ReceiptSigner::generate();
        let contract = ActionContract {
            id: "create_customer".into(),
            description: "create".into(),
            requires_claims: vec![],
            produces_claim_template: Some("verified_customer:{id}".into()),
            verifier: VerifierSpec {
                id: "crm_readback".into(),
                kind: VerifierKind::DbReadback,
                params: serde_json::json!({}),
            },
            verifier_attempts: 1,
            verifier_backoff_ms: 0,
        };
        let now = Utc::now();
        let obs = ExecutionObservation {
            invocation_id: Uuid::new_v4(),
            action_id: contract.id.clone(),
            input: serde_json::json!({"id":"a"}),
            output: serde_json::json!({"success":true}),
            transport_success: true,
            started_at: now,
            finished_at: now,
            latency_ms: 1,
        };
        let evidence = VerificationEvidence {
            verifier_id: "crm_readback".into(),
            verifier_kind: VerifierKind::DbReadback,
            passed: true,
            attempts: 1,
            observed: serde_json::json!({"exists":true}),
            reason: "row exists".into(),
            latency_ms: 1,
        };
        let receipt = signer.sign_pass(&contract, &obs, &evidence);
        ReceiptSigner::verify(&receipt).unwrap();
    }
}
