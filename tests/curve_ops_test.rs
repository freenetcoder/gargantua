use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use zerosol_solana::curve_ops::{
    CurveOpsManager, PrecomputedTable, SpecializedOps, PrecomputedConstants,
    init_curve_ops, get_curve_ops, get_precomputed_constants,
};
use zerosol_solana::utils::G1Point;
use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_POINT,
    scalar::Scalar,
    ristretto::RistrettoPoint,
};

#[tokio::test]
async fn test_precomputed_table_performance() {
    let base = RISTRETTO_BASEPOINT_POINT;
    let table = PrecomputedTable::new(base);
    
    // Test multiple scalar multiplications
    let scalars = vec![
        Scalar::from(1u64),
        Scalar::from(5u64),
        Scalar::from(10u64),
        Scalar::from(255u64),
    ];
    
    for scalar in scalars {
        let result_table = table.scalar_mul(&scalar);
        let result_standard = base * scalar;
        assert_eq!(result_table, result_standard);
    }
}

#[tokio::test]
async fn test_curve_ops_manager_initialization() {
    init_curve_ops();
    
    let ops = get_curve_ops();
    let constants = get_precomputed_constants();
    
    // Test that managers are properly initialized
    assert_eq!(ops.generator_table.base, RISTRETTO_BASEPOINT_POINT);
    assert!(constants.powers_of_two.len() > 0);
}

#[tokio::test]
async fn test_optimized_pedersen_commitment() {
    init_curve_ops();
    let ops = get_curve_ops();
    
    let value = Scalar::from(42u64);
    let blinding = Scalar::from(123u64);
    
    let commitment_optimized = ops.pedersen_commit(&value, &blinding);
    let commitment_standard = RISTRETTO_BASEPOINT_POINT * value + ops.h_generator_table.base * blinding;
    
    assert_eq!(commitment_optimized, commitment_standard);
}

#[tokio::test]
async fn test_batch_operations() {
    init_curve_ops();
    let ops = get_curve_ops();
    
    // Test batch scalar multiplication
    ops.add_to_batch(Scalar::from(2u64), RISTRETTO_BASEPOINT_POINT);
    ops.add_to_batch(Scalar::from(3u64), RISTRETTO_BASEPOINT_POINT);
    ops.add_to_batch(Scalar::from(5u64), RISTRETTO_BASEPOINT_POINT);
    
    let batch_result = ops.execute_batch();
    let expected = RISTRETTO_BASEPOINT_POINT * Scalar::from(10u64); // 2 + 3 + 5
    
    assert_eq!(batch_result, expected);
}

#[tokio::test]
async fn test_cached_point_operations() {
    init_curve_ops();
    let ops = get_curve_ops();
    
    let p1 = RISTRETTO_BASEPOINT_POINT;
    let p2 = RISTRETTO_BASEPOINT_POINT * Scalar::from(2u64);
    
    // First call should compute and cache
    let result1 = ops.cached_point_add(&p1, &p2);
    let (cache_size_1, _) = ops.cache_stats();
    
    // Second call should use cache
    let result2 = ops.cached_point_add(&p1, &p2);
    let (cache_size_2, _) = ops.cache_stats();
    
    assert_eq!(result1, result2);
    assert_eq!(cache_size_1, cache_size_2); // Cache size shouldn't change on second call
}

#[tokio::test]
async fn test_linear_combination() {
    init_curve_ops();
    let ops = get_curve_ops();
    
    let scalars = vec![
        Scalar::from(2u64),
        Scalar::from(3u64),
        Scalar::from(5u64),
    ];
    
    let points = vec![
        RISTRETTO_BASEPOINT_POINT,
        RISTRETTO_BASEPOINT_POINT * Scalar::from(2u64),
        RISTRETTO_BASEPOINT_POINT * Scalar::from(3u64),
    ];
    
    let result = ops.linear_combination(&scalars, &points).unwrap();
    
    // Manual calculation: 2*P + 3*(2P) + 5*(3P) = 2P + 6P + 15P = 23P
    let expected = RISTRETTO_BASEPOINT_POINT * Scalar::from(23u64);
    
    assert_eq!(result, expected);
}

#[tokio::test]
async fn test_specialized_ops_batch_verify() {
    let commitments = vec![
        RISTRETTO_BASEPOINT_POINT,
        RISTRETTO_BASEPOINT_POINT * Scalar::from(2u64),
        RISTRETTO_BASEPOINT_POINT * Scalar::from(3u64),
    ];
    
    let values = vec![
        Scalar::from(1u64),
        Scalar::from(2u64),
        Scalar::from(3u64),
    ];
    
    let blindings = vec![
        Scalar::zero(),
        Scalar::zero(),
        Scalar::zero(),
    ];
    
    init_curve_ops();
    let ops = get_curve_ops();
    
    let result = SpecializedOps::batch_verify_commitments(
        &commitments,
        &values,
        &blindings,
        ops,
    ).unwrap();
    
    assert!(result);
}

#[tokio::test]
async fn test_batch_scalar_inversion() {
    let scalars = vec![
        Scalar::from(2u64),
        Scalar::from(3u64),
        Scalar::from(5u64),
        Scalar::from(7u64),
    ];
    
    let inverses = SpecializedOps::batch_invert(&scalars).unwrap();
    
    // Verify that each scalar * its inverse = 1
    for (scalar, inverse) in scalars.iter().zip(inverses.iter()) {
        assert_eq!(scalar * inverse, Scalar::one());
    }
}

#[tokio::test]
async fn test_precomputed_constants() {
    init_curve_ops();
    let constants = get_precomputed_constants();
    
    // Test powers of 2
    for i in 0..10 {
        let power = constants.power_of_two(i).unwrap();
        let expected = Scalar::from(1u64 << i);
        assert_eq!(power, expected);
    }
    
    // Test small scalars
    for i in 0..16 {
        let scalar = constants.small_scalar(i).unwrap();
        let expected = Scalar::from(i as u64);
        assert_eq!(scalar, expected);
    }
    
    // Test generator multiples
    for i in 0..10 {
        let multiple = constants.generator_multiple(i).unwrap();
        let expected = RISTRETTO_BASEPOINT_POINT * Scalar::from(i as u64);
        assert_eq!(multiple, expected);
    }
}

#[tokio::test]
async fn test_hash_to_curve_optimized() {
    let data = b"test_data_for_hashing";
    
    let point1 = SpecializedOps::hash_to_curve_optimized(data);
    let point2 = SpecializedOps::hash_to_curve_optimized(data);
    
    // Same input should produce same output
    assert_eq!(point1, point2);
    
    // Different input should produce different output
    let point3 = SpecializedOps::hash_to_curve_optimized(b"different_data");
    assert_ne!(point1, point3);
}

#[tokio::test]
async fn test_range_constraint_verification() {
    let valid_commitments = vec![
        RISTRETTO_BASEPOINT_POINT,
        RISTRETTO_BASEPOINT_POINT * Scalar::from(100u64),
    ];
    
    let invalid_commitments = vec![
        RistrettoPoint::identity(), // Identity point should be invalid
        RISTRETTO_BASEPOINT_POINT,
    ];
    
    let result_valid = SpecializedOps::verify_range_constraints(&valid_commitments, 32).unwrap();
    let result_invalid = SpecializedOps::verify_range_constraints(&invalid_commitments, 32).unwrap();
    
    assert!(result_valid);
    assert!(!result_invalid);
}

#[tokio::test]
async fn test_performance_comparison() {
    use std::time::Instant;
    
    init_curve_ops();
    let ops = get_curve_ops();
    
    let scalar = Scalar::from(12345u64);
    let point = RISTRETTO_BASEPOINT_POINT;
    
    // Test optimized scalar multiplication
    let start = Instant::now();
    for _ in 0..100 {
        let _result = ops.fast_scalar_mul(&point, &scalar);
    }
    let optimized_time = start.elapsed();
    
    // Test standard scalar multiplication
    let start = Instant::now();
    for _ in 0..100 {
        let _result = point * scalar;
    }
    let standard_time = start.elapsed();
    
    println!("Optimized time: {:?}", optimized_time);
    println!("Standard time: {:?}", standard_time);
    
    // The optimized version should be competitive or faster
    // Note: For the generator point, it should be faster due to precomputed table
}