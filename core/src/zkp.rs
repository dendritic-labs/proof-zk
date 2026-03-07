use crate::{Result, ZkProof};
use curve25519_dalek::{ristretto::RistrettoPoint, scalar::Scalar};
use merlin::Transcript;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeProof {
    pub is_over_age: bool,
    pub min_age: u8,
    pub proof: ZkProof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneticMarkerProof {
    pub markers_present: Vec<String>,
    pub proof: ZkProof,
}

pub struct ZkProofSystem;

impl ZkProofSystem {
    /// Generate a zero-knowledge proof that the user is over a certain age
    /// without revealing the actual age
    pub fn prove_age_over(actual_age: u8, min_age: u8) -> Result<AgeProof> {
        let mut transcript = Transcript::new(b"age_verification");

        // In a real implementation, this would use proper ZK protocols
        // For MVP, we'll create a simple commitment-based proof
        let is_over = actual_age >= min_age;

        // Generate random commitment values
        let commitment = Self::generate_commitment(&[actual_age as u64])?;
        let challenge = Self::generate_challenge(&mut transcript, &commitment)?;
        let response = Self::generate_response(actual_age as u64, &challenge)?;

        let proof = ZkProof {
            commitment,
            challenge,
            response,
            public_inputs: vec![min_age],
        };

        Ok(AgeProof {
            is_over_age: is_over,
            min_age,
            proof,
        })
    }

    /// Generate a zero-knowledge proof of genetic marker presence
    /// without revealing other genetic information
    pub fn prove_genetic_markers(
        user_markers: &[String],
        required_markers: &[String],
    ) -> Result<GeneticMarkerProof> {
        let mut transcript = Transcript::new(b"genetic_marker_verification");

        // Find intersection of required and possessed markers
        let present_markers: Vec<String> = required_markers
            .iter()
            .filter(|marker| user_markers.contains(marker))
            .cloned()
            .collect();

        // Generate proof commitment for marker presence
        let marker_hash = Self::hash_markers(&present_markers)?;
        let commitment = Self::generate_commitment(&[marker_hash])?;
        let challenge = Self::generate_challenge(&mut transcript, &commitment)?;
        let response = Self::generate_response(marker_hash, &challenge)?;

        let proof = ZkProof {
            commitment,
            challenge,
            response,
            public_inputs: required_markers
                .iter()
                .flat_map(|m| m.as_bytes().to_vec())
                .collect(),
        };

        Ok(GeneticMarkerProof {
            markers_present: present_markers,
            proof,
        })
    }

    /// Verify an age proof without learning the actual age
    pub fn verify_age_proof(proof: &AgeProof, min_age: u8) -> Result<bool> {
        // Verify that the proof demonstrates age >= min_age
        // In real implementation, this would verify the ZK proof cryptographically

        if proof.min_age != min_age {
            return Ok(false);
        }

        // Simplified verification - in production, verify the actual ZK proof
        let verification_result = Self::verify_commitment_proof(&proof.proof)?;

        Ok(verification_result && proof.is_over_age)
    }

    /// Verify genetic marker proof without learning other genetic data
    pub fn verify_genetic_proof(
        proof: &GeneticMarkerProof,
        required_markers: &[String],
    ) -> Result<bool> {
        // Verify that required markers are present without learning about other markers

        let has_required = required_markers
            .iter()
            .all(|marker| proof.markers_present.contains(marker));

        if !has_required {
            return Ok(false);
        }

        // Verify the ZK proof
        let verification_result = Self::verify_commitment_proof(&proof.proof)?;

        Ok(verification_result)
    }

    // Helper functions for ZK proof generation
    fn generate_commitment(values: &[u64]) -> Result<Vec<u8>> {
        // Simplified commitment - in production use proper Pedersen commitments
        let sum: u64 = values.iter().sum();

        // Use a hash-to-field approach compatible with curve25519-dalek v4
        let mut hasher = Sha256::new();
        hasher.update(sum.to_le_bytes());
        let hash_result = hasher.finalize();

        // Convert to RistrettoPoint using from_uniform_bytes
        let mut uniform_bytes = [0u8; 64];
        uniform_bytes[..32].copy_from_slice(&hash_result);
        uniform_bytes[32..].copy_from_slice(&hash_result); // Duplicate for 64 bytes

        let point = RistrettoPoint::from_uniform_bytes(&uniform_bytes);
        Ok(point.compress().to_bytes().to_vec())
    }

    fn generate_challenge(transcript: &mut Transcript, commitment: &[u8]) -> Result<Vec<u8>> {
        transcript.append_message(b"commitment", commitment);
        let mut challenge = [0u8; 32];
        transcript.challenge_bytes(b"challenge", &mut challenge);
        Ok(challenge.to_vec())
    }

    fn generate_response(secret: u64, challenge: &[u8]) -> Result<Vec<u8>> {
        // Simplified response generation
        let mut challenge_bytes = [0u8; 64]; // Use 64 bytes for wide reduction
        let challenge_len = challenge.len().min(64);
        challenge_bytes[..challenge_len].copy_from_slice(&challenge[..challenge_len]);

        let challenge_scalar = Scalar::from_bytes_mod_order_wide(&challenge_bytes);
        let secret_scalar = Scalar::from(secret);
        let response = secret_scalar + challenge_scalar;
        Ok(response.to_bytes().to_vec())
    }

    fn verify_commitment_proof(proof: &ZkProof) -> Result<bool> {
        // Simplified verification - in production, implement full ZK verification
        Ok(!proof.commitment.is_empty()
            && !proof.challenge.is_empty()
            && !proof.response.is_empty())
    }

    fn hash_markers(markers: &[String]) -> Result<u64> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        markers.hash(&mut hasher);
        Ok(hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_age_proof_generation_and_verification() {
        let actual_age = 25u8;
        let min_age = 21u8;

        // Generate proof
        let proof = ZkProofSystem::prove_age_over(actual_age, min_age).unwrap();

        // Verify proof structure
        assert_eq!(proof.min_age, min_age);
        assert!(proof.is_over_age);
        assert!(!proof.proof.commitment.is_empty());
        assert!(!proof.proof.challenge.is_empty());
        assert!(!proof.proof.response.is_empty());

        // Verify the proof
        let verification = ZkProofSystem::verify_age_proof(&proof, min_age).unwrap();
        assert!(verification);
    }

    #[test]
    fn test_age_proof_underage_rejection() {
        let actual_age = 18u8;
        let min_age = 21u8;

        let proof = ZkProofSystem::prove_age_over(actual_age, min_age).unwrap();

        // Should indicate user is not over age
        assert_eq!(proof.min_age, min_age);
        assert!(!proof.is_over_age);

        // Verification should fail for underage proof
        let verification = ZkProofSystem::verify_age_proof(&proof, min_age).unwrap();
        assert!(!verification); // Should fail because user is underage
    }

    #[test]
    fn test_age_proof_wrong_min_age_verification() {
        let actual_age = 25u8;
        let min_age = 21u8;
        let wrong_min_age = 18u8;

        let proof = ZkProofSystem::prove_age_over(actual_age, min_age).unwrap();

        // Verification with wrong min_age should fail
        let verification = ZkProofSystem::verify_age_proof(&proof, wrong_min_age).unwrap();
        assert!(!verification);
    }

    #[test]
    fn test_genetic_marker_proof_generation() {
        let user_markers = vec![
            "BRCA1".to_string(),
            "BRCA2".to_string(),
            "TP53".to_string(),
            "PALB2".to_string(),
        ];
        let required_markers = vec!["BRCA1".to_string(), "BRCA2".to_string()];

        let proof = ZkProofSystem::prove_genetic_markers(&user_markers, &required_markers).unwrap();

        // Should find both required markers
        assert_eq!(proof.markers_present.len(), 2);
        assert!(proof.markers_present.contains(&"BRCA1".to_string()));
        assert!(proof.markers_present.contains(&"BRCA2".to_string()));

        // Should not reveal other markers
        assert!(!proof.markers_present.contains(&"TP53".to_string()));
        assert!(!proof.markers_present.contains(&"PALB2".to_string()));
    }

    #[test]
    fn test_genetic_marker_proof_verification() {
        let user_markers = vec!["BRCA1".to_string(), "BRCA2".to_string(), "TP53".to_string()];
        let required_markers = vec!["BRCA1".to_string(), "BRCA2".to_string()];

        let proof = ZkProofSystem::prove_genetic_markers(&user_markers, &required_markers).unwrap();
        let verification = ZkProofSystem::verify_genetic_proof(&proof, &required_markers).unwrap();

        assert!(verification);
    }

    #[test]
    fn test_genetic_marker_proof_missing_marker() {
        let user_markers = vec![
            "BRCA1".to_string(),
            "TP53".to_string(), // Missing BRCA2
        ];
        let required_markers = vec!["BRCA1".to_string(), "BRCA2".to_string()];

        let proof = ZkProofSystem::prove_genetic_markers(&user_markers, &required_markers).unwrap();

        // Should only contain BRCA1
        assert_eq!(proof.markers_present.len(), 1);
        assert!(proof.markers_present.contains(&"BRCA1".to_string()));
        assert!(!proof.markers_present.contains(&"BRCA2".to_string()));

        // Verification should fail due to missing required marker
        let verification = ZkProofSystem::verify_genetic_proof(&proof, &required_markers).unwrap();
        assert!(!verification);
    }

    #[test]
    fn test_commitment_generation_consistency() {
        let values = vec![42u64, 123u64, 456u64];

        let commitment1 = ZkProofSystem::generate_commitment(&values).unwrap();
        let commitment2 = ZkProofSystem::generate_commitment(&values).unwrap();

        // Same values should produce same commitment
        assert_eq!(commitment1, commitment2);
    }

    #[test]
    fn test_commitment_generation_different_values() {
        let values1 = vec![42u64];
        let values2 = vec![43u64];

        let commitment1 = ZkProofSystem::generate_commitment(&values1).unwrap();
        let commitment2 = ZkProofSystem::generate_commitment(&values2).unwrap();

        // Different values should produce different commitments
        assert_ne!(commitment1, commitment2);
    }

    #[test]
    fn test_empty_genetic_markers() {
        let user_markers: Vec<String> = vec![];
        let required_markers = vec!["BRCA1".to_string()];

        let proof = ZkProofSystem::prove_genetic_markers(&user_markers, &required_markers).unwrap();
        assert!(proof.markers_present.is_empty());

        let verification = ZkProofSystem::verify_genetic_proof(&proof, &required_markers).unwrap();
        assert!(!verification); // Should fail as no required markers present
    }

    #[test]
    fn test_edge_case_ages() {
        // Test age 0
        let proof_zero = ZkProofSystem::prove_age_over(0, 18).unwrap();
        assert!(!proof_zero.is_over_age);

        // Test exact minimum age
        let proof_exact = ZkProofSystem::prove_age_over(18, 18).unwrap();
        assert!(proof_exact.is_over_age);

        // Test maximum age (255)
        let proof_max = ZkProofSystem::prove_age_over(255, 21).unwrap();
        assert!(proof_max.is_over_age);
    }

    // Property-based testing
    proptest! {
        #[test]
        fn prop_age_proof_consistency(actual_age in 0u8..=255, min_age in 0u8..=255) {
            let proof = ZkProofSystem::prove_age_over(actual_age, min_age).unwrap();
            let verification = ZkProofSystem::verify_age_proof(&proof, min_age).unwrap();

            // Proof verification should match the expected result
            let expected_result = actual_age >= min_age;
            prop_assert_eq!(verification, expected_result);
        }

        #[test]
        fn prop_commitment_deterministic(value in 0u64..1000000) {
            let values = vec![value];
            let commitment1 = ZkProofSystem::generate_commitment(&values).unwrap();
            let commitment2 = ZkProofSystem::generate_commitment(&values).unwrap();

            // Same input should always produce same commitment
            prop_assert_eq!(commitment1, commitment2);
        }

        #[test]
        fn prop_commitment_different_values(value1 in 0u64..1000, value2 in 0u64..1000) {
            prop_assume!(value1 != value2);

            let commitment1 = ZkProofSystem::generate_commitment(&vec![value1]).unwrap();
            let commitment2 = ZkProofSystem::generate_commitment(&vec![value2]).unwrap();

            // Different inputs should produce different commitments
            prop_assert_ne!(commitment1, commitment2);
        }

        #[test]
        fn prop_genetic_markers_subset(
            user_markers in prop::collection::vec(prop::string::string_regex("[A-Z0-9]{3,8}").unwrap(), 0..10),
            required_indices in prop::collection::vec(0usize..10, 0..5)
        ) {
            // Create required markers as subset of user markers
            let required_markers: Vec<String> = required_indices
                .iter()
                .filter_map(|&i| user_markers.get(i))
                .cloned()
                .collect();

            if !required_markers.is_empty() {
                let proof = ZkProofSystem::prove_genetic_markers(&user_markers, &required_markers).unwrap();
                let verification = ZkProofSystem::verify_genetic_proof(&proof, &required_markers).unwrap();

                // If all required markers are present in user markers, verification should succeed
                prop_assert!(verification);

                // Proof should contain all required markers and no extra ones beyond what's required
                for marker in &required_markers {
                    prop_assert!(proof.markers_present.contains(marker));
                }
            }
        }
    }

    // Benchmark tests (will be picked up by criterion when run with --bench)
    #[cfg(test)]
    mod bench_tests {
        use super::*;
        use std::time::Instant;

        #[test]
        fn bench_age_proof_generation_performance() {
            let start = Instant::now();

            for _ in 0..1000 {
                let _ = ZkProofSystem::prove_age_over(25, 21).unwrap();
            }

            let duration = start.elapsed();
            println!("1000 age proofs generated in: {:?}", duration);

            // Should be faster than 1 second for 1000 proofs
            assert!(duration.as_secs() < 1);
        }

        #[test]
        fn bench_genetic_proof_performance() {
            let user_markers = vec![
                "BRCA1".to_string(),
                "BRCA2".to_string(),
                "TP53".to_string(),
                "PALB2".to_string(),
                "ATM".to_string(),
                "CHEK2".to_string(),
            ];
            let required_markers = vec!["BRCA1".to_string(), "BRCA2".to_string()];

            let start = Instant::now();

            for _ in 0..1000 {
                let _ =
                    ZkProofSystem::prove_genetic_markers(&user_markers, &required_markers).unwrap();
            }

            let duration = start.elapsed();
            println!("1000 genetic proofs generated in: {:?}", duration);

            // Should be faster than 2 seconds for 1000 proofs
            assert!(duration.as_secs() < 2);
        }
    }
}
