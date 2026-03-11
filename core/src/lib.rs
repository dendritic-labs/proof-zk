use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod did;
pub mod proofs;
pub mod storage;
pub mod wallet;
pub mod zkp; // New secure storage module

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofRequest {
    pub id: Uuid,
    pub requester: String,
    pub proof_type: ProofType,
    pub required_claims: Vec<String>,
    pub context: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofType {
    AgeVerification { min_age: u8 },
    GeneticMarker { markers: Vec<String> },
    Identity { fields: Vec<String> },
    Custom { schema: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofResponse {
    pub request_id: Uuid,
    pub proof: ZkProof,
    pub selective_disclosure: HashMap<String, bool>,
    pub verified: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProof {
    pub commitment: Vec<u8>,
    pub challenge: Vec<u8>,
    pub response: Vec<u8>,
    pub public_inputs: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidDocument {
    pub id: String,
    pub public_keys: Vec<PublicKeyEntry>,
    pub services: Vec<ServiceEndpoint>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyEntry {
    pub id: String,
    pub key_type: String,
    pub public_key_base58: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub id: String,
    pub service_type: String,
    pub endpoint: String,
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct ProofZK {
    _did_registry: HashMap<String, DidDocument>,
    active_requests: HashMap<Uuid, ProofRequest>,
}

impl ProofZK {
    pub fn new() -> Self {
        Self {
            _did_registry: HashMap::new(),
            active_requests: HashMap::new(),
        }
    }

    pub async fn create_proof_request(&mut self, request: ProofRequest) -> Result<Uuid> {
        let id = request.id;
        self.active_requests.insert(id, request);
        Ok(id)
    }

    pub async fn verify_proof(&self, response: &ProofResponse) -> Result<bool> {
        // Implementation will use zkp module
        Ok(response.verified)
    }
}

impl Default for ProofZK {
    fn default() -> Self {
        Self::new()
    }
}
