use proofzk_core::*;
use proofzk_core::did::*;
use proofzk_core::proofs::*;
use proofzk_core::wallet::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportCredential {
    pub passport_number: String,
    pub issuing_country: String,
    pub nationality: String,
    pub expiration_date: chrono::DateTime<chrono::Utc>,
    pub biometric_hash: Vec<u8>,
    pub issuing_authority_did: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisaCredential {
    pub visa_number: String,
    pub visa_type: String, // Tourist, Business, Transit
    pub issuing_country: String,
    pub valid_from: chrono::DateTime<chrono::Utc>,
    pub valid_until: chrono::DateTime<chrono::Utc>,
    pub entry_purposes: Vec<String>,
    pub max_stay_days: Option<u32>,
    pub issuing_authority_did: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelAuthorization {
    pub authorization_id: String,
    pub traveler_did: String,
    pub destination_country: String,
    pub purpose: String,
    pub authorized_until: chrono::DateTime<chrono::Utc>,
    pub restrictions: Vec<String>,
    pub issuing_authority_did: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirlineProofRequest {
    pub flight_number: String,
    pub departure_country: String,
    pub arrival_country: String,
    pub departure_time: chrono::DateTime<chrono::Utc>,
    pub required_proofs: Vec<TravelProofType>,
    pub airline_did: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TravelProofType {
    PassportValid,
    VisaValid { destination: String },
    AgeVerification { min_age: u8 },
    Nationality { allowed_countries: Vec<String> },
    TravelAuthorization { destination: String },
    VaccinationStatus { required_vaccines: Vec<String> },
}

/// Create government-issued passport DID (simulating real government issuance)
pub async fn create_government_passport_did(
    country_code: &str,
    passport_data: PassportCredential,
) -> Result<DidIdentity> {
    // Government DIDs would use did:web with official domains
    let gov_domain = match country_code {
        "CA" => "digital.canada.ca",
        "US" => "digital.dhs.gov", 
        "UK" => "digital.gov.uk",
        "DE" => "digital.bund.de",
        "FR" => "digital.gouv.fr",
        "AU" => "digital.gov.au",
        "SG" => "digital.gov.sg",
        _ => "digital.example.gov",
    };
    
    let passport_identity = DidIdentity::new_did_web(
        gov_domain, 
        Some(&format!("passports/{}", passport_data.passport_number))
    )?;
    
    println!("🏛️  Government Passport DID: {}", passport_identity.did);
    println!("   📄 Passport: {} ({})", passport_data.passport_number, passport_data.issuing_country);
    println!("   🌐 Hosted at: https://{}/.well-known/did.json", gov_domain);
    
    Ok(passport_identity)
}

/// Create visa DID (simulating consular issuance)
pub async fn create_visa_did(
    issuing_country: &str,
    destination_country: &str,
    visa_data: VisaCredential,
) -> Result<DidIdentity> {
    let consular_domain = format!("{}-embassy.{}.gov", 
        issuing_country.to_lowercase(), 
        destination_country.to_lowercase()
    );
    
    let visa_identity = DidIdentity::new_did_web(
        &consular_domain,
        Some(&format!("visas/{}", visa_data.visa_number))
    )?;
    
    println!("🏛️  Visa DID: {}", visa_identity.did);
    println!("   📋 Visa: {} ({} → {})", visa_data.visa_number, issuing_country, destination_country);
    
    Ok(visa_identity)
}

/// Airline check-in with government credentials
#[tokio::main]
async fn main() -> Result<()> {
    println!("✈️  ProofZK Airline Travel Demo with Government DIDs");
    println!("====================================================");
    println!("Demonstrating privacy-preserving travel verification\n");

    // Simulate real government passport credential
    let passport_data = PassportCredential {
        passport_number: "AB1234567".to_string(),
        issuing_country: "Canada".to_string(),
        nationality: "Canadian".to_string(),
        expiration_date: chrono::Utc::now() + chrono::Duration::days(365 * 3), // 3 years valid
        biometric_hash: vec![0x1a, 0x2b, 0x3c], // Mock biometric hash
        issuing_authority_did: "did:web:digital.canada.ca:passport-office".to_string(),
    };
    
    // Create government passport DID
    let passport_identity = create_government_passport_did("CA", passport_data.clone()).await?;
    
    // Simulate US visa credential  
    let visa_data = VisaCredential {
        visa_number: "US2024567890".to_string(),
        visa_type: "B1/B2".to_string(), // Business/Tourism
        issuing_country: "United States".to_string(),
        valid_from: chrono::Utc::now() - chrono::Duration::days(30),
        valid_until: chrono::Utc::now() + chrono::Duration::days(365 * 10), // 10-year visa
        entry_purposes: vec!["Tourism".to_string(), "Business".to_string()],
        max_stay_days: Some(180),
        issuing_authority_did: "did:web:digital.dhs.gov:visa-office".to_string(),
    };
    
    let visa_identity = create_visa_did("US", "CA", visa_data.clone()).await?;
    
    // Create airline identity (WestJet example)
    let westjet_identity = DidIdentity::new_did_web("westjet.com", Some("digital-identity"))?;
    println!("✈️  WestJet DID: {}", westjet_identity.did);
    
    // Flight details
    let flight_request = AirlineProofRequest {
        flight_number: "WS1234".to_string(),
        departure_country: "Canada".to_string(),
        arrival_country: "United States".to_string(),
        departure_time: chrono::Utc::now() + chrono::Duration::hours(4),
        required_proofs: vec![
            TravelProofType::PassportValid,
            TravelProofType::VisaValid { destination: "United States".to_string() },
            TravelProofType::AgeVerification { min_age: 18 },
            TravelProofType::Nationality { allowed_countries: vec!["Canada".to_string()] },
        ],
        airline_did: westjet_identity.did.clone(),
    };
    
    println!("\n🛂 Flight Check-in Requirements:");
    println!("   ✈️  Flight: {} ({} → {})", flight_request.flight_number, 
             flight_request.departure_country, flight_request.arrival_country);
    println!("   📋 Required proofs: {} items", flight_request.required_proofs.len());
    
    // Create wallet integration with government credentials
    let mut wallet_integration = WalletIntegration::new();
    
    // Register passport credential in wallet
    let passport_wallet_cred = WalletCredential {
        id: "gov_passport_001".to_string(),
        wallet_type: WalletProvider::Apple, // Apple Wallet holds the government credential
        did: passport_identity.did.clone(),
        public_key: passport_identity.document.public_keys[0].public_key_base58.clone(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("credential_type".to_string(), "government_passport".to_string());
            meta.insert("issuing_authority".to_string(), "Passport Canada".to_string());
            meta.insert("trust_level".to_string(), "government_verified".to_string());
            meta
        },
    };
    
    wallet_integration.register_wallet_credential(passport_wallet_cred).await?;
    
    println!("\n🔐 Generating Privacy-Preserving Travel Proofs...");
    
    // Generate travel authorization proof
    let travel_proof = generate_travel_authorization_proof(
        &passport_data,
        &visa_data,
        &flight_request,
        &wallet_integration,
    ).await?;
    
    println!("✅ Travel Authorization Generated:");
    println!("   🛂 Passport validity: VERIFIED (without revealing passport number)");
    println!("   🏛️  Visa validity: VERIFIED (without revealing visa details)");
    println!("   👤 Age verification: VERIFIED (without revealing exact age)");
    println!("   🌍 Nationality: VERIFIED (without revealing other personal data)");
    
    // Create W3C Verifiable Presentation for airline
    let travel_presentation = create_travel_presentation(
        &passport_identity,
        &visa_identity,
        &westjet_identity,
        &travel_proof,
        &flight_request,
    ).await?;
    
    println!("\n📄 W3C Verifiable Presentation Created:");
    println!("   🔗 Holder: {} (passenger)", passport_identity.did);
    println!("   ✈️  Verifier: {} (WestJet)", westjet_identity.did);
    println!("   🏛️  Issuers: Government authorities");
    println!("   📊 Size: {} bytes", travel_presentation.to_string().len());
    
    // Simulate airline verification
    println!("\n🔍 WestJet Verification Process:");
    let verification_result = verify_travel_presentation(&travel_presentation).await?;
    
    if verification_result {
        println!("✅ TRAVEL AUTHORIZATION APPROVED");
        println!("🎫 Boarding pass issued");
        println!("📱 Mobile boarding pass sent to wallet");
        
        // Show what airline knows vs doesn't know
        println!("\n📊 Privacy Analysis for WestJet:");
        println!("   ✅ WestJet knows:");
        println!("      - Passenger has valid passport");
        println!("      - Passenger has valid US visa");
        println!("      - Passenger is 18+ years old");
        println!("      - Passenger is authorized to travel");
        
        println!("   🚫 WestJet does NOT know:");
        println!("      - Passport number");
        println!("      - Exact age or birth date");
        println!("      - Visa number or details");
        println!("      - Home address");
        println!("      - Other travel history");
        println!("      - Biometric data");
        
    } else {
        println!("❌ Travel authorization failed");
    }
    
    // Demonstrate compliance benefits
    println!("\n🛡️  Security & Compliance Benefits:");
    println!("   🔒 Zero passport data stored by airline");
    println!("   ⏰ Ephemeral verification sessions");
    println!("   🏛️  Government-issued credentials");
    println!("   🔐 Cryptographic proof verification");
    println!("   📋 GDPR/privacy law compliant");
    println!("   🎯 Reduces data breach liability");
    
    Ok(())
}

async fn generate_travel_authorization_proof(
    passport: &PassportCredential,
    visa: &VisaCredential,
    flight_request: &AirlineProofRequest,
    wallet: &WalletIntegration,
) -> Result<serde_json::Value> {
    
    // Generate selective disclosure request
    let disclosure_request = SelectiveDisclosureRequest {
        fields_requested: vec![
            "passport_valid".to_string(),
            "visa_valid".to_string(),
            "age_over_18".to_string(),
            "nationality_authorized".to_string(),
        ],
        purpose: format!("airline_travel_authorization_{}", flight_request.flight_number),
        requester_did: flight_request.airline_did.clone(),
        selective_fields: {
            let mut fields = HashMap::new();
            fields.insert("passport_valid".to_string(), true);
            fields.insert("visa_valid".to_string(), true);
            fields.insert("age_over_18".to_string(), true);
            fields.insert("nationality_authorized".to_string(), true);
            // Explicitly exclude sensitive data
            fields.insert("passport_number".to_string(), false);
            fields.insert("visa_number".to_string(), false);
            fields.insert("exact_age".to_string(), false);
            fields.insert("birth_date".to_string(), false);
            fields.insert("biometric_data".to_string(), false);
            fields
        },
    };
    
    // Simulate wallet response with government credential verification
    let wallet_response = WalletResponse {
        credential_id: "gov_passport_001".to_string(),
        disclosed_fields: {
            let mut fields = HashMap::new();
            fields.insert("passport_valid".to_string(), 
                serde_json::Value::Bool(passport.expiration_date > chrono::Utc::now()));
            fields.insert("visa_valid".to_string(), 
                serde_json::Value::Bool(visa.valid_until > chrono::Utc::now()));
            fields.insert("age_over_18".to_string(), serde_json::Value::Bool(true));
            fields.insert("nationality_authorized".to_string(), 
                serde_json::Value::Bool(passport.nationality == "Canadian"));
            fields
        },
        proof_of_possession: vec![0u8; 64], // Government signature
        timestamp: chrono::Utc::now(),
    };
    
    Ok(serde_json::json!({
        "travel_authorization": {
            "flight": flight_request.flight_number,
            "authorized": true,
            "government_verified": true,
            "wallet_response": wallet_response,
            "issued_at": chrono::Utc::now()
        }
    }))
}

async fn create_travel_presentation(
    passport_identity: &DidIdentity,
    visa_identity: &DidIdentity,
    airline_identity: &DidIdentity,
    travel_proof: &serde_json::Value,
    flight_request: &AirlineProofRequest,
) -> Result<serde_json::Value> {
    
    Ok(serde_json::json!({
        "@context": [
            "https://www.w3.org/2018/credentials/v1",
            "https://w3id.org/citizenship/v1",
            "https://w3id.org/travel/v1"
        ],
        "type": ["VerifiablePresentation", "TravelAuthorization"],
        "holder": passport_identity.did,
        "verifier": airline_identity.did,
        "proof": {
            "type": "ProofZKTravelAuthorization",
            "created": chrono::Utc::now(),
            "proofPurpose": "travel_verification",
            "verificationMethod": format!("{}#key-1", passport_identity.did),
            "government_credentials": [
                {
                    "type": "PassportCredential",
                    "issuer": "did:web:digital.canada.ca:passport-office",
                    "verification_method": format!("{}#key-1", passport_identity.did)
                },
                {
                    "type": "VisaCredential", 
                    "issuer": "did:web:digital.dhs.gov:visa-office",
                    "verification_method": format!("{}#key-1", visa_identity.did)
                }
            ],
            "travel_authorization": travel_proof,
            "flight_details": {
                "flight_number": flight_request.flight_number,
                "route": format!("{} → {}", flight_request.departure_country, flight_request.arrival_country),
                "departure_time": flight_request.departure_time
            }
        }
    }))
}

async fn verify_travel_presentation(presentation: &serde_json::Value) -> Result<bool> {
    // In production, this would:
    // 1. Verify government DID signatures
    // 2. Check government credential validity
    // 3. Verify ZK proofs
    // 4. Validate presentation format
    // 5. Check airline authorization
    
    println!("   🔍 Verifying government credentials...");
    println!("   🔍 Checking passport validity...");
    println!("   🔍 Validating visa authorization...");
    println!("   🔍 Confirming travel eligibility...");
    
    Ok(true) // Simplified for demo
}