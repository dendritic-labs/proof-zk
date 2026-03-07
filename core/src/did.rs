use serde::{Deserialize, Serialize};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::{DidDocument, PublicKeyEntry, ServiceEndpoint, Result};
use std::collections::HashMap;
use rand::rngs::OsRng;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DidMethod {
    Key,      // did:key - cryptographic keys
    Web,      // did:web - web-based DIDs
    Ion,      // did:ion - Bitcoin-anchored
    Ethr,     // did:ethr - Ethereum-based
    ProofZK,  // did:proofzk - our custom method
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DidIdentity {
    pub did: String,
    pub method: DidMethod,
    #[serde(skip)] // Don't serialize private keys
    pub keypair: Option<Keypair>,
    pub document: DidDocument,
}

impl Clone for DidIdentity {
    fn clone(&self) -> Self {
        Self {
            did: self.did.clone(),
            method: self.method.clone(),
            keypair: None, // Don't clone private keys for security
            document: self.document.clone(),
        }
    }
}

impl DidIdentity {
    /// Create a new DID:key identity (cryptographically derived)
    pub fn new_did_key() -> Result<Self> {
        let keypair = Keypair::generate(&mut OsRng);
        let public_key_bytes = keypair.public.to_bytes();
        
        // Create DID:key according to spec
        let multicodec_prefix = [0xed, 0x01]; // ed25519-pub multicodec
        let mut key_bytes = Vec::new();
        key_bytes.extend_from_slice(&multicodec_prefix);
        key_bytes.extend_from_slice(&public_key_bytes);
        
        let did = format!("did:key:z{}", bs58::encode(&key_bytes).into_string());
        
        let document = DidDocument {
            id: did.clone(),
            public_keys: vec![PublicKeyEntry {
                id: format!("{}#key-1", did),
                key_type: "Ed25519VerificationKey2020".to_string(),
                public_key_base58: bs58::encode(&public_key_bytes).into_string(),
            }],
            services: vec![ServiceEndpoint {
                id: format!("{}#proofzk-relay", did),
                service_type: "ProofZKRelay".to_string(),
                endpoint: "http://localhost:4000/api/v1".to_string(),
            }],
            created: Utc::now(),
            updated: Utc::now(),
        };

        Ok(DidIdentity {
            did,
            method: DidMethod::Key,
            keypair: Some(keypair),
            document,
        })
    }

    /// Create a DID:web identity (web-hosted)
    pub fn new_did_web(domain: &str, path: Option<&str>) -> Result<Self> {
        let keypair = Keypair::generate(&mut OsRng);
        let public_key_bytes = keypair.public.to_bytes();
        
        let did = match path {
            Some(p) => format!("did:web:{}:{}", domain, p.replace('/', ":")),
            None => format!("did:web:{}", domain),
        };
        
        let document = DidDocument {
            id: did.clone(),
            public_keys: vec![PublicKeyEntry {
                id: format!("{}#key-1", did),
                key_type: "Ed25519VerificationKey2020".to_string(),
                public_key_base58: bs58::encode(&public_key_bytes).into_string(),
            }],
            services: vec![
                ServiceEndpoint {
                    id: format!("{}#proofzk-relay", did),
                    service_type: "ProofZKRelay".to_string(),
                    endpoint: "http://localhost:4000/api/v1".to_string(),
                },
                ServiceEndpoint {
                    id: format!("{}#did-web-endpoint", did),
                    service_type: "DIDWebEndpoint".to_string(),
                    endpoint: format!("https://{}/.well-known/did.json", domain),
                }
            ],
            created: Utc::now(),
            updated: Utc::now(),
        };

        Ok(DidIdentity {
            did,
            method: DidMethod::Web,
            keypair: Some(keypair),
            document,
        })
    }

    /// Create our custom ProofZK DID (for backwards compatibility)
    pub fn new() -> Result<Self> {
        let keypair = Keypair::generate(&mut OsRng);
        let public_key = BASE64.encode(keypair.public.to_bytes());
        let did = format!("did:proofzk:{}", Uuid::new_v4());
        
        let document = DidDocument {
            id: did.clone(),
            public_keys: vec![PublicKeyEntry {
                id: format!("{}#key-1", did),
                key_type: "Ed25519VerificationKey2020".to_string(),
                public_key_base58: public_key,
            }],
            services: vec![ServiceEndpoint {
                id: format!("{}#relay", did),
                service_type: "ProofZKRelay".to_string(),
                endpoint: "http://localhost:4000/relay".to_string(),
            }],
            created: Utc::now(),
            updated: Utc::now(),
        };

        Ok(DidIdentity {
            did,
            method: DidMethod::ProofZK,
            keypair: Some(keypair),
            document,
        })
    }

    pub fn from_document(document: DidDocument) -> Self {
        let method = Self::parse_did_method(&document.id);
        
        Self {
            did: document.id.clone(),
            method,
            keypair: None, // External DID, no private key
            document,
        }
    }

    pub fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>> {
        match &self.keypair {
            Some(kp) => {
                let signature = kp.sign(message);
                Ok(signature.to_bytes().to_vec())
            }
            None => Err("No private key available for signing".into()),
        }
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> Result<bool> {
        if let Some(pk_entry) = self.document.public_keys.first() {
            let pk_bytes = match self.method {
                DidMethod::Key => {
                    // For did:key, decode base58
                    bs58::decode(&pk_entry.public_key_base58).into_vec()?
                }
                _ => {
                    // For others, decode base64 (legacy)
                    BASE64.decode(&pk_entry.public_key_base58)?
                }
            };
            
            let public_key = PublicKey::from_bytes(&pk_bytes[pk_bytes.len().saturating_sub(32)..])?; // Last 32 bytes for ed25519
            let sig = Signature::from_bytes(signature)?;
            
            Ok(public_key.verify(message, &sig).is_ok())
        } else {
            Err("No public key found in DID document".into())
        }
    }

    pub fn get_public_document(&self) -> DidDocument {
        self.document.clone()
    }

    /// Export DID document for web hosting (did:web)
    pub fn export_did_web_document(&self) -> Result<String> {
        if !matches!(self.method, DidMethod::Web) {
            return Err("Not a did:web identity".into());
        }
        
        Ok(serde_json::to_string_pretty(&self.document)?)
    }

    fn parse_did_method(did: &str) -> DidMethod {
        if did.starts_with("did:key:") {
            DidMethod::Key
        } else if did.starts_with("did:web:") {
            DidMethod::Web
        } else if did.starts_with("did:ion:") {
            DidMethod::Ion
        } else if did.starts_with("did:ethr:") {
            DidMethod::Ethr
        } else if did.starts_with("did:proofzk:") {
            DidMethod::ProofZK
        } else {
            DidMethod::ProofZK // Default fallback
        }
    }
}

#[derive(Debug, Clone)]
pub struct UniversalDidResolver {
    local_registry: HashMap<String, DidDocument>,
    http_client: reqwest::Client,
}

impl UniversalDidResolver {
    pub fn new() -> Self {
        Self {
            local_registry: HashMap::new(),
            http_client: reqwest::Client::new(),
        }
    }

    /// Register a DID document locally
    pub fn register(&mut self, did_document: DidDocument) -> Result<()> {
        self.local_registry.insert(did_document.id.clone(), did_document);
        Ok(())
    }

    /// Universal DID resolver supporting multiple methods
    pub async fn resolve(&self, did: &str) -> Result<Option<DidDocument>> {
        // First check local registry
        if let Some(doc) = self.local_registry.get(did) {
            return Ok(Some(doc.clone()));
        }

        // Parse DID method and resolve accordingly
        if did.starts_with("did:key:") {
            self.resolve_did_key(did).await
        } else if did.starts_with("did:web:") {
            self.resolve_did_web(did).await
        } else if did.starts_with("did:ion:") {
            self.resolve_did_ion(did).await
        } else if did.starts_with("did:ethr:") {
            self.resolve_did_ethr(did).await
        } else {
            Ok(None) // Unknown method
        }
    }

    /// Resolve did:key (cryptographically derived)
    async fn resolve_did_key(&self, did: &str) -> Result<Option<DidDocument>> {
        // Extract key from DID
        let key_part = did.strip_prefix("did:key:z").ok_or("Invalid did:key format")?;
        let key_bytes = bs58::decode(key_part).into_vec()?;
        
        // Skip multicodec prefix (first 2 bytes for ed25519)
        let public_key_bytes = &key_bytes[2..];
        
        let document = DidDocument {
            id: did.to_string(),
            public_keys: vec![PublicKeyEntry {
                id: format!("{}#key-1", did),
                key_type: "Ed25519VerificationKey2020".to_string(),
                public_key_base58: bs58::encode(public_key_bytes).into_string(),
            }],
            services: vec![], // did:key has no services by default
            created: Utc::now(),
            updated: Utc::now(),
        };

        Ok(Some(document))
    }

    /// Resolve did:web (web-hosted)
    async fn resolve_did_web(&self, did: &str) -> Result<Option<DidDocument>> {
        // Convert DID to URL
        let domain_path = did.strip_prefix("did:web:").ok_or("Invalid did:web format")?;
        let url_path = domain_path.replace(':', "/");
        let url = format!("https://{}/.well-known/did.json", url_path);

        // Fetch DID document from web
        match self.http_client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                let doc: DidDocument = response.json().await?;
                Ok(Some(doc))
            }
            _ => Ok(None), // Failed to resolve
        }
    }

    /// Resolve did:ion (Microsoft ION)
    async fn resolve_did_ion(&self, did: &str) -> Result<Option<DidDocument>> {
        // Use Microsoft's ION resolver
        let url = format!("https://beta.discover.did.microsoft.com/1.0/identifiers/{}", did);
        
        match self.http_client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                let doc: DidDocument = response.json().await?;
                Ok(Some(doc))
            }
            _ => Ok(None),
        }
    }

    /// Resolve did:ethr (Ethereum-based)
    async fn resolve_did_ethr(&self, did: &str) -> Result<Option<DidDocument>> {
        // Use uPort's universal resolver
        let url = format!("https://dev.uniresolver.io/1.0/identifiers/{}", did);
        
        match self.http_client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                let resolver_response: serde_json::Value = response.json().await?;
                if let Some(doc) = resolver_response.get("didDocument") {
                    let did_doc: DidDocument = serde_json::from_value(doc.clone())?;
                    Ok(Some(did_doc))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}

impl Default for UniversalDidResolver {
    fn default() -> Self {
        Self::new()
    }
}