use proofzk_core::*;
use proofzk_core::did::*;
use proofzk_core::proofs::*;
use proofzk_core::wallet::*;

/// Test ProofZK with real DID methods
#[tokio::main]
async fn main() -> Result<()> {
    println!("🆔 ProofZK Real DID Testing Suite");
    println!("==================================\n");

    // Test 1: DID:key (cryptographically derived)
    println!("🔑 Testing DID:key (cryptographically derived)");
    test_did_key().await?;
    
    // Test 2: DID:web (web-hosted)
    println!("\n🌐 Testing DID:web (web-hosted)");
    test_did_web().await?;
    
    // Test 3: Universal DID resolver
    println!("\n🔍 Testing Universal DID Resolver");
    test_universal_resolver().await?;
    
    // Test 4: Real DID integration with ProofZK
    println!("\n✅ Testing ProofZK with Real DIDs");
    test_proofzk_with_real_dids().await?;
    
    println!("\n🎉 All DID tests completed successfully!");
    Ok(())
}

async fn test_did_key() -> Result<()> {
    // Create a real did:key identity
    let identity = DidIdentity::new_did_key()?;
    println!("  ✅ Created DID:key: {}", identity.did);
    
    // Test signing and verification
    let message = b"Hello ProofZK with real DIDs!";
    let signature = identity.sign_message(message)?;
    let verified = identity.verify_signature(message, &signature)?;
    
    println!("  ✅ Message signed and verified: {}", verified);
    
    // Show the DID document
    let doc = identity.get_public_document();
    println!("  📄 DID Document public keys: {}", doc.public_keys.len());
    
    Ok(())
}

async fn test_did_web() -> Result<()> {
    // Create a did:web identity
    let identity = DidIdentity::new_did_web("example.com", Some("users/alice"))?;
    println!("  ✅ Created DID:web: {}", identity.did);
    
    // Export the DID document for web hosting
    let did_json = identity.export_did_web_document()?;
    println!("  📄 Exportable DID document created ({} bytes)", did_json.len());
    
    // Show where it should be hosted
    println!("  🌐 Should be hosted at: https://example.com/users/alice/.well-known/did.json");
    
    Ok(())
}

async fn test_universal_resolver() -> Result<()> {
    let mut resolver = UniversalDidResolver::new();
    
    // Test resolving a real did:key
    let key_identity = DidIdentity::new_did_key()?;
    let key_did = &key_identity.did;
    
    // Register it locally first
    resolver.register(key_identity.get_public_document())?;
    
    // Resolve from local registry
    if let Some(resolved_doc) = resolver.resolve(key_did).await? {
        println!("  ✅ Resolved DID from local registry: {}", resolved_doc.id);
    }
    
    // Test resolving a real did:key cryptographically
    let test_did = "did:key:zDnaerDaTF5BXEavCrfRZEk316dpbLsfPDZ3WJ5hRTPFU2169";
    if let Some(resolved_doc) = resolver.resolve(test_did).await? {
        println!("  ✅ Resolved real DID:key cryptographically: {}", resolved_doc.id);
    } else {
        println!("  ⚠️  Could not resolve DID:key (this is expected for invalid keys)");
    }
    
    // Test resolving Microsoft ION DID (if available)
    let ion_did = "did:ion:EiDyOQbbZAa3aiRzeCkV7LOx3SERjjH93EXoIM3UoN4oWg";
    match resolver.resolve(ion_did).await {
        Ok(Some(doc)) => println!("  ✅ Resolved Microsoft ION DID: {}", doc.id),
        Ok(None) => println!("  ⚠️  ION DID not found (network dependent)"),
        Err(_) => println!("  ⚠️  ION resolution failed (network dependent)"),
    }
    
    Ok(())
}

async fn test_proofzk_with_real_dids() -> Result<()> {
    // Create real DID identities for different actors
    let airline_identity = DidIdentity::new_did_key()?;
    let passenger_identity = DidIdentity::new_did_web("passenger.example.com", None)?;
    let genomics_identity = DidIdentity::new_did_key()?;
    
    println!("  🏢 Airline DID: {}", airline_identity.did);
    println!("  👤 Passenger DID: {}", passenger_identity.did);
    println!("  🧬 Genomics Service DID: {}", genomics_identity.did);
    
    // Create wallet with real DID
    let mut wallet_integration = WalletIntegration::new();
    let wallet_credential = WalletCredential {
        id: "real_wallet_001".to_string(),
        wallet_type: WalletProvider::Apple,
        did: passenger_identity.did.clone(),
        public_key: passenger_identity.document.public_keys[0].public_key_base58.clone(),
        metadata: {
            let mut meta = std::collections::HashMap::new();
            meta.insert("device".to_string(), "iPhone 15".to_string());
            meta.insert("wallet_version".to_string(), "iOS 17.0".to_string());
            meta
        },
    };
    
    wallet_integration.register_wallet_credential(wallet_credential).await?;
    println!("  ✅ Registered wallet with real DID");
    
    // Test age verification with real DIDs
    let (age_proof, wallet_response) = create_age_verification_proof(
        25, // User age
        18, // Min age
        &wallet_integration,
    ).await?;
    
    // Create verifiable presentation using real DIDs
    let proof_presentation = serde_json::json!({
        "@context": ["https://www.w3.org/2018/credentials/v1"],
        "type": ["VerifiablePresentation"],
        "holder": passenger_identity.did,
        "verifier": airline_identity.did,
        "proof": {
            "type": "ProofZKAgeVerification",
            "created": chrono::Utc::now(),
            "proofPurpose": "age_verification",
            "verificationMethod": format!("{}#key-1", passenger_identity.did),
            "zkProof": age_proof,
            "walletResponse": wallet_response
        }
    });
    
    println!("  ✅ Created verifiable presentation with real DIDs");
    println!("  📄 Presentation size: {} bytes", proof_presentation.to_string().len());
    
    // Verify the proof with DID verification
    let message = proof_presentation.to_string();
    let signature = passenger_identity.sign_message(message.as_bytes())?;
    let verified = passenger_identity.verify_signature(message.as_bytes(), &signature)?;
    
    println!("  ✅ DID signature verification: {}", verified);
    
    // Show the complete DID-based verification flow
    println!("\n  🔄 Complete DID Verification Flow:");
    println!("     1. Passenger creates DID:web identity");
    println!("     2. Airline creates DID:key identity");
    println!("     3. ProofZK generates ZK proof with DID signatures");
    println!("     4. Verifiable presentation links all identities");
    println!("     5. Cryptographic verification confirms authenticity");
    
    Ok(())
}

/// Helper function to test with external DID resolvers
pub async fn test_external_did_resolution() -> Result<()> {
    println!("\n🌍 Testing External DID Resolution");
    
    let resolver = UniversalDidResolver::new();
    
    // Test known public DIDs (these should resolve if network is available)
    let test_dids = vec![
        "did:web:did.actor:alice",  // Example DID:web
        "did:ion:EiDyOQbbZAa3aiRzeCkV7LOx3SERjjH93EXoIM3UoN4oWg", // Microsoft ION
    ];
    
    for did in test_dids {
        match resolver.resolve(did).await {
            Ok(Some(doc)) => {
                println!("  ✅ Resolved {}: {} keys", did, doc.public_keys.len());
            }
            Ok(None) => {
                println!("  ⚠️  Could not resolve {}", did);
            }
            Err(e) => {
                println!("  ❌ Error resolving {}: {}", did, e);
            }
        }
    }
    
    Ok(())
}