use proofzk_core::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔐 ProofZK - DID-based Privacy Service");
    println!("=====================================");
    
    let mut proof_system = ProofZK::new();
    
    // Example: Age verification request
    let age_request = ProofRequest {
        id: uuid::Uuid::new_v4(),
        requester: "airline-checkin-service".to_string(),
        proof_type: ProofType::AgeVerification { min_age: 18 },
        required_claims: vec!["age_over_18".to_string()],
        context: "airline_boarding".to_string(),
        expires_at: chrono::Utc::now() + chrono::Duration::minutes(5),
    };
    
    let request_id = proof_system.create_proof_request(age_request).await?;
    println!("✅ Created proof request: {}", request_id);
    
    // Example: Genetic marker request
    let genetic_request = ProofRequest {
        id: uuid::Uuid::new_v4(),
        requester: "genomics-health-service".to_string(),
        proof_type: ProofType::GeneticMarker { 
            markers: vec!["BRCA1".to_string(), "APOE4".to_string()] 
        },
        required_claims: vec!["genetic_predisposition".to_string()],
        context: "personalized_medicine".to_string(),
        expires_at: chrono::Utc::now() + chrono::Duration::minutes(10),
    };
    
    let genetic_id = proof_system.create_proof_request(genetic_request).await?;
    println!("✅ Created genetic proof request: {}", genetic_id);
    
    println!("\n🔗 Connect to Elixir relay service on port 4000");
    println!("📱 Wallet integration ready for Apple/Google/Samsung wallets");
    
    Ok(())
}