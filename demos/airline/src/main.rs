use proofzk_core::proofs::*;
use proofzk_core::wallet::*;
use proofzk_core::*;
use std::collections::HashMap;

/// Airline check-in demo showing privacy-preserving age verification
#[tokio::main]
async fn main() -> Result<()> {
    println!("ProofZK Airline Check-in Demo");
    println!("==================================");
    println!("Demonstrating age verification WITHOUT storing personal data\n");

    // Initialize wallet integration
    let wallet_integration = WalletIntegration::new();

    // Simulate passenger data (stored only in user's wallet)
    let passenger_age = 25;
    let min_required_age = 18;

    println!("Passenger opens airline app...");
    println!(
        "App requests proof of age ≥ {} (no birthdate stored)",
        min_required_age
    );

    // Create ephemeral session for this transaction
    let session_data = serde_json::json!({
        "transaction_type": "age_verification",
        "requester": "airline_checkin",
        "flight": "UA123",
        "timestamp": chrono::Utc::now()
    });

    println!("⏳ Creating ephemeral session (expires in 5 minutes)...");

    // Generate privacy-preserving proof
    println!("Generating zero-knowledge age proof...");
    let (age_proof, wallet_response) =
        create_age_verification_proof(passenger_age, min_required_age, &wallet_integration).await?;

    println!("Proof generated:");
    println!(
        "   - Passenger age ≥ {}: {}",
        min_required_age, age_proof.is_over_age
    );
    println!("   - Actual age disclosed: NO");
    println!("   - Birthdate disclosed: NO");
    println!("   - Name disclosed: NO");

    // Verify the proof
    println!("\nAirline verifying proof (without learning actual age)...");
    let verification_result =
        verify_age_proof_complete(&age_proof, &wallet_response, min_required_age).await?;

    if verification_result {
        println!("AGE VERIFICATION SUCCESSFUL");
        println!("Check-in approved!");
        println!("Session data will auto-expire in 5 minutes");
    } else {
        println!("Age verification failed");
        return Ok(());
    }

    // Demonstrate what airline system knows vs doesn't know
    println!("\nPrivacy Analysis:");
    println!(
        "   Airline knows: Passenger is ≥ {} years old",
        min_required_age
    );
    println!("   Airline does NOT know:");
    println!("      - Exact age");
    println!("      - Birthdate");
    println!("      - Name (unless separately provided)");
    println!("      - Any other personal data");

    println!("\nDemonstrating ephemeral relay:");
    println!("   - Transaction data stored temporarily");
    println!("   - Auto-expires after successful verification");
    println!("   - No central database of passenger data");

    // Simulate Apple Wallet integration
    println!("\nApple Wallet Integration:");
    let apple_config =
        wallet_integration.generate_apple_wallet_integration("did:proofzk:passenger123")?;
    println!("   {}", serde_json::to_string_pretty(&apple_config)?);

    println!("\nDemo complete! Privacy preserved, verification successful.");

    Ok(())
}
