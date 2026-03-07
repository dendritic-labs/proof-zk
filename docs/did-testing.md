# Testing ProofZK with Real DIDs

This guide shows you how to test ProofZK with actual DID methods instead of just mock DIDs.

## 🆔 Supported DID Methods

ProofZK now supports real DID resolution for:

- **`did:key`** - Cryptographically derived DIDs (no network required)
- **`did:web`** - Web-hosted DID documents
- **`did:ion`** - Microsoft ION (Bitcoin-anchored)
- **`did:ethr`** - Ethereum-based DIDs
- **`did:proofzk`** - Our custom method (backwards compatibility)

## 🧪 Running DID Tests

### Quick Test
```bash
# Run the comprehensive DID test suite
cd core
cargo run --example did_testing
```

### Full Test Setup
```bash
# Run the complete testing script (includes web server)
./scripts/test-dids.sh
```

## 📖 Test Scenarios

### 1. DID:key Testing (No Network Required)

```rust
// Create a cryptographically derived DID
let identity = DidIdentity::new_did_key()?;
println!("DID: {}", identity.did);
// Output: did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK

// The DID can be resolved without any network calls
let resolver = UniversalDidResolver::new();
let resolved = resolver.resolve(&identity.did).await?;
```

**What this tests:**
- ✅ Cryptographic DID generation
- ✅ Self-contained resolution
- ✅ Ed25519 signature verification
- ✅ ProofZK integration with real crypto

### 2. DID:web Testing (Requires Web Server)

```rust
// Create a web-hosted DID
let identity = DidIdentity::new_did_web("example.com", Some("users/alice"))?;
println!("DID: {}", identity.did);
// Output: did:web:example.com:users:alice

// Export the document for web hosting
let did_json = identity.export_did_web_document()?;
// Host this at: https://example.com/users/alice/.well-known/did.json
```

**What this tests:**
- ✅ Web-based DID hosting
- ✅ HTTP resolution
- ✅ Domain verification
- ✅ Real-world DID document format

### 3. External DID Resolution

```rust
let resolver = UniversalDidResolver::new();

// Resolve Microsoft ION DID
let ion_did = "did:ion:EiDyOQbbZAa3aiRzeCkV7LOx3SERjjH93EXoIM3UoN4oWg";
if let Some(doc) = resolver.resolve(ion_did).await? {
    println!("Resolved real Microsoft ION DID");
}

// Resolve Ethereum DID
let ethr_did = "did:ethr:0x3b0BC51Ab9De1e5B7B6E34E5b960285805C41736";
if let Some(doc) = resolver.resolve(ethr_did).await? {
    println!("Resolved real Ethereum DID");
}
```

**What this tests:**
- ✅ Integration with real DID networks
- ✅ Microsoft ION resolution
- ✅ Ethereum DID resolution
- ✅ Universal resolver compatibility

## 🔧 Local DID:web Setup

To test DID:web resolution locally:

### Step 1: Create DID Document
```bash
mkdir -p test-web-did/.well-known
```

### Step 2: Generate DID Document
```rust
let identity = DidIdentity::new_did_web("localhost:8000", None)?;
let did_json = identity.export_did_web_document()?;
// Save to test-web-did/.well-known/did.json
```

### Step 3: Start Web Server
```bash
cd test-web-did
python3 -m http.server 8000
```

### Step 4: Test Resolution
```bash
curl http://localhost:8000/.well-known/did.json
```

### Step 5: Test with ProofZK
```rust
let resolver = UniversalDidResolver::new();
let resolved = resolver.resolve("did:web:localhost:8000").await?;
```

## 🏢 Integration with ProofZK Flows

### Real DID Age Verification
```rust
// Create real DIDs for all parties
let airline_did = DidIdentity::new_did_key()?;      // Airline
let passenger_did = DidIdentity::new_did_web("passenger.example.com", None)?; // Passenger

// Create wallet credential with real DID
let wallet_credential = WalletCredential {
    id: "real_wallet_001".to_string(),
    wallet_type: WalletProvider::Apple,
    did: passenger_did.did.clone(),  // Real DID
    public_key: passenger_did.document.public_keys[0].public_key_base58.clone(),
    metadata: HashMap::new(),
};

// Generate proof with real DID signatures
let (age_proof, wallet_response) = create_age_verification_proof(
    25, 18, &wallet_integration
).await?;

// Create W3C Verifiable Presentation
let presentation = serde_json::json!({
    "@context": ["https://www.w3.org/2018/credentials/v1"],
    "type": ["VerifiablePresentation"],
    "holder": passenger_did.did,      // Real holder DID
    "verifier": airline_did.did,      // Real verifier DID
    "proof": {
        "type": "ProofZKAgeVerification",
        "verificationMethod": format!("{}#key-1", passenger_did.did),
        "zkProof": age_proof,
        "walletResponse": wallet_response
    }
});
```

## 🌐 Testing with Public DIDs

### Known Working DIDs (for testing)

**Microsoft ION:**
```
did:ion:EiDyOQbbZAa3aiRzeCkV7LOx3SERjjH93EXoIM3UoN4oWg
```

**DID Actor (DID:web example):**
```
did:web:did.actor:alice
```

**Test Resolution:**
```rust
let resolver = UniversalDidResolver::new();

for did in ["did:ion:EiDyOQbbZAa3aiRzeCkV7LOx3SERjjH93EXoIM3UoN4oWg", 
            "did:web:did.actor:alice"] {
    match resolver.resolve(did).await {
        Ok(Some(doc)) => println!("✅ Resolved: {}", did),
        Ok(None) => println!("❌ Not found: {}", did),
        Err(e) => println!("⚠️ Error: {} - {}", did, e),
    }
}
```

## 🔍 Verification Checklist

When testing with real DIDs, verify:

- [ ] **DID Format**: Proper did:method:identifier format
- [ ] **Resolution**: Can resolve to DID Document
- [ ] **Keys**: Public keys are accessible and valid
- [ ] **Signatures**: Can verify cryptographic signatures
- [ ] **Services**: Service endpoints are reachable
- [ ] **ProofZK Integration**: Works with age/genetic proofs
- [ ] **Wallet Integration**: Compatible with wallet credentials

## 🚀 Next Steps

After testing with real DIDs:

1. **Deploy DID:web documents** to your own domain
2. **Register with DID registries** (ION, Element, etc.)
3. **Integrate with real wallet apps** using the DID standards
4. **Create production DID resolver** with caching and fallbacks
5. **Add DID rotation** and key management features

## 🐛 Troubleshooting

**DID:web resolution fails:**
- Check HTTPS certificate
- Verify `.well-known/did.json` path
- Ensure CORS headers if browser-based

**DID:key invalid:**
- Check multicodec prefix (0xed01 for ed25519)
- Verify base58 encoding
- Ensure 32-byte public key

**External DID resolution slow:**
- Use resolver caching
- Implement timeout handling
- Add fallback resolvers

---

*This testing framework validates that ProofZK works with the real DID ecosystem, not just internal mock data.*