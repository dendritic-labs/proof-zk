/// Integration tests for ProofZK system components
use proofzk_core::{
    zkp::{ZkProofSystem, AgeProof, GeneticMarkerProof},
    ProofRequest, ProofType, ZkProof,
};
use std::time::SystemTime;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[tokio::test]
async fn test_end_to_end_age_verification_flow() {
    // Simulate airline check-in scenario
    let passenger_age = 25u8;
    let required_min_age = 21u8;
    
    // 1. Create proof request (from airline)
    let proof_request = ProofRequest {
        id: Uuid::new_v4(),
        requester: "Delta Airlines".to_string(),
        proof_type: ProofType::AgeVerification { 
            min_age: required_min_age 
        },
        required_claims: vec!["age_over_21".to_string()],
        context: "Alcohol purchase on flight".to_string(),
        expires_at: DateTime::from(SystemTime::now()) + chrono::Duration::hours(1),
    };
    
    // 2. Generate proof (from passenger's device)
    let age_proof = ZkProofSystem::prove_age_over(passenger_age, required_min_age)
        .expect("Should generate valid age proof");
    
    // 3. Verify proof (by airline system)
    let verification_result = ZkProofSystem::verify_age_proof(&age_proof, required_min_age)
        .expect("Should verify proof without error");
    
    assert!(verification_result);
    assert_eq!(age_proof.min_age, required_min_age);
    assert!(age_proof.is_over_age);
}

#[tokio::test]
async fn test_genetic_research_consent_flow() {
    // Simulate medical research scenario
    let patient_genetic_markers = vec![
        "BRCA1".to_string(),
        "BRCA2".to_string(),
        "TP53".to_string(),
        "ATM".to_string(),
        "PALB2".to_string(),
    ];
    
    let research_required_markers = vec![
        "BRCA1".to_string(),
        "BRCA2".to_string(),
    ];
    
    // 1. Create proof request (from research institution)
    let proof_request = ProofRequest {
        id: Uuid::new_v4(),
        requester: "Cancer Research Institute".to_string(),
        proof_type: ProofType::GeneticMarker { 
            markers: research_required_markers.clone() 
        },
        required_claims: vec!["brca_variant_status".to_string()],
        context: "Breast cancer risk study participation".to_string(),
        expires_at: DateTime::from(SystemTime::now()) + chrono::Duration::days(30),
    };
    
    // 2. Generate proof (from patient's secure health wallet)
    let genetic_proof = ZkProofSystem::prove_genetic_markers(
        &patient_genetic_markers, 
        &research_required_markers
    ).expect("Should generate valid genetic proof");
    
    // 3. Verify proof (by research system)
    let verification_result = ZkProofSystem::verify_genetic_proof(
        &genetic_proof, 
        &research_required_markers
    ).expect("Should verify proof without error");
    
    assert!(verification_result);
    assert_eq!(genetic_proof.markers_present.len(), 2);
    assert!(genetic_proof.markers_present.contains(&"BRCA1".to_string()));
    assert!(genetic_proof.markers_present.contains(&"BRCA2".to_string()));
    
    // Privacy check: should not reveal other genetic markers
    assert!(!genetic_proof.markers_present.contains(&"TP53".to_string()));
    assert!(!genetic_proof.markers_present.contains(&"ATM".to_string()));
}

#[test]
fn test_proof_serialization_deserialization() {
    let age_proof = ZkProofSystem::prove_age_over(30, 21).unwrap();
    
    // Test JSON serialization
    let json_str = serde_json::to_string(&age_proof).expect("Should serialize to JSON");
    let deserialized_proof: AgeProof = serde_json::from_str(&json_str)
        .expect("Should deserialize from JSON");
    
    assert_eq!(age_proof.min_age, deserialized_proof.min_age);
    assert_eq!(age_proof.is_over_age, deserialized_proof.is_over_age);
    assert_eq!(age_proof.proof.commitment, deserialized_proof.proof.commitment);
}

#[test]
fn test_multiple_concurrent_proofs() {
    use std::sync::Arc;
    use std::thread;
    
    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                let age = 20 + (i % 10) as u8;
                let min_age = 21u8;
                
                let proof = ZkProofSystem::prove_age_over(age, min_age).unwrap();
                let verification = ZkProofSystem::verify_age_proof(&proof, min_age).unwrap();
                
                (age >= min_age, verification)
            })
        })
        .collect();
    
    for handle in handles {
        let (expected, actual) = handle.join().unwrap();
        assert_eq!(expected, actual);
    }
}

#[test]
fn test_proof_request_validation() {
    let now = SystemTime::now();
    
    // Valid request
    let valid_request = ProofRequest {
        id: Uuid::new_v4(),
        requester: "TestCorp".to_string(),
        proof_type: ProofType::AgeVerification { min_age: 18 },
        required_claims: vec!["age_verification".to_string()],
        context: "Service access".to_string(),
        expires_at: DateTime::from(now) + chrono::Duration::hours(1),
    };
    
    // Should be able to serialize/deserialize
    let json = serde_json::to_string(&valid_request).unwrap();
    let _: ProofRequest = serde_json::from_str(&json).unwrap();
}

#[test]
fn test_privacy_guarantees() {
    // Test that proofs don't leak information beyond the claim
    let actual_age = 35u8;
    let min_age1 = 21u8;
    let min_age2 = 30u8;
    
    let proof1 = ZkProofSystem::prove_age_over(actual_age, min_age1).unwrap();
    let proof2 = ZkProofSystem::prove_age_over(actual_age, min_age2).unwrap();
    
    // Both proofs should verify for their respective requirements
    assert!(ZkProofSystem::verify_age_proof(&proof1, min_age1).unwrap());
    assert!(ZkProofSystem::verify_age_proof(&proof2, min_age2).unwrap());
    
    // Cross-verification should fail (different min_age parameters)
    assert!(!ZkProofSystem::verify_age_proof(&proof1, min_age2).unwrap());
    assert!(!ZkProofSystem::verify_age_proof(&proof2, min_age1).unwrap());
}

#[test]  
fn test_genetic_marker_privacy() {
    let comprehensive_markers = vec![
        "BRCA1".to_string(), "BRCA2".to_string(), "TP53".to_string(),
        "ATM".to_string(), "PALB2".to_string(), "CHEK2".to_string(),
        "MLH1".to_string(), "MSH2".to_string(), "MSH6".to_string(),
    ];
    
    // Different research studies requiring different marker sets
    let cancer_markers = vec!["BRCA1".to_string(), "BRCA2".to_string()];
    let lynch_markers = vec!["MLH1".to_string(), "MSH2".to_string()];
    
    let cancer_proof = ZkProofSystem::prove_genetic_markers(
        &comprehensive_markers, 
        &cancer_markers
    ).unwrap();
    
    let lynch_proof = ZkProofSystem::prove_genetic_markers(
        &comprehensive_markers,
        &lynch_markers  
    ).unwrap();
    
    // Each proof should only reveal the requested markers
    assert_eq!(cancer_proof.markers_present.len(), 2);
    assert!(cancer_proof.markers_present.contains(&"BRCA1".to_string()));
    assert!(cancer_proof.markers_present.contains(&"BRCA2".to_string()));
    assert!(!cancer_proof.markers_present.contains(&"MLH1".to_string()));
    
    assert_eq!(lynch_proof.markers_present.len(), 2);
    assert!(lynch_proof.markers_present.contains(&"MLH1".to_string()));
    assert!(lynch_proof.markers_present.contains(&"MSH2".to_string()));
    assert!(!lynch_proof.markers_present.contains(&"BRCA1".to_string()));
    
    // Verify proofs independently
    assert!(ZkProofSystem::verify_genetic_proof(&cancer_proof, &cancer_markers).unwrap());
    assert!(ZkProofSystem::verify_genetic_proof(&lynch_proof, &lynch_markers).unwrap());
}

#[test]
fn test_performance_constraints() {
    use std::time::Instant;
    
    // Test that proof generation meets performance requirements
    let start = Instant::now();
    let _proof = ZkProofSystem::prove_age_over(25, 21).unwrap();
    let generation_time = start.elapsed();
    
    // Should generate proof in under 10ms (target from CLAUDE.md)
    assert!(generation_time.as_millis() < 10, 
        "Proof generation took {}ms, should be < 10ms", 
        generation_time.as_millis());
    
    // Test verification performance
    let proof = ZkProofSystem::prove_age_over(25, 21).unwrap();
    let start = Instant::now();
    let _result = ZkProofSystem::verify_age_proof(&proof, 21).unwrap();
    let verification_time = start.elapsed();
    
    // Should verify proof in under 5ms (target from CLAUDE.md)
    assert!(verification_time.as_millis() < 5,
        "Proof verification took {}ms, should be < 5ms",
        verification_time.as_millis());
}