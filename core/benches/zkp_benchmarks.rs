use criterion::{black_box, criterion_group, criterion_main, Criterion};
use proofzk_core::zkp::ZkProofSystem;

fn bench_age_proof_generation(c: &mut Criterion) {
    c.bench_function("age_proof_generation", |b| {
        b.iter(|| {
            let actual_age = black_box(25u8);
            let min_age = black_box(21u8);
            ZkProofSystem::prove_age_over(actual_age, min_age).unwrap()
        })
    });
}

fn bench_age_proof_verification(c: &mut Criterion) {
    let proof = ZkProofSystem::prove_age_over(25, 21).unwrap();
    
    c.bench_function("age_proof_verification", |b| {
        b.iter(|| {
            ZkProofSystem::verify_age_proof(black_box(&proof), black_box(21)).unwrap()
        })
    });
}

fn bench_genetic_proof_generation(c: &mut Criterion) {
    let user_markers = vec![
        "BRCA1".to_string(), "BRCA2".to_string(), "TP53".to_string(),
        "ATM".to_string(), "PALB2".to_string(), "CHEK2".to_string()
    ];
    let required_markers = vec!["BRCA1".to_string(), "BRCA2".to_string()];
    
    c.bench_function("genetic_proof_generation", |b| {
        b.iter(|| {
            ZkProofSystem::prove_genetic_markers(
                black_box(&user_markers), 
                black_box(&required_markers)
            ).unwrap()
        })
    });
}

fn bench_genetic_proof_verification(c: &mut Criterion) {
    let user_markers = vec![
        "BRCA1".to_string(), "BRCA2".to_string(), "TP53".to_string()
    ];
    let required_markers = vec!["BRCA1".to_string(), "BRCA2".to_string()];
    let proof = ZkProofSystem::prove_genetic_markers(&user_markers, &required_markers).unwrap();
    
    c.bench_function("genetic_proof_verification", |b| {
        b.iter(|| {
            ZkProofSystem::verify_genetic_proof(
                black_box(&proof), 
                black_box(&required_markers)
            ).unwrap()
        })
    });
}

fn bench_batch_age_verification(c: &mut Criterion) {
    // Generate multiple proofs for batch testing
    let proofs: Vec<_> = (18..28u8)
        .map(|age| ZkProofSystem::prove_age_over(age, 21).unwrap())
        .collect();
    
    c.bench_function("batch_age_verification_10_proofs", |b| {
        b.iter(|| {
            for proof in black_box(&proofs) {
                let _ = ZkProofSystem::verify_age_proof(proof, 21).unwrap();
            }
        })
    });
}

criterion_group!(
    benches,
    bench_age_proof_generation,
    bench_age_proof_verification,
    bench_genetic_proof_generation,
    bench_genetic_proof_verification,
    bench_batch_age_verification
);
criterion_main!(benches);