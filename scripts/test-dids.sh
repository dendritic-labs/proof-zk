#!/bin/bash

# ProofZK DID Testing Script
# This script demonstrates how to test ProofZK with real DID methods

echo "🆔 ProofZK DID Testing Setup"
echo "============================"

# Step 1: Build the core with DID support
echo "📦 Building ProofZK core with DID support..."
cd core
cargo build --release

# Step 2: Run DID testing suite
echo "🧪 Running DID test suite..."
cargo run --example did_testing

# Step 3: Create a sample DID:web document
echo "🌐 Creating sample DID:web setup..."
mkdir -p ../test-web-did/.well-known

# Generate a DID:web document for testing
cat > ../test-web-did/.well-known/did.json << 'EOF'
{
  "@context": [
    "https://www.w3.org/ns/did/v1",
    "https://w3id.org/security/suites/ed25519-2020/v1"
  ],
  "id": "did:web:localhost:8000",
  "verificationMethod": [
    {
      "id": "did:web:localhost:8000#key-1",
      "type": "Ed25519VerificationKey2020",
      "controller": "did:web:localhost:8000",
      "publicKeyMultibase": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
    }
  ],
  "authentication": [
    "did:web:localhost:8000#key-1"
  ],
  "service": [
    {
      "id": "did:web:localhost:8000#proofzk-relay",
      "type": "ProofZKRelay", 
      "serviceEndpoint": "http://localhost:4000/api/v1"
    }
  ]
}
EOF

echo "✅ Created test DID:web document at test-web-did/.well-known/did.json"

# Step 4: Start a simple web server for DID:web testing
echo "🚀 Starting web server for DID:web testing..."
echo "   Visit: http://localhost:8000/.well-known/did.json"
echo "   DID:   did:web:localhost:8000"
echo ""
echo "To test DID:web resolution:"
echo "1. Keep this server running"
echo "2. Run: cargo run --example did_testing"
echo "3. Or test manually: curl http://localhost:8000/.well-known/did.json"
echo ""

cd ../test-web-did
python3 -m http.server 8000