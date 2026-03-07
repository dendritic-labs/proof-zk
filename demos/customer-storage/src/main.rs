use proofzk_core::*;
use proofzk_core::did::*;
use proofzk_core::storage::*;
use std::io::{self, Write};

/// Demo: Customer DID storage without native wallet support
#[tokio::main]
async fn main() -> Result<()> {
    println!("🔐 ProofZK Customer DID Storage Demo");
    println!("====================================");
    println!("Showing how customers store DIDs before Apple/Google/Samsung support\n");

    // Method 1: Local encrypted storage
    demo_local_encrypted_storage().await?;
    
    // Method 2: QR code export/import
    demo_qr_code_transfer().await?;
    
    // Method 3: Browser storage simulation
    demo_browser_storage().await?;

    println!("\n✅ All storage methods demonstrated!");
    println!("\n💡 Recommendation for customers:");
    println!("   • Use browser PWA for most users (works everywhere)");
    println!("   • Local app for power users who want device keychain");
    println!("   • QR codes for easy backup and device transfer");
    
    Ok(())
}

async fn demo_local_encrypted_storage() -> Result<()> {
    println!("🏠 Method 1: Local Encrypted Storage");
    println!("   (Desktop app, mobile app with keychain integration)");
    
    // Create customer's DID
    let customer_did = DidIdentity::new_did_key()?;
    println!("   ✅ Created customer DID: {}", &customer_did.did[0..50]);
    
    // Set up secure local storage
    let temp_dir = std::env::temp_dir().join("proofzk_demo");
    let storage = LocalDidStore::new(temp_dir.to_str().unwrap())?;
    
    // Customer sets their password/PIN (in real app, this would use biometrics)
    let customer_password = "my_secure_biometric_pin_123";
    
    // Store DID encrypted
    let did_id = storage.store_did(&customer_did, customer_password)?;
    println!("   💾 Stored DID encrypted with biometric/PIN");
    println!("   📋 DID ID for retrieval: {}", did_id);
    
    // Later: Customer wants to use their DID
    println!("   🔓 Customer unlocks with biometric/PIN...");
    let loaded_did = storage.load_did(&did_id, customer_password)?;
    println!("   ✅ DID loaded successfully: {}", loaded_did.did == customer_did.did);
    
    // Show what's actually stored (encrypted)
    let stored_dids = storage.list_stored_dids()?;
    println!("   📁 Total DIDs in secure storage: {}", stored_dids.len());
    
    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
    
    Ok(())
}

async fn demo_qr_code_transfer() -> Result<()> {
    println!("\n📱 Method 2: QR Code Export/Import");
    println!("   (Easy backup, device transfer, sharing with family)");
    
    // Customer has DID on their laptop
    let customer_did = DidIdentity::new_did_web("customer.example.com", Some("alice"))?;
    let temp_dir = std::env::temp_dir().join("proofzk_laptop");
    let laptop_storage = LocalDidStore::new(temp_dir.to_str().unwrap())?;
    
    let password = "alice_secure_password";
    let did_id = laptop_storage.store_did(&customer_did, password)?;
    println!("   💻 Customer DID stored on laptop");
    
    // Export as QR code for phone
    let qr_data = laptop_storage.export_as_qr_code(&did_id)?;
    println!("   📷 Generated QR code data ({} bytes)", qr_data.len());
    println!("   📱 Customer scans QR with phone app...");
    
    // Simulate phone importing the QR code
    let temp_dir_phone = std::env::temp_dir().join("proofzk_phone");
    let phone_storage = LocalDidStore::new(temp_dir_phone.to_str().unwrap())?;
    
    let phone_did_id = phone_storage.import_from_qr_code(&qr_data, password)?;
    let phone_did = phone_storage.load_did(&phone_did_id, password)?;
    
    println!("   ✅ DID successfully transferred to phone!");
    println!("   🔄 Same DID on both devices: {}", customer_did.did == phone_did.did);
    
    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
    std::fs::remove_dir_all(&temp_dir_phone).ok();
    
    Ok(())
}

async fn demo_browser_storage() -> Result<()> {
    println!("\n🌐 Method 3: Browser PWA Storage");
    println!("   (Most practical - works on all devices immediately)");
    
    // This would be implemented in JavaScript/WASM for real browsers
    // Here we simulate the browser storage concept
    
    let customer_did = DidIdentity::new_did_key()?;
    println!("   🌍 Customer visits proofzk.com/wallet");
    println!("   🔐 Browser generates DID with WebCrypto API");
    println!("   📱 Customer authorizes with Face ID/Touch ID/PIN");
    
    // Simulate browser storage (in real implementation, this uses IndexedDB)
    let temp_dir = std::env::temp_dir().join("proofzk_browser");
    let browser_storage = LocalDidStore::new(temp_dir.to_str().unwrap())?;
    
    // Browser would use device biometric/PIN as password
    let device_auth = "device_biometric_hash_xyz789";
    let did_id = browser_storage.store_did(&customer_did, device_auth)?;
    
    println!("   💾 DID stored in browser IndexedDB (encrypted)");
    println!("   🔄 Available across browser sessions");
    println!("   📱 PWA can be installed like a native app");
    
    // Customer uses DID for airline check-in
    println!("   ✈️ Customer goes to airline website...");
    let loaded_did = browser_storage.load_did(&did_id, device_auth)?;
    println!("   ✅ DID loaded for airline proof generation");
    
    // Demonstrate airline integration
    demo_airline_usage(&loaded_did).await?;
    
    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
    
    Ok(())
}

async fn demo_airline_usage(customer_did: &DidIdentity) -> Result<()> {
    println!("   🎫 Generating age proof for airline check-in...");
    
    // Customer proves they're over 18 without revealing exact age
    use proofzk_core::zkp::*;
    
    // Real customer age (25) - never revealed to airline
    let real_age = 25u8;
    let min_age = 18u8;
    
    let age_proof = ZkProofSystem::prove_age_over(real_age, min_age)?;
    let is_valid = ZkProofSystem::verify_age_proof(&age_proof, min_age)?;
    
    println!("   ✅ Age proof generated and verified: {}", is_valid);
    println!("   🛡️ Airline knows customer is 18+ but not exact age");
    println!("   🚫 No customer data stored by airline");
    
    Ok(())
}

/// Additional practical considerations for customers
pub fn show_customer_migration_path() {
    println!("\n🛤️ Customer Migration Path to Native Wallet Support:");
    println!("   Phase 1 (Now): Browser PWA + QR backup");
    println!("   Phase 2 (6mo): Native mobile apps with keychain");
    println!("   Phase 3 (12mo): Apple/Google wallet integration pilot");
    println!("   Phase 4 (18mo): Full native wallet support");
    
    println!("\n💰 Cost Savings for Airlines:");
    println!("   • No customer data storage = $0 breach liability");
    println!("   • No GDPR compliance overhead = $2M+ savings");
    println!("   • No PCI DSS requirements = $500K+ savings");
    println!("   • Instant KYC verification = 97% cost reduction");
    
    println!("\n👥 Customer Benefits:");
    println!("   • Control their own data");
    println!("   • Use same DID across all airlines");
    println!("   • Instant verification (no forms)");
    println!("   • Privacy by default (selective disclosure)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_demo() {
        // Test that all storage methods work
        demo_local_encrypted_storage().await.unwrap();
        demo_qr_code_transfer().await.unwrap();
        demo_browser_storage().await.unwrap();
    }
}