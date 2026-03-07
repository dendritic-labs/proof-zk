use serde::{Deserialize, Serialize};
use crate::{did::DidIdentity, Result};
use std::fs;
use rand::RngCore;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedDidStorage {
    pub encrypted_did: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct LocalDidStore {
    storage_path: std::path::PathBuf,
}

impl LocalDidStore {
    pub fn new(app_data_dir: &str) -> Result<Self> {
        let storage_path = std::path::PathBuf::from(app_data_dir).join("proofzk_dids");
        fs::create_dir_all(&storage_path)?;
        
        Ok(Self { storage_path })
    }

    /// Store DID encrypted with user password/biometric
    pub fn store_did(&self, did: &DidIdentity, password: &str) -> Result<String> {
        let did_json = serde_json::to_string(did)?;
        let key = Self::derive_key_from_password(password);
        
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        
        // Simple XOR encryption (in production, use proper AES-GCM)
        let encrypted_did = Self::xor_encrypt(did_json.as_bytes(), &key, &nonce_bytes);
        
        let storage = EncryptedDidStorage {
            encrypted_did,
            nonce: nonce_bytes.to_vec(),
            created_at: chrono::Utc::now(),
        };
        
        // Save to file
        let did_id = Self::generate_did_id(&did.did);
        let file_path = self.storage_path.join(format!("{}.json", did_id));
        let storage_json = serde_json::to_string_pretty(&storage)?;
        fs::write(&file_path, storage_json)?;
        
        Ok(did_id)
    }

    /// Load DID with user password/biometric
    pub fn load_did(&self, did_id: &str, password: &str) -> Result<DidIdentity> {
        let file_path = self.storage_path.join(format!("{}.json", did_id));
        let storage_json = fs::read_to_string(file_path)?;
        let storage: EncryptedDidStorage = serde_json::from_str(&storage_json)?;
        
        let key = Self::derive_key_from_password(password);
        
        let decrypted_bytes = Self::xor_decrypt(&storage.encrypted_did, &key, &storage.nonce);
        let did_json = String::from_utf8(decrypted_bytes)?;
        let did: DidIdentity = serde_json::from_str(&did_json)?;
        
        Ok(did)
    }

    /// List all stored DID IDs
    pub fn list_stored_dids(&self) -> Result<Vec<String>> {
        let mut dids = Vec::new();
        
        for entry in fs::read_dir(&self.storage_path)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    let did_id = name.strip_suffix(".json").unwrap().to_string();
                    dids.push(did_id);
                }
            }
        }
        
        Ok(dids)
    }

    /// Export DID as QR code data for mobile scanning
    pub fn export_as_qr_code(&self, did_id: &str) -> Result<String> {
        let file_path = self.storage_path.join(format!("{}.json", did_id));
        let storage_json = fs::read_to_string(file_path)?;
        
        // Create QR-friendly format
        let qr_data = serde_json::json!({
            "type": "proofzk_did",
            "version": "1.0",
            "data": storage_json,
            "app": "proofzk://import"
        });
        
        Ok(qr_data.to_string())
    }

    /// Import DID from QR code scan
    pub fn import_from_qr_code(&self, qr_data: &str, password: &str) -> Result<String> {
        let qr_json: serde_json::Value = serde_json::from_str(qr_data)?;
        let storage_json = qr_json["data"].as_str().ok_or("Invalid QR code format")?;
        
        let storage: EncryptedDidStorage = serde_json::from_str(storage_json)?;
        
        // Decrypt to verify it works with password
        let key = Self::derive_key_from_password(password);
        let decrypted_bytes = Self::xor_decrypt(&storage.encrypted_did, &key, &storage.nonce);
        let did_json = String::from_utf8(decrypted_bytes)?;
        let did: DidIdentity = serde_json::from_str(&did_json)?;
        
        // Store locally
        let did_id = Self::generate_did_id(&did.did);
        let file_path = self.storage_path.join(format!("{}.json", did_id));
        fs::write(&file_path, storage_json)?;
        
        Ok(did_id)
    }

    // Helper methods
    fn derive_key_from_password(password: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(b"proofzk_salt_v1"); // Simple salt
        let hash = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash);
        key
    }

    fn generate_did_id(did: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(did.as_bytes());
        let hash = hasher.finalize();
        hex::encode(&hash[0..8]) // First 8 bytes as hex
    }

    fn xor_encrypt(data: &[u8], key: &[u8; 32], nonce: &[u8]) -> Vec<u8> {
        // Simple XOR encryption with key derivation from nonce
        let mut key_stream = Vec::new();
        for i in 0..data.len() {
            let key_byte = key[i % 32] ^ nonce[i % nonce.len()];
            key_stream.push(key_byte);
        }
        
        data.iter().zip(key_stream.iter()).map(|(a, b)| a ^ b).collect()
    }

    fn xor_decrypt(data: &[u8], key: &[u8; 32], nonce: &[u8]) -> Vec<u8> {
        // XOR decryption is same as encryption
        Self::xor_encrypt(data, key, nonce)
    }
}

/// Browser-compatible storage using WASM
#[cfg(target_arch = "wasm32")]
pub mod browser_storage {
    use super::*;
    use wasm_bindgen::prelude::*;
    
    pub struct BrowserDidStore {
        db_name: String,
    }
    
    impl BrowserDidStore {
        pub fn new() -> Self {
            Self {
                db_name: "proofzk_dids".to_string(),
            }
        }
        
        /// Store DID in browser's IndexedDB (encrypted)
        pub async fn store_did_encrypted(&self, did: &DidIdentity, password: &str) -> Result<String> {
            // Implementation for browser IndexedDB storage
            // Uses WebCrypto API for encryption
            let did_id = format!("browser_{}", uuid::Uuid::new_v4());
            // TODO: Implement actual browser storage
            Ok(did_id)
        }
        
        /// Load DID from browser storage
        pub async fn load_did_encrypted(&self, did_id: &str, password: &str) -> Result<DidIdentity> {
            // TODO: Implement browser IndexedDB loading
            Err("Browser storage not yet implemented".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::did::DidIdentity;

    #[test]
    fn test_local_did_storage() {
        let temp_dir = std::env::temp_dir().join("proofzk_test");
        let store = LocalDidStore::new(temp_dir.to_str().unwrap()).unwrap();
        
        // Create test DID
        let did = DidIdentity::new().unwrap();
        let password = "test_password_123";
        
        // Store DID
        let did_id = store.store_did(&did, password).unwrap();
        assert!(!did_id.is_empty());
        
        // Load DID back
        let loaded_did = store.load_did(&did_id, password).unwrap();
        assert_eq!(did.did, loaded_did.did);
        
        // Test QR export/import
        let qr_data = store.export_as_qr_code(&did_id).unwrap();
        assert!(qr_data.contains("proofzk_did"));
        
        // Test wrong password fails
        assert!(store.load_did(&did_id, "wrong_password").is_err());
        
        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }
}