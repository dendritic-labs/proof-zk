# **Claude Rust Development Guidelines for ProofZK**

## **Core Principles**

### **1. Idiomatic Rust, Always**
- Use `Result<T, E>` for fallible operations, never panic in library code
- Leverage the type system for compile-time guarantees
- Prefer `impl Trait` over boxed trait objects when possible
- Use `#[must_use]` for important return values that shouldn't be ignored

### **2. Zero Unnecessary Cloning**
```rust
// Avoid unnecessary clones
fn bad_example(data: Vec<u8>) -> String {
    let cloned = data.clone(); // unnecessary
    process_data(cloned).to_string() // another unnecessary conversion
}

// Use references and borrowing
fn good_example(data: &[u8]) -> Cow<'_, str> {
    process_data(data) // return Cow to avoid allocation when possible
}
```

### **3. Memory Safety & Performance**
- Prefer `&[T]` over `&Vec<T>` in function parameters
- Use `String` for owned, `&str` for borrowed text
- Leverage `Cow<'_, T>` for zero-copy when possible
- Explicit lifetime annotations only when required

---

## **Cryptographic Code Standards**

### **Security-First Development**
```rust
// Constant-time operations for sensitive data
use subtle::ConstantTimeEq;

impl ConstantTimeEq for SecretKey {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.bytes.ct_eq(&other.bytes)
    }
}

// Zeroize sensitive data
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
struct PrivateKey {
    #[zeroize(skip)] // Only zeroize the actual key material
    curve_type: CurveType,
    key: [u8; 32],
}
```

### **Secure Random Generation**
```rust
// Use cryptographically secure RNG
use rand_core::OsRng;
let mut rng = OsRng;

// Never use non-crypto RNG for keys
// let mut rng = thread_rng(); // NEVER for crypto!
```

---

## **Performance & Efficiency**

### **Smart Allocations**
```rust
// Pre-allocate when size is known
let mut buffer = Vec::with_capacity(expected_size);

// Use iterators instead of collecting unnecessarily
fn process_points(points: &[Point]) -> impl Iterator<Item = ProcessedPoint> + '_ {
    points.iter().map(|p| p.process()) // lazy, no intermediate allocation
}

// In-place operations when possible
fn scalar_mult_assign(&mut self, scalar: &Scalar) {
    *self = &*self * scalar; // curve25519-dalek pattern
}
```

### **Zero-Copy Parsing**
```rust
// Parse without allocating
use nom::{bytes::complete::take, IResult};

fn parse_proof(input: &[u8]) -> IResult<&[u8], ZkProof> {
    let (input, commitment_bytes) = take(32usize)(input)?;
    let commitment = CompressedRistretto::from_slice(commitment_bytes)?;
    // ... parse rest without copying
}
```

---

## **Testing Excellence**

### **Comprehensive Test Categories**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Unit tests for individual functions
    #[test]
    fn test_commitment_hiding() {
        let value = 42u64;
        let blinding1 = Scalar::random(&mut OsRng);
        let blinding2 = Scalar::random(&mut OsRng);
        
        let commit1 = commit_to_value(value, &blinding1);
        let commit2 = commit_to_value(value, &blinding2);
        
        // Different blindings should produce different commitments
        assert_ne!(commit1.compress(), commit2.compress());
    }

    // Property-based testing for crypto properties
    proptest! {
        #[test]
        fn commitment_binding_property(value1 in any::<u64>(), value2 in any::<u64>()) {
            prop_assume!(value1 != value2);
            
            let blinding = Scalar::random(&mut OsRng);
            let commit1 = commit_to_value(value1, &blinding);
            let commit2 = commit_to_value(value2, &blinding);
            
            // Same blinding, different values = different commitments
            prop_assert_ne!(commit1.compress(), commit2.compress());
        }
    }

    // Integration tests for complete protocols
    #[test]
    fn test_age_proof_protocol() {
        let actual_age = 25u8;
        let min_age = 21u8;
        
        let proof = prove_age_over(actual_age, min_age).unwrap();
        let verification = verify_age_proof(&proof, min_age).unwrap();
        
        assert!(verification);
    }
}
```

### **Benchmark Critical Paths**
```rust
#[cfg(test)]
mod benches {
    use super::*;
    use criterion::{criterion_group, criterion_main, Criterion};

    fn bench_proof_generation(c: &mut Criterion) {
        c.bench_function("age_proof_generation", |b| {
            b.iter(|| prove_age_over(25, 21))
        });
    }

    criterion_group!(benches, bench_proof_generation);
    criterion_main!(benches);
}
```

---

## **Error Handling Patterns**

### **Rich Error Types**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProofError {
    #[error("Invalid age: must be between 0 and 150, got {age}")]
    InvalidAge { age: u8 },
    
    #[error("Proof verification failed")]
    VerificationFailed,
    
    #[error("Cryptographic error: {source}")]
    CryptoError {
        #[from]
        source: curve25519_dalek::errors::InternalError,
    },
}

pub type ProofResult<T> = Result<T, ProofError>;
```

### **Early Returns with `?`**
```rust
// Clean error propagation
fn create_proof(age: u8, min_age: u8) -> ProofResult<AgeProof> {
    let validated_age = validate_age(age)?;
    let commitment = create_commitment(validated_age)?;
    let proof = generate_proof(commitment, min_age)?;
    Ok(proof)
}
```

---

## **Type Safety & API Design**

### **Newtype Patterns for Domain Types**
```rust
// Prevent mixing up different types of scalars
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlindingFactor(Scalar);

#[derive(Debug, Clone, PartialEq, Eq)]  
pub struct CommitmentValue(u64);

impl BlindingFactor {
    pub fn random() -> Self {
        Self(Scalar::random(&mut OsRng))
    }
    
    pub fn as_scalar(&self) -> &Scalar {
        &self.0
    }
}
```

### **Builder Patterns for Complex Construction**
```rust
// Clear, hard-to-misuse APIs
pub struct ProofBuilder {
    value: Option<u64>,
    blinding: Option<BlindingFactor>,
    range: Option<(u64, u64)>,
}

impl ProofBuilder {
    pub fn new() -> Self { /* ... */ }
    
    pub fn value(mut self, value: u64) -> Self {
        self.value = Some(value);
        self
    }
    
    pub fn range(mut self, min: u64, max: u64) -> Self {
        self.range = Some((min, max));
        self
    }
    
    pub fn build(self) -> ProofResult<RangeProof> {
        let value = self.value.ok_or(ProofError::MissingValue)?;
        let (min, max) = self.range.ok_or(ProofError::MissingRange)?;
        // ... construct proof
    }
}
```

---

## **Documentation Standards**

### **Comprehensive Rustdoc**
```rust
/// Creates a cryptographic commitment to a value with a blinding factor.
///
/// # Security
/// 
/// The blinding factor MUST be randomly generated and never reused.
/// Reusing blinding factors breaks the hiding property of commitments.
///
/// # Examples
///
/// ```
/// use proof_zk::{commit_to_value, BlindingFactor};
/// 
/// let value = 42u64;
/// let blinding = BlindingFactor::random();
/// let commitment = commit_to_value(value, &blinding)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns [`ProofError::InvalidValue`] if the value exceeds the field order.
pub fn commit_to_value(
    value: u64, 
    blinding: &BlindingFactor
) -> ProofResult<PedersenCommitment> {
    // Implementation...
}
```

---

## **Code Organization**

### **Module Structure**
```rust
// src/lib.rs - Public API
pub use crate::{
    proofs::{AgeProof, RangeProof},
    commitments::{PedersenCommitment, BlindingFactor},
    errors::{ProofError, ProofResult},
};

// src/proofs/mod.rs - Proof implementations
mod age;
mod range;

pub use age::AgeProof;
pub use range::RangeProof;

// src/commitments.rs - Commitment schemes
mod pedersen;
mod polynomial;

pub use pedersen::*;
```

### **Feature Flags for Optional Dependencies**
```toml
[features]
default = ["std"]
std = ["curve25519-dalek/std"]
serde = ["dep:serde", "curve25519-dalek/serde"]
batch-verify = ["dep:rayon"]
```

---

## **Concurrency Patterns**

### **Safe Parallelism**
```rust
// Batch verification with rayon
#[cfg(feature = "batch-verify")]
pub fn verify_proofs_batch(
    proofs: &[AgeProof], 
    min_ages: &[u8]
) -> ProofResult<Vec<bool>> {
    use rayon::prelude::*;
    
    proofs
        .par_iter()
        .zip(min_ages)
        .map(|(proof, &min_age)| verify_age_proof(proof, min_age))
        .collect()
}
```

---

## **Development Workflow**

### **Required Checks Before Commit**
```bash
# Format code
cargo fmt

# Lint with clippy
cargo clippy -- -D warnings

# Run all tests
cargo test

# Check documentation
cargo doc --no-deps --document-private-items

# Security audit
cargo audit

# Benchmark critical paths
cargo bench
```

### **CI/CD Requirements**
- All tests pass on stable, beta, and nightly Rust
- No clippy warnings in deny mode  
- Security audit clean
- Documentation builds without warnings
- MSRV compatibility maintained

---

## **Performance Targets**

| Operation | Target Time | Notes |
|-----------|-------------|-------|
| Age proof generation | < 1ms | Single-threaded |
| Age proof verification | < 0.5ms | Single-threaded |  
| Batch verification (100 proofs) | < 10ms | Multi-threaded |
| Commitment generation | < 0.1ms | Critical path |

---

## **Tools & Dependencies**

### **Required Development Tools**
```toml
[dev-dependencies]
criterion = "0.5"      # Benchmarking
proptest = "1.0"       # Property-based testing  
quickcheck = "1.0"     # Alternative property testing
hex-literal = "0.4"    # Test vector constants
```

### **Recommended Clippy Configuration**
```toml
# .cargo/config.toml
[alias]
check-all = ["clippy", "--", "-D", "warnings", "-D", "clippy::all", "-D", "clippy::pedantic"]
```

---

## **DON'T DO: Common Anti-Patterns to Avoid**

### **Memory Management**
- Never use `clone()` when you can borrow
- Don't collect iterators unnecessarily into Vec when you can chain
- Avoid `Box<dyn Trait>` when `impl Trait` suffices
- Don't use `Rc<RefCell<T>>` unless you absolutely need shared mutable ownership
- Never use `unsafe` without extensive documentation and testing

### **Error Handling**
- Don't use `unwrap()` or `expect()` in library code
- Avoid generic `anyhow::Error` in public APIs - use specific error types
- Don't ignore errors with `let _ = result;`
- Never panic in library functions that users will call
- Don't use `Result<T, String>` - create proper error enums

### **Cryptographic Code**
- Never use `thread_rng()` for cryptographic keys or nonces
- Don't reuse nonces or initialization vectors
- Avoid timing-dependent operations on secret data
- Never log or print secret key material
- Don't implement your own cryptographic primitives

### **API Design**
- Don't expose internal implementation details in public APIs
- Avoid taking `Vec<T>` when `&[T]` or `impl AsRef<[T]>` works
- Don't use overly generic lifetimes when concrete ones are clearer
- Avoid `String` parameters when `&str` or `impl AsRef<str>` suffices
- Never break backward compatibility without major version bump

### **Performance**
- Don't allocate in hot loops when you can pre-allocate
- Avoid unnecessary format! calls - use write! to existing buffers
- Don't use HashMap for small, fixed key sets - consider arrays or match statements
- Avoid recursion for deep call stacks - use iteration instead
- Don't ignore zero-copy opportunities with Cow or borrowed types

### **Testing**
- Don't write tests that depend on external network resources
- Avoid non-deterministic tests that sometimes fail
- Don't test implementation details - test public behavior
- Never commit tests that are marked with ignore without good reason
- Avoid overly complex test setups that obscure what's being tested

### **Documentation**
- Don't document obvious functionality
- Avoid examples that don't compile
- Don't forget to document panics and safety requirements
- Never leave TODO comments in production code
- Don't use unclear variable names in examples

### **Dependencies**
- Don't add dependencies for functionality you can implement simply
- Avoid deprecated crates or those with known security issues
- Don't use features you don't need from large crates
- Never vendor dependencies without understanding licensing
- Avoid dependencies that haven't been updated recently

### **Concurrency**
- Don't use Arc<Mutex<T>> when channels would be clearer
- Avoid shared mutable state when message passing works
- Don't forget to handle channel disconnection errors
- Never assume operations are atomic when they're not
- Avoid blocking async code with synchronous operations

---

**Remember: Cryptographic code has zero tolerance for bugs. When in doubt, prioritize correctness and security over performance.**