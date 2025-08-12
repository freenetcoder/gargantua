use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use zerosol_solana::constraint_system::{
    ConstraintSystemBuilder, R1CSVerifier, RangeConstraintVerifier,
    ArithmeticConstraintVerifier, ConstraintProof, RangeConstraintProof,
    MultiplicationProof, BitConstraintProof,
};
use zerosol_solana::utils::G1Point;
use curve25519_dalek::scalar::Scalar;

#[tokio::test]
async fn test_constraint_system_basic() {
    let mut builder = ConstraintSystemBuilder::new();
    
    // Create a simple constraint system: a * b = c
    let a = builder.add_variable();
    let b = builder.add_variable();
    let c = builder.add_variable();
    
    builder.add_multiplication_constraint(a, b, c);
    
    // Create witness: a = 5, b = 6, c = 30
    let witness = vec![
        Scalar::from(5u64),
        Scalar::from(6u64),
        Scalar::from(30u64),
    ];
    
    let cs = builder.build(witness);
    let verifier = R1CSVerifier::new(cs);
    
    assert!(verifier.verify_constraints().unwrap());
}

#[tokio::test]
async fn test_constraint_system_addition() {
    let mut builder = ConstraintSystemBuilder::new();
    
    // Create constraint system: a + b = c
    let a = builder.add_variable();
    let b = builder.add_variable();
    let c = builder.add_variable();
    
    builder.add_addition_constraint(a, b, c);
    
    // Create witness: a = 10, b = 15, c = 25
    let witness = vec![
        Scalar::from(10u64),
        Scalar::from(15u64),
        Scalar::from(25u64),
    ];
    
    let cs = builder.build(witness);
    let verifier = R1CSVerifier::new(cs);
    
    assert!(verifier.verify_constraints().unwrap());
}

#[tokio::test]
async fn test_constraint_system_invalid() {
    let mut builder = ConstraintSystemBuilder::new();
    
    // Create constraint system: a * b = c
    let a = builder.add_variable();
    let b = builder.add_variable();
    let c = builder.add_variable();
    
    builder.add_multiplication_constraint(a, b, c);
    
    // Create invalid witness: a = 5, b = 6, c = 31 (should be 30)
    let witness = vec![
        Scalar::from(5u64),
        Scalar::from(6u64),
        Scalar::from(31u64),
    ];
    
    let cs = builder.build(witness);
    let verifier = R1CSVerifier::new(cs);
    
    assert!(!verifier.verify_constraints().unwrap());
}

#[tokio::test]
async fn test_range_constraint_verifier() {
    let verifier = RangeConstraintVerifier::new(8);
    
    // Create a commitment to a value within range
    let g = G1Point::generator();
    let value = Scalar::from(100u64); // Within 8-bit range (0-255)
    let commitment = g.mul(&value);
    
    // Create dummy range proof
    let range_proof = RangeConstraintProof {
        bit_commitments: vec![commitment; 8],
        bit_proofs: vec![BitConstraintProof {
            challenge: Scalar::one(),
            response: Scalar::one(),
        }; 8],
    };
    
    // This should pass basic structural validation
    let result = verifier.verify_range_constraint(&commitment, &range_proof);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_arithmetic_constraint_addition() {
    let g = G1Point::generator();
    
    // Create commitments: Com(7), Com(3), Com(10)
    let comm_a = g.mul(&Scalar::from(7u64));
    let comm_b = g.mul(&Scalar::from(3u64));
    let comm_c = g.mul(&Scalar::from(10u64));
    
    // Test addition constraint: 7 + 3 = 10
    let result = ArithmeticConstraintVerifier::verify_addition_constraint(
        &comm_a, &comm_b, &comm_c
    );
    
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_arithmetic_constraint_addition_invalid() {
    let g = G1Point::generator();
    
    // Create commitments: Com(7), Com(3), Com(11) (should be 10)
    let comm_a = g.mul(&Scalar::from(7u64));
    let comm_b = g.mul(&Scalar::from(3u64));
    let comm_c = g.mul(&Scalar::from(11u64));
    
    // Test addition constraint: 7 + 3 ≠ 11
    let result = ArithmeticConstraintVerifier::verify_addition_constraint(
        &comm_a, &comm_b, &comm_c
    );
    
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_multiplication_constraint() {
    let g = G1Point::generator();
    
    // Create commitments for multiplication
    let comm_a = g.mul(&Scalar::from(4u64));
    let comm_b = g.mul(&Scalar::from(5u64));
    let comm_c = g.mul(&Scalar::from(20u64));
    
    // Create dummy multiplication proof
    let mult_proof = MultiplicationProof {
        intermediate_commitments: vec![comm_a, comm_b],
        challenges: vec![Scalar::from(123u64), Scalar::from(456u64)],
        responses: vec![Scalar::from(789u64), Scalar::from(101u64)],
    };
    
    // Test multiplication constraint: 4 * 5 = 20
    let result = ArithmeticConstraintVerifier::verify_multiplication_constraint(
        &comm_a, &comm_b, &comm_c, &mult_proof
    );
    
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_complex_constraint_system() {
    let mut builder = ConstraintSystemBuilder::new();
    
    // Create a more complex constraint system
    // Variables: a, b, c, d, e
    // Constraints: 
    // 1. a * b = c
    // 2. c + d = e
    // 3. a + b = 10 (linear constraint)
    
    let a = builder.add_variable();
    let b = builder.add_variable();
    let c = builder.add_variable();
    let d = builder.add_variable();
    let e = builder.add_variable();
    
    // Add constraints
    builder.add_multiplication_constraint(a, b, c);
    builder.add_addition_constraint(c, d, e);
    
    // Add linear constraint: a + b - 10 = 0
    let ten_var = builder.add_public_input(Scalar::from(10u64));
    builder.add_linear_constraint(vec![
        (a, Scalar::one()),
        (b, Scalar::one()),
        (ten_var, -Scalar::one()),
    ]);
    
    // Create witness: a = 4, b = 6, c = 24, d = 6, e = 30
    let witness = vec![
        Scalar::from(4u64),  // a
        Scalar::from(6u64),  // b
        Scalar::from(24u64), // c = a * b
        Scalar::from(6u64),  // d
        Scalar::from(30u64), // e = c + d
    ];
    
    let cs = builder.build(witness);
    let verifier = R1CSVerifier::new(cs);
    
    assert!(verifier.verify_constraints().unwrap());
}

#[tokio::test]
async fn test_constraint_proof_generation() {
    let mut builder = ConstraintSystemBuilder::new();
    
    // Simple constraint system
    let a = builder.add_variable();
    let b = builder.add_variable();
    let c = builder.add_variable();
    
    builder.add_multiplication_constraint(a, b, c);
    
    let witness = vec![
        Scalar::from(3u64),
        Scalar::from(7u64),
        Scalar::from(21u64),
    ];
    
    let cs = builder.build(witness);
    let verifier = R1CSVerifier::new(cs);
    
    // Generate proof
    let proof_result = verifier.generate_proof();
    assert!(proof_result.is_ok());
    
    let proof = proof_result.unwrap();
    assert_eq!(proof.witness_commitment.len(), 3);
    assert!(!proof.constraint_proof.is_empty());
}

#[tokio::test]
async fn test_range_constraint_edge_cases() {
    let verifier = RangeConstraintVerifier::new(1); // 1-bit range (0 or 1)
    
    let g = G1Point::generator();
    
    // Test commitment to 0
    let zero_commitment = G1Point::identity();
    let range_proof_zero = RangeConstraintProof {
        bit_commitments: vec![zero_commitment],
        bit_proofs: vec![BitConstraintProof {
            challenge: Scalar::zero(),
            response: Scalar::zero(),
        }],
    };
    
    let result_zero = verifier.verify_range_constraint(&zero_commitment, &range_proof_zero);
    assert!(result_zero.is_ok());
    
    // Test commitment to 1
    let one_commitment = g;
    let range_proof_one = RangeConstraintProof {
        bit_commitments: vec![one_commitment],
        bit_proofs: vec![BitConstraintProof {
            challenge: Scalar::one(),
            response: Scalar::one(),
        }],
    };
    
    let result_one = verifier.verify_range_constraint(&one_commitment, &range_proof_one);
    assert!(result_one.is_ok());
}

#[tokio::test]
async fn test_constraint_system_with_public_inputs() {
    let mut builder = ConstraintSystemBuilder::new();
    
    // Create constraint system with public inputs
    let private_var = builder.add_variable();
    let public_input = builder.add_public_input(Scalar::from(42u64));
    let result_var = builder.add_variable();
    
    // Constraint: private_var * public_input = result_var
    builder.add_multiplication_constraint(private_var, public_input, result_var);
    
    // Witness: private_var = 2, result_var = 84
    let witness = vec![
        Scalar::from(2u64),  // private_var
        Scalar::from(84u64), // result_var
    ];
    
    let cs = builder.build(witness);
    let verifier = R1CSVerifier::new(cs);
    
    assert!(verifier.verify_constraints().unwrap());
}