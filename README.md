# ProofZK - Privacy-First DID Service MVP

> **Zero central data storage. Maximum privacy. Selective disclosure by default.**

ProofZK is a DID-based privacy service that revolutionizes how applications handle sensitive data by eliminating central storage entirely. Instead of collecting and storing personal information, applications request cryptographic proofs that verify claims without revealing underlying data.

## 🚫 What ProofZK Eliminates

- ❌ Central data warehouses
- ❌ Personal data storage
- ❌ Data breach risk
- ❌ Compliance burden
- ❌ User onboarding friction

## ✅ What ProofZK Enables

- ✅ Zero-knowledge proof verification
- ✅ Selective disclosure by default
- ✅ Existing wallet integration (Apple/Google/Samsung)
- ✅ Ephemeral data sessions
- ✅ Privacy-preserving compliance

## Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   User Wallet   │    │  ProofZK Core   │    │ Application/    │
│ (Apple/Google/  │◄──►│   (Rust)        │◄──►│   Service       │
│  Samsung)       │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │              ┌─────────────────┐              │
         └──────────────►│ Ephemeral Relay │◄─────────────┘
                         │   (Elixir)      │
                         └─────────────────┘
```

## Core Components

### 🦀 Rust Core (`/core`)
- **DID Management**: Decentralized identity creation and resolution
- **Zero-Knowledge Proofs**: Age verification, genetic markers, custom claims
- **Wallet Integration**: Apple Wallet, Google Wallet, Samsung Wallet support
- **Cryptographic Primitives**: Ed25519 signatures, bulletproofs, commitments

### ⚗️ Elixir Relay (`/relay`)
- **Ephemeral Sessions**: Temporary data storage with auto-expiration
- **GenServer Architecture**: Fault-tolerant actor model for session management
- **REST API**: HTTP endpoints for session and proof management
- **Auto-Cleanup**: Background processes remove expired sessions

### 📱 Wallet Integration (`/wallet-integration`)
- **Selective Disclosure**: User chooses which fields to reveal
- **Deep Links**: Native wallet app integration
- **Proof Requests**: Standardized proof request/response protocol
- **Biometric Security**: Leverage device-native security features

## Use Cases

### ✈️ Airline Check-in
**Problem**: Airlines collect and store passenger birthdates for age verification  
**ProofZK Solution**: Prove age ≥ 18 without revealing birthdate

```rust
// Generate proof without revealing actual age
let age_proof = ZkProofSystem::prove_age_over(actual_age, 18)?;

// Wallet discloses only: "age_over_18: true"
// Wallet does NOT disclose: birthdate, exact age, name
```

### 🧬 Genomics Applications  
**Problem**: Health apps store full genetic profiles for targeted recommendations  
**ProofZK Solution**: Prove specific genetic markers without revealing full genome

```rust
// Prove presence of specific markers only
let genetic_proof = ZkProofSystem::prove_genetic_markers(
    &user_markers, 
    &["BRCA1", "MTHFR"]
)?;

// Discloses: presence of requested markers
// Does NOT disclose: other variants, family history, full genome
```

## Quick Start

### Prerequisites
- Rust 1.70+
- Elixir 1.15+
- Mix (Elixir build tool)

### 1. Build Core Components
```bash
cd core
cargo build --release
cargo run  # Start DID and ZK proof system
```

### 2. Start Ephemeral Relay
```bash
cd relay
mix deps.get
mix phx.server  # Starts on http://localhost:4000
```

### 3. Run Demos

**Airline Demo:**
```bash
cd demos/airline
cargo run
```

**Genomics Demo:**
```bash
cd demos/genomics  
cargo run
```

## API Examples

### Create Ephemeral Session
```bash
curl -X POST http://localhost:4000/api/v1/sessions \
  -H "Content-Type: application/json" \
  -d '{
    "data": {"proof_request": "age_verification"}, 
    "ttl": 300
  }'
```

### Retrieve Session Data (Auto-Destroys)
```bash
curl http://localhost:4000/api/v1/sessions/{session_id}
```

### Request Age Proof
```rust
let proof_request = ProofRequest {
    proof_type: ProofType::AgeVerification { min_age: 18 },
    required_claims: vec!["age_over_18".to_string()],
    context: "airline_boarding".to_string(),
    expires_at: Utc::now() + Duration::minutes(5),
};
```

## Wallet Integration

### Apple Wallet
```json
{
  "format": "apple_wallet",
  "capabilities": ["selective_disclosure", "age_verification"],
  "endpoints": {
    "proof_request": "proofzk://request",
    "selective_disclosure": "proofzk://disclose"
  }
}
```

### Google Wallet
```json
{
  "format": "google_wallet", 
  "object_class": "proofzk.privacy.credential",
  "capabilities": ["zero_knowledge_proof", "selective_disclosure"],
  "wallet_integration": {
    "api_endpoint": "https://relay.proofzk.com/api/v1"
  }
}
```

## Privacy Guarantees

### 🔒 Zero Central Storage
- No user data stored in application databases
- Proofs verify claims without revealing underlying data
- Ephemeral sessions auto-expire after successful verification

### 🎯 Selective Disclosure
- Users control exactly which fields are revealed
- "Prove age ≥ 18" instead of "store birthdate"
- "Prove BRCA1 marker" instead of "store full genome"

### ⏰ Ephemeral Architecture  
- Session data exists only during active transactions
- Auto-expiration prevents data accumulation
- No persistent traces of sensitive information

### 🔐 Cryptographic Verification
- Zero-knowledge proofs enable verification without revelation
- Ed25519 signatures ensure authentic wallet responses  
- Bulletproofs provide efficient range proofs (age verification)

## Development Roadmap

- [ ] **Phase 1**: MVP Core (✅ Complete)
- [ ] **Phase 2**: Production wallet integrations
- [ ] **Phase 3**: Additional proof types (location, credentials)
- [ ] **Phase 4**: Mobile SDKs (iOS/Android)
- [ ] **Phase 5**: Enterprise deployment tools

## Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## Security Considerations

- **Proof Verification**: Always verify both ZK proofs and wallet signatures
- **Session Management**: Implement proper TTL and cleanup mechanisms
- **Cryptographic Libraries**: Use audited libraries (ring, dalek family)
- **Wallet Security**: Leverage device-native biometric security
- **Network Security**: Use TLS 1.3 for all communications

## License

MIT License - see [LICENSE](LICENSE) for details

## Support

- **Documentation**: [docs.proofzk.com](https://docs.proofzk.com)
- **Issues**: [GitHub Issues](https://github.com/proofzk/proofzk/issues)
- **Discord**: [ProofZK Community](https://discord.gg/proofzk)

---

**"Don't store data. Prove claims."** - ProofZK Team