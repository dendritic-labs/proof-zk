use serde::{Deserialize, Serialize};
use crate::{zkp::*, wallet::*, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOrchestrator {
    pub request_id: uuid::Uuid,
    pub requester: String,
    pub proof_type: String,
    pub status: ProofStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofStatus {
    Pending,
    WalletRequested,
    ProofGenerated,
    Verified,
    Rejected,
}

/// Age verification proof for airline check-in
pub async fn create_age_verification_proof(
    user_age: u8,
    min_age: u8,
    _wallet_integration: &WalletIntegration,
) -> Result<(AgeProof, WalletResponse)> {
    // Generate ZK proof without revealing actual age
    let zk_proof = ZkProofSystem::prove_age_over(user_age, min_age)?;
    
    // Request selective disclosure from wallet (just age verification, not birthdate)
    let _disclosure_request = SelectiveDisclosureRequest {
        fields_requested: vec!["age_over_18".to_string()],
        purpose: "airline_age_verification".to_string(),
        requester_did: "did:proofzk:airline-system".to_string(),
        selective_fields: {
            let mut fields = std::collections::HashMap::new();
            fields.insert("age_over_18".to_string(), true);
            fields.insert("birthdate".to_string(), false); // Not disclosed
            fields.insert("name".to_string(), false); // Not disclosed
            fields
        },
    };
    
    // Simulate wallet response (in production, this goes to actual wallet)
    let wallet_response = WalletResponse {
        credential_id: "mock_credential".to_string(),
        disclosed_fields: {
            let mut fields = std::collections::HashMap::new();
            fields.insert("age_over_18".to_string(), serde_json::Value::Bool(user_age >= min_age));
            fields
        },
        proof_of_possession: vec![0u8; 32], // Mock signature
        timestamp: chrono::Utc::now(),
    };
    
    Ok((zk_proof, wallet_response))
}

/// Genetic marker proof for genomics applications
pub async fn create_genetic_marker_proof(
    user_markers: &[String],
    requested_markers: &[String],
    _wallet_integration: &WalletIntegration,
) -> Result<(GeneticMarkerProof, WalletResponse)> {
    // Generate ZK proof of specific markers without revealing full genome
    let zk_proof = ZkProofSystem::prove_genetic_markers(user_markers, requested_markers)?;
    
    // Request selective disclosure from wallet
    let _disclosure_request = SelectiveDisclosureRequest {
        fields_requested: requested_markers.iter().map(|m| format!("genetic_marker_{}", m.to_lowercase())).collect(),
        purpose: "personalized_health_recommendations".to_string(),
        requester_did: "did:proofzk:genomics-service".to_string(),
        selective_fields: {
            let mut fields = std::collections::HashMap::new();
            for marker in requested_markers {
                fields.insert(format!("genetic_marker_{}", marker.to_lowercase()), true);
            }
            // Explicitly exclude other sensitive genetic data
            fields.insert("full_genome".to_string(), false);
            fields.insert("family_history".to_string(), false);
            fields
        },
    };
    
    // Simulate wallet response with only requested markers
    let wallet_response = WalletResponse {
        credential_id: "genetic_credential".to_string(),
        disclosed_fields: {
            let mut fields = std::collections::HashMap::new();
            for marker in requested_markers {
                let has_marker = user_markers.contains(marker);
                fields.insert(
                    format!("genetic_marker_{}", marker.to_lowercase()), 
                    serde_json::Value::Bool(has_marker)
                );
            }
            fields
        },
        proof_of_possession: vec![1u8; 32], // Mock signature
        timestamp: chrono::Utc::now(),
    };
    
    Ok((zk_proof, wallet_response))
}

/// Verify age proof without learning the actual age
pub async fn verify_age_proof_complete(
    proof: &AgeProof,
    wallet_response: &WalletResponse,
    min_age: u8,
) -> Result<bool> {
    // Verify ZK proof
    let zk_valid = ZkProofSystem::verify_age_proof(proof, min_age)?;
    
    // Verify wallet response consistency
    let wallet_valid = wallet_response.disclosed_fields
        .get("age_over_18")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    // Both proofs must be valid and consistent
    Ok(zk_valid && wallet_valid && proof.is_over_age == wallet_valid)
}

/// Verify genetic marker proof without learning other genetic information
pub async fn verify_genetic_proof_complete(
    proof: &GeneticMarkerProof,
    wallet_response: &WalletResponse,
    required_markers: &[String],
) -> Result<bool> {
    // Verify ZK proof
    let zk_valid = ZkProofSystem::verify_genetic_proof(proof, required_markers)?;
    
    // Verify wallet response has required markers
    let wallet_valid = required_markers.iter().all(|marker| {
        wallet_response.disclosed_fields
            .get(&format!("genetic_marker_{}", marker.to_lowercase()))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    });
    
    Ok(zk_valid && wallet_valid)
}