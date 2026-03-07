use proofzk_core::proofs::*;
use proofzk_core::wallet::*;
use proofzk_core::*;

/// Genomics application demo showing selective genetic marker disclosure
#[tokio::main]
async fn main() -> Result<()> {
    println!("ProofZK Genomics Demo");
    println!("========================");
    println!("Demonstrating selective genetic marker disclosure WITHOUT full genome access\n");

    // Initialize wallet integration
    let wallet_integration = WalletIntegration::new();

    // Simulate user's genetic profile (stored only in secure wallet)
    let user_genetic_markers = vec![
        "BRCA1".to_string(),  // Breast cancer risk
        "APOE4".to_string(),  // Alzheimer's risk
        "CYP2D6".to_string(), // Drug metabolism
        "MTHFR".to_string(),  // Folate metabolism
        "COMT".to_string(),   // Dopamine metabolism
    ];

    // Health service requests specific markers for personalized recommendations
    let requested_markers = vec![
        "BRCA1".to_string(), // For cancer screening recommendations
        "MTHFR".to_string(), // For nutrition recommendations
    ];

    println!("Health service requests genetic markers:");
    for marker in &requested_markers {
        println!("   - {}", marker);
    }
    println!("   (Full genome NOT requested or accessed)\n");

    // Create ephemeral session for this genomics transaction
    let session_data = serde_json::json!({
        "transaction_type": "genetic_marker_verification",
        "requester": "personalized_health_service",
        "requested_markers": requested_markers,
        "purpose": "personalized_nutrition_recommendations",
        "timestamp": chrono::Utc::now()
    });

    println!("⏳ Creating ephemeral session (expires in 10 minutes)...");

    // Generate privacy-preserving genetic proof
    println!("Generating zero-knowledge genetic marker proof...");
    let (genetic_proof, wallet_response) = create_genetic_marker_proof(
        &user_genetic_markers,
        &requested_markers,
        &wallet_integration,
    )
    .await?;

    println!("Proof generated:");
    println!(
        "   - Markers disclosed: {:?}",
        genetic_proof.markers_present
    );
    println!("   - Full genome disclosed: NO");
    println!("   - Other markers disclosed: NO");
    println!("   - Family history disclosed: NO");

    // Verify the proof
    println!("\nHealth service verifying genetic markers...");
    let verification_result =
        verify_genetic_proof_complete(&genetic_proof, &wallet_response, &requested_markers).await?;

    if verification_result {
        println!("GENETIC MARKER VERIFICATION SUCCESSFUL");

        // Generate personalized recommendations based ONLY on disclosed markers
        println!("\nPersonalized Health Recommendations:");

        if genetic_proof.markers_present.contains(&"BRCA1".to_string()) {
            println!("   Enhanced cancer screening recommended");
            println!("   Consider genetic counseling");
        }

        if genetic_proof.markers_present.contains(&"MTHFR".to_string()) {
            println!("   Methylated folate supplementation recommended");
            println!("   Leafy green vegetables especially beneficial");
        }

        println!("\nSession data will auto-expire in 10 minutes");
    } else {
        println!("Genetic marker verification failed");
        return Ok(());
    }

    // Demonstrate privacy preservation
    println!("\nPrivacy Analysis:");
    println!("   Health service knows:");
    for marker in &genetic_proof.markers_present {
        println!("      - {} variant: PRESENT", marker);
    }

    println!("   Health service does NOT know:");
    println!("      - Other genetic variants");
    println!("      - Full genome sequence");
    println!("      - Family genetic history");
    println!("      - Variants not requested");

    // Show what markers were NOT disclosed
    let undisclosed: Vec<_> = user_genetic_markers
        .iter()
        .filter(|marker| !requested_markers.contains(marker))
        .collect();

    if !undisclosed.is_empty() {
        println!("      - Undisclosed markers: {:?}", undisclosed);
    }

    println!("\nDemonstrating ephemeral relay:");
    println!("   - Genetic data request stored temporarily");
    println!("   - Auto-expires after successful verification");
    println!("   - No permanent genetic database created");

    // Simulate Google Wallet integration for genomics
    println!("\nGoogle Wallet Integration:");
    let google_config =
        wallet_integration.generate_google_wallet_integration("did:proofzk:genetics_user456")?;
    println!("   {}", serde_json::to_string_pretty(&google_config)?);

    println!("\nDemo complete! Genetic privacy preserved, personalized recommendations delivered.");

    Ok(())
}
