use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use zerosol_solana::bulletproof::{
    BulletproofVerifier, RangeProof, InnerProductProof, OptimizedBulletproofVerifier,
    BulletproofAggregator, Transcript,
};
use zerosol_solana::utils::G1Point;
use curve25519_dalek::scalar::Scalar;

#[tokio::test]
async fn test_bulletproof_verifier_creation() {
    let verifier = BulletproofVerifier::new(64);
    assert_eq!(verifier.n, 64);
    assert_eq!(verifier.g.len(), 64);
    assert_eq!(verifier.h.len(), 64);
}

#[tokio::test]
async fn test_transcript_functionality() {
    let mut transcript = Transcript::new();
    let point = G1Point::generator();
    let scalar = Scalar::one();
    
    transcript.append_point(b"test_point", &point);
    transcript.append_scalar(b"test_scalar", &scalar);
    
    let challenge1 = transcript.challenge_scalar(b"challenge");
    let challenge2 = transcript.challenge_scalar(b"challenge");
    
    // Same label should produce different challenges due to transcript state
    assert_ne!(challenge1, challenge2);
}

#[tokio::test]
async fn test_optimized_verifier() {
    let verifier = OptimizedBulletproofVerifier::new(32);
    assert_eq!(verifier.base_verifier.n, 32);
    assert_eq!(verifier.precomputed_generators.len(), 64); // 2 * n
}

#[tokio::test]
async fn test_bulletproof_aggregator() {
    let aggregator = BulletproofAggregator::new(64);
    
    // Create dummy proofs for testing
    let commitment1 = G1Point::generator();
    let commitment2 = G1Point::generator().mul(&Scalar::from(2u64));
    
    let dummy_inner_product = InnerProductProof {
        l_vec: vec![G1Point::generator()],
        r_vec: vec![G1Point::generator()],
        a: Scalar::one(),
        b: Scalar::one(),
    };
    
    let proof1 = RangeProof {
        a: G1Point::generator(),
        s: G1Point::generator(),
        t1: G1Point::generator(),
        t2: G1Point::generator(),
        t_hat: Scalar::one(),
        tau_x: Scalar::one(),
        mu: Scalar::one(),
        inner_product_proof: dummy_inner_product.clone(),
    };
    
    let proof2 = RangeProof {
        a: G1Point::generator(),
        s: G1Point::generator(),
        t1: G1Point::generator(),
        t2: G1Point::generator(),
        t_hat: Scalar::from(2u64),
        tau_x: Scalar::from(2u64),
        mu: Scalar::from(2u64),
        inner_product_proof: dummy_inner_product,
    };
    
    let proofs = vec![(commitment1, proof1), (commitment2, proof2)];
    
    let aggregated = aggregator.aggregate_proofs(&proofs);
    assert!(aggregated.is_ok());
    
    let aggregated_proof = aggregated.unwrap();
    assert_eq!(aggregated_proof.commitments.len(), 2);
}

#[tokio::test]
async fn test_range_proof_structure() {
    let verifier = BulletproofVerifier::new(32);
    
    // Test that verifier rejects invalid proof structures
    let commitment = G1Point::generator();
    
    let invalid_inner_product = InnerProductProof {
        l_vec: vec![], // Empty vectors should be invalid
        r_vec: vec![],
        a: Scalar::one(),
        b: Scalar::one(),
    };
    
    let invalid_proof = RangeProof {
        a: G1Point::generator(),
        s: G1Point::generator(),
        t1: G1Point::generator(),
        t2: G1Point::generator(),
        t_hat: Scalar::one(),
        tau_x: Scalar::one(),
        mu: Scalar::one(),
        inner_product_proof: invalid_inner_product,
    };
    
    let result = verifier.verify_range_proof(&commitment, &invalid_proof, 32);
    assert!(result.is_err() || result.unwrap() == false);
}

#[tokio::test]
async fn test_batch_verification() {
    let verifier = OptimizedBulletproofVerifier::new(64);
    
    // Create multiple dummy proofs
    let mut proofs = Vec::new();
    
    for i in 0..5 {
        let commitment = G1Point::generator().mul(&Scalar::from(i as u64 + 1));
        let inner_product = InnerProductProof {
            l_vec: vec![G1Point::generator(); 5], // log2(32) = 5
            r_vec: vec![G1Point::generator(); 5],
            a: Scalar::from(i as u64 + 1),
            b: Scalar::from(i as u64 + 1),
        };
        
        let proof = RangeProof {
            a: G1Point::generator(),
            s: G1Point::generator(),
            t1: G1Point::generator(),
            t2: G1Point::generator(),
            t_hat: Scalar::from(i as u64 + 1),
            tau_x: Scalar::from(i as u64 + 1),
            mu: Scalar::from(i as u64 + 1),
            inner_product_proof: inner_product,
        };
        
        proofs.push((commitment, proof, 32));
    }
    
    let result = verifier.verify_batch_optimized(&proofs);
    // This should not panic and should return a result
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_transcript_determinism() {
    let point = G1Point::generator();
    let scalar = Scalar::from(42u64);
    
    // Create two identical transcripts
    let mut transcript1 = Transcript::new();
    let mut transcript2 = Transcript::new();
    
    // Add same data to both
    transcript1.append_point(b"test", &point);
    transcript1.append_scalar(b"scalar", &scalar);
    
    transcript2.append_point(b"test", &point);
    transcript2.append_scalar(b"scalar", &scalar);
    
    // Should produce same challenge
    let challenge1 = transcript1.challenge_scalar(b"challenge");
    let challenge2 = transcript2.challenge_scalar(b"challenge");
    
    assert_eq!(challenge1, challenge2);
}