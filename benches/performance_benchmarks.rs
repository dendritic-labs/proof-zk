use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use proofzk_core::*;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark age proof generation performance
fn bench_age_proof_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("age_proof_generation");
    
    // Test different age ranges to ensure consistent performance
    for age in [18, 25, 50, 75, 100] {
        group.bench_with_input(
            format!("age_{}", age), 
            &age, 
            |b, &age| {
                b.iter(|| {
                    let proof = ZkProofSystem::prove_age_over(
                        black_box(age), 
                        black_box(18)
                    ).unwrap();
                    black_box(proof)
                });
            }
        );
    }
    
    group.finish();
}

/// Benchmark age proof verification performance  
fn bench_age_proof_verification(c: &mut Criterion) {
    // Pre-generate proofs for verification benchmarking
    let proofs: Vec<_> = (18..100).map(|age| {
        ZkProofSystem::prove_age_over(age, 18).unwrap()
    }).collect();
    
    let mut group = c.benchmark_group("age_proof_verification");
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("verify_age_proof", |b| {
        let mut idx = 0;
        b.iter(|| {
            let proof = &proofs[idx % proofs.len()];
            idx += 1;
            
            let result = ZkProofSystem::verify_age_proof(
                black_box(proof), 
                black_box(18)
            ).unwrap();
            black_box(result)
        });
    });
    
    group.finish();
}

/// Benchmark DID operations
fn bench_did_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("did_operations");
    
    // DID creation
    group.bench_function("did_creation", |b| {
        b.iter(|| {
            let identity = DidIdentity::new().unwrap();
            black_box(identity)
        });
    });
    
    // DID signature generation
    let identity = DidIdentity::new().unwrap();
    let message = b"ProofZK benchmark message for signing performance test";
    
    group.bench_function("did_signature", |b| {
        b.iter(|| {
            let signature = identity.sign_message(black_box(message)).unwrap();
            black_box(signature)
        });
    });
    
    // DID signature verification
    let signature = identity.sign_message(message).unwrap();
    
    group.bench_function("did_verification", |b| {
        b.iter(|| {
            let result = identity.verify_signature(
                black_box(message), 
                black_box(&signature)
            ).unwrap();
            black_box(result)
        });
    });
    
    group.finish();
}

/// Benchmark genetic marker proofs
fn bench_genetic_marker_proofs(c: &mut Criterion) {
    let user_markers = vec![
        "BRCA1".to_string(), "BRCA2".to_string(), "APOE4".to_string(),
        "CYP2D6".to_string(), "MTHFR".to_string(), "COMT".to_string(),
    ];
    
    let mut group = c.benchmark_group("genetic_marker_proofs");
    
    // Test different numbers of required markers
    for required_count in [1, 3, 5] {
        let required_markers = user_markers[0..required_count].to_vec();
        
        group.bench_with_input(
            format!("markers_{}", required_count),
            &required_markers,
            |b, required| {
                b.iter(|| {
                    let proof = ZkProofSystem::prove_genetic_markers(
                        black_box(&user_markers),
                        black_box(required)
                    ).unwrap();
                    black_box(proof)
                });
            }
        );
    }
    
    group.finish();
}

/// Benchmark ephemeral relay performance
fn bench_relay_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("relay_operations");
    
    // Session creation
    group.bench_function("session_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let data = serde_json::json!({
                "proof_type": "age_verification",
                "timestamp": chrono::Utc::now()
            });
            
            let session_id = relay::create_session(
                black_box(data), 
                black_box(300)
            ).await.unwrap();
            black_box(session_id)
        });
    });
    
    // Session retrieval (creates and immediately retrieves)
    group.bench_function("session_retrieval", |b| {
        b.to_async(&rt).iter(|| async {
            let data = serde_json::json!({"test": "data"});
            let session_id = relay::create_session(data.clone(), 300).await.unwrap();
            
            let retrieved_data = relay::get_session_data(
                black_box(&session_id)
            ).await.unwrap();
            black_box(retrieved_data)
        });
    });
    
    group.finish();
}

/// Benchmark concurrent proof generation (1Password scale)
fn bench_concurrent_proofs(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_proofs");
    
    // Test different concurrency levels
    for concurrency in [10, 100, 1000] {
        group.throughput(Throughput::Elements(concurrency));
        
        group.bench_with_input(
            format!("concurrent_{}", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async move {
                    let tasks = (0..concurrency).map(|i| {
                        tokio::spawn(async move {
                            ZkProofSystem::prove_age_over((20 + (i % 50)) as u8, 18)
                        })
                    });
                    
                    let results = futures::future::join_all(tasks).await;
                    black_box(results)
                });
            }
        );
    }
    
    group.finish();
}

/// Memory usage benchmarks
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    // Measure memory overhead of proof storage
    group.bench_function("proof_memory_overhead", |b| {
        b.iter(|| {
            let proofs: Vec<_> = (0..1000).map(|i| {
                ZkProofSystem::prove_age_over((18 + (i % 50)) as u8, 18).unwrap()
            }).collect();
            
            // Force memory allocation
            black_box(proofs)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_age_proof_generation,
    bench_age_proof_verification, 
    bench_did_operations,
    bench_genetic_marker_proofs,
    bench_relay_operations,
    bench_concurrent_proofs,
    bench_memory_usage
);

criterion_main!(benches);