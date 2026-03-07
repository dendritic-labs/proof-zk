# ProofZK API Reference

## Core Rust API

### DID Management

#### `DidIdentity::new() -> Result<DidIdentity>`
Creates a new DID identity with Ed25519 keypair.

```rust
let identity = DidIdentity::new()?;
println!("DID: {}", identity.did);
```

#### `DidIdentity::sign_message(&self, message: &[u8]) -> Result<Vec<u8>>`
Signs a message with the DID's private key.

```rust
let signature = identity.sign_message(b"Hello ProofZK")?;
```

### Zero-Knowledge Proofs

#### `ZkProofSystem::prove_age_over(actual_age: u8, min_age: u8) -> Result<AgeProof>`
Generate age verification proof without revealing actual age.

```rust
let proof = ZkProofSystem::prove_age_over(25, 18)?;
assert!(proof.is_over_age);
```

#### `ZkProofSystem::verify_age_proof(proof: &AgeProof, min_age: u8) -> Result<bool>`
Verify age proof without learning actual age.

```rust
let valid = ZkProofSystem::verify_age_proof(&proof, 18)?;
```

#### `ZkProofSystem::prove_genetic_markers(user_markers: &[String], required: &[String]) -> Result<GeneticMarkerProof>`
Generate genetic marker proof with selective disclosure.

```rust
let user_markers = vec!["BRCA1".to_string(), "APOE4".to_string()];
let required = vec!["BRCA1".to_string()];
let proof = ZkProofSystem::prove_genetic_markers(&user_markers, &required)?;
```

### Wallet Integration

#### `WalletIntegration::new() -> WalletIntegration`
Initialize wallet integration system.

```rust
let wallet = WalletIntegration::new();
```

#### `WalletIntegration::register_wallet_credential(&mut self, credential: WalletCredential) -> Result<()>`
Register a wallet credential for future proof requests.

```rust
let credential = WalletCredential {
    id: "wallet_123".to_string(),
    wallet_type: WalletProvider::Apple,
    did: "did:proofzk:user123".to_string(),
    public_key: "base64_key".to_string(),
    metadata: HashMap::new(),
};
wallet.register_wallet_credential(credential).await?;
```

#### `WalletIntegration::request_selective_disclosure(&self, credential_id: &str, request: SelectiveDisclosureRequest) -> Result<WalletResponse>`
Request selective disclosure from a registered wallet.

```rust
let request = SelectiveDisclosureRequest {
    fields_requested: vec!["age_over_18".to_string()],
    purpose: "airline_checkin".to_string(),
    requester_did: "did:proofzk:airline".to_string(),
    selective_fields: {
        let mut fields = HashMap::new();
        fields.insert("age_over_18".to_string(), true);
        fields.insert("birthdate".to_string(), false);
        fields
    },
};

let response = wallet.request_selective_disclosure("wallet_123", request).await?;
```

## Elixir Relay API

### Session Management

#### `Relay.create_session(data, ttl_seconds \\ 300) -> {:ok, session_id} | {:error, reason}`
Create ephemeral session with auto-expiration.

```elixir
{:ok, session_id} = Relay.create_session(%{
  proof_type: "age_verification",
  requester: "airline_app"
}, 600)
```

#### `Relay.get_session_data(session_id) -> {:ok, data} | {:error, :session_not_found}`
Retrieve and destroy session data.

```elixir
case Relay.get_session_data(session_id) do
  {:ok, data} -> 
    # Session automatically destroyed
    IO.puts("Retrieved: #{inspect(data)}")
  {:error, :session_not_found} ->
    IO.puts("Session expired or not found")
end
```

#### `Relay.session_exists?(session_id) -> boolean()`
Check if session exists without consuming it.

```elixir
if Relay.session_exists?(session_id) do
  IO.puts("Session is active")
end
```

#### `Relay.destroy_session(session_id) -> :ok | {:error, :session_not_found}`
Manually destroy a session before expiration.

```elixir
:ok = Relay.destroy_session(session_id)
```

## REST API Endpoints

Base URL: `http://localhost:4000/api/v1`

### Sessions

#### `POST /sessions`
Create a new ephemeral session.

**Request:**
```json
{
  "data": {
    "proof_type": "age_verification",
    "min_age": 18,
    "requester": "airline_checkin"
  },
  "ttl": 300
}
```

**Response:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "expires_in": 300,
  "message": "Ephemeral session created"
}
```

#### `GET /sessions/:session_id`
Retrieve session data (auto-destroys session).

**Response:**
```json
{
  "data": {
    "proof_type": "age_verification",
    "min_age": 18,
    "requester": "airline_checkin"
  },
  "message": "Session data retrieved and destroyed"
}
```

**Error Response:**
```json
{
  "error": "Session not found or expired"
}
```

#### `DELETE /sessions/:session_id`
Manually destroy a session.

**Response:**
```json
{
  "message": "Session destroyed"
}
```

### Health Check

#### `GET /health`
Service health check.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-10-11T10:00:00Z",
  "services": {
    "session_registry": "operational",
    "cleanup_agent": "operational"
  }
}
```

## Data Structures

### Rust Types

```rust
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
pub struct ZkProof {
    pub commitment: Vec<u8>,
    pub challenge: Vec<u8>,
    pub response: Vec<u8>,
    pub public_inputs: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletCredential {
    pub id: String,
    pub wallet_type: WalletProvider,
    pub did: String,
    pub public_key: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalletProvider {
    Apple,
    Google,
    Samsung,
    Custom(String),
}
```

### Elixir Types

```elixir
defmodule Relay.SessionState do
  @type t :: %{
    session_id: String.t(),
    data: map(),
    created_at: DateTime.t(),
    expires_at: DateTime.t(),
    timer_ref: reference()
  }
end

defmodule Relay.ProofRequest do
  @type t :: %{
    id: String.t(),
    proof_type: String.t(),
    requester: String.t(),
    required_claims: [String.t()],
    expires_at: DateTime.t()
  }
end
```

## Error Handling

### Rust Error Types

```rust
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// Common errors:
// - "Credential not found"
// - "Invalid proof format"
// - "Signature verification failed" 
// - "Session expired"
// - "Insufficient entropy"
```

### Elixir Error Types

```elixir
# Session errors
{:error, :session_not_found}
{:error, :session_expired}
{:error, :invalid_session_data}

# Proof errors  
{:error, :invalid_proof_format}
{:error, :verification_failed}
{:error, :insufficient_claims}
```

## Integration Examples

### Age Verification Flow

```rust
// 1. Create proof system
let mut proof_system = ProofZK::new();
let wallet = WalletIntegration::new();

// 2. Generate age proof
let age_proof = ZkProofSystem::prove_age_over(25, 18)?;

// 3. Create ephemeral session
let session_data = serde_json::json!({
    "proof": age_proof,
    "verification_type": "age_over_18"
});

// 4. Verify with wallet response
let wallet_response = wallet.request_selective_disclosure(
    "credential_id",
    SelectiveDisclosureRequest {
        fields_requested: vec!["age_over_18".to_string()],
        // ... other fields
    }
).await?;

// 5. Final verification
let verified = verify_age_proof_complete(&age_proof, &wallet_response, 18).await?;
```

### Genetic Marker Flow

```rust
// 1. User has genetic markers (stored in wallet only)
let user_markers = vec!["BRCA1".to_string(), "APOE4".to_string()];

// 2. Service requests specific markers
let requested = vec!["BRCA1".to_string()];

// 3. Generate selective proof
let (genetic_proof, wallet_response) = create_genetic_marker_proof(
    &user_markers,
    &requested,
    &wallet
).await?;

// 4. Service verifies markers without seeing others
let verified = verify_genetic_proof_complete(
    &genetic_proof,
    &wallet_response, 
    &requested
).await?;

// Result: Service knows BRCA1 status, doesn't know APOE4 or other markers
```

## Rate Limiting & Security

### Session Limits
- **Max Sessions per IP**: 100/hour
- **Max Session Duration**: 1 hour
- **Cleanup Frequency**: Every 60 seconds

### Proof Verification Limits
- **Max Proof Size**: 1MB
- **Verification Timeout**: 30 seconds
- **Failed Verification Rate**: 10/minute

### Wallet Integration Security
- **Signature Verification**: Required for all wallet responses
- **Nonce Validation**: Prevents replay attacks
- **TLS 1.3**: All communications encrypted

---

*For more examples and advanced usage, see the `/demos` directory in the repository.*