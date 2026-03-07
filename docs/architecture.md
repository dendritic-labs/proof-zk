# ProofZK Architecture Documentation

## System Overview

ProofZK implements a privacy-first architecture that eliminates central data storage through three core principles:

1. **Proof-Based Verification**: Applications verify claims without storing underlying data
2. **Selective Disclosure**: Users control exactly which information is revealed  
3. **Ephemeral Sessions**: Temporary data access that auto-expires

## Component Architecture

### Core Layer (Rust)

**Purpose**: Cryptographic primitives and DID management  
**Location**: `/core`  
**Key Functions**:
- DID creation and resolution
- Zero-knowledge proof generation/verification
- Wallet protocol integration
- Cryptographic utilities

```rust
// Core proof generation
pub fn prove_age_over(actual_age: u8, min_age: u8) -> Result<AgeProof>
pub fn prove_genetic_markers(user_markers: &[String], required: &[String]) -> Result<GeneticMarkerProof>
```

### Relay Layer (Elixir)

**Purpose**: Ephemeral data management with auto-expiration  
**Location**: `/relay`  
**Key Functions**:
- Temporary session creation
- Auto-cleanup of expired data
- REST API for session management
- Real-time session monitoring

```elixir
# Ephemeral session lifecycle
{:ok, session_id} = Relay.create_session(data, ttl_seconds)
{:ok, data} = Relay.get_session_data(session_id)  # Auto-destroys
```

### Wallet Integration Layer

**Purpose**: Native wallet connectivity and selective disclosure  
**Location**: `/wallet-integration`  
**Supported Wallets**:
- Apple Wallet (iOS)
- Google Wallet (Android)  
- Samsung Wallet (Samsung devices)

## Data Flow

### 1. Proof Request Flow
```
Application → Core → Relay → Wallet → User
```

1. Application creates proof request
2. Core generates cryptographic challenge  
3. Relay creates ephemeral session
4. Wallet receives selective disclosure request
5. User approves specific fields only

### 2. Proof Response Flow
```
User → Wallet → Relay → Core → Application
```

1. User approves disclosure in wallet
2. Wallet generates signed response
3. Relay temporarily stores proof data
4. Core verifies cryptographic proofs
5. Application receives verification result
6. Session auto-expires

## Security Model

### Threat Model

**Protected Against**:
- Data breaches (no central storage)
- Over-disclosure (selective by default)
- Persistent tracking (ephemeral sessions)
- Unauthorized access (cryptographic verification)

**Not Protected Against**:
- Device compromise
- User social engineering
- Network traffic analysis (use TLS)
- Quantum attacks (post-quantum roadmap)

### Cryptographic Foundations

**Digital Signatures**: Ed25519 for DID authentication  
**Zero-Knowledge Proofs**: Bulletproofs for efficient range proofs  
**Commitments**: Pedersen commitments for hiding values  
**Hashing**: SHA-256 for integrity verification

## Privacy Architecture

### Zero Central Storage

```
Traditional App:          ProofZK App:
┌─────────────┐          ┌─────────────┐
│ User Data   │          │ Proof Only  │
│ Database    │    VS    │ Verification│
│ (Permanent) │          │ (Ephemeral) │
└─────────────┘          └─────────────┘
```

### Selective Disclosure Protocol

```json
{
  "disclosure_request": {
    "fields_available": ["name", "age", "birthdate", "address"],
    "fields_requested": ["age_over_18"],
    "purpose": "age_verification",
    "user_choice": {
      "age_over_18": true,     // ✅ Approved
      "exact_age": false,      // ❌ Denied  
      "birthdate": false,      // ❌ Denied
      "name": false            // ❌ Denied
    }
  }
}
```

## Use Case Implementations

### Airline Age Verification

**Traditional Approach**:
```sql
-- Stored permanently
INSERT INTO passengers (name, birthdate, age, flight_id) 
VALUES ('John Doe', '1990-05-15', 33, 'UA123');
```

**ProofZK Approach**:
```rust
// Verified temporarily
let age_proof = prove_age_over(user_age, 18)?;
let verification = verify_age_proof(&age_proof, 18)?;
// No data stored, session expires
```

### Genomics Selective Disclosure

**Traditional Approach**:
```sql
-- Full genome stored
INSERT INTO genetic_profiles (user_id, full_genome_sequence, variants) 
VALUES (123, 'ATCG...', '["BRCA1", "APOE4", ...]');
```

**ProofZK Approach**:
```rust
// Only requested markers verified
let genetic_proof = prove_genetic_markers(
    &user_markers, 
    &["BRCA1"]  // Only this marker verified
)?;
// Other genetic data never disclosed
```

## Session Management

### Ephemeral Session Lifecycle

```elixir
defmodule Relay.EphemeralSession do
  # Created with TTL
  def init({session_id, data, ttl_seconds}) do
    timer_ref = Process.send_after(self(), :expire, ttl_seconds * 1000)
    {:ok, %{data: data, timer_ref: timer_ref}}
  end
  
  # Auto-expires
  def handle_info(:expire, state) do
    {:stop, :normal, state}
  end
  
  # Consumed once
  def handle_call(:get_and_destroy, _from, state) do
    {:stop, :normal, {:ok, state.data}, state}
  end
end
```

## Integration Patterns

### RESTful API Integration

```bash
# Create proof session
POST /api/v1/sessions
{
  "data": {"proof_type": "age_verification", "min_age": 18},
  "ttl": 300
}

# Response
{
  "session_id": "uuid-here",
  "expires_in": 300
}

# Retrieve proof (auto-destroys)
GET /api/v1/sessions/{session_id}
{
  "data": {"age_verified": true},
  "message": "Session data retrieved and destroyed"
}
```

### Wallet Deep Link Integration

```json
{
  "apple_wallet": {
    "deep_link": "proofzk://request?session_id=uuid",
    "callback": "myapp://proof-response"
  },
  "google_wallet": {
    "intent": "com.proofzk.VERIFY_CLAIM",
    "extras": {"session_id": "uuid"}
  }
}
```

## Deployment Architecture

### Development Setup
```
┌─────────────────┐    ┌─────────────────┐
│ Rust Core       │    │ Elixir Relay    │
│ localhost:8080  │◄──►│ localhost:4000  │
└─────────────────┘    └─────────────────┘
```

### Production Setup
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Load Balancer   │    │ Core Cluster    │    │ Relay Cluster   │
│ (nginx/envoy)   │◄──►│ (Kubernetes)    │◄──►│ (Kubernetes)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Performance Considerations

### Proof Generation
- **Age Proofs**: ~10ms generation, ~5ms verification
- **Genetic Proofs**: ~50ms generation, ~20ms verification  
- **Custom Proofs**: Varies by complexity

### Session Management
- **Session Creation**: ~1ms
- **Session Retrieval**: ~1ms (then destroyed)
- **Auto-Cleanup**: Runs every 60 seconds

### Scalability
- **Horizontal Scaling**: Stateless core, distributed relay
- **Session Distribution**: Consistent hashing across relay nodes
- **Database**: No persistent storage required

## Future Enhancements

### Advanced Proof Types
- Location proofs (without revealing exact location)
- Credential proofs (degree verification without transcript)
- Temporal proofs (prove access at specific time)

### Enhanced Privacy
- Post-quantum cryptography migration
- Anonymous credentials integration  
- Differential privacy for aggregated analytics

### Mobile SDKs
- iOS SDK with Apple Wallet integration
- Android SDK with Google Wallet integration
- React Native cross-platform SDK

---

*This architecture prioritizes privacy by design, ensuring that sensitive data never accumulates in central databases while still enabling robust verification and compliance.*