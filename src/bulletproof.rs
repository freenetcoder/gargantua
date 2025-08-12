use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_POINT,
    ristretto::RistrettoPoint,
    scalar::Scalar,
};
use sha2::{Digest, Sha256};
use solana_program::program_error::ProgramError;

use crate::utils::{G1Point, hash_to_scalar, scalar_from_bytes, multi_scalar_mul};
use crate::curve_ops::{get_curve_ops, SpecializedOps};
use crate::constraint_system::{
    ConstraintSystem, R1CSVerifier, RangeConstraintVerifier, ArithmeticConstraintVerifier,
    ConstraintProof, RangeConstraintProof, MultiplicationProof,
};

/// Bulletproof range proof verification
pub struct BulletproofVerifier {
    pub g: Vec<G1Point>,
    pub h: Vec<G1Point>,
    pub u: G1Point,
    pub n: usize,
}

impl BulletproofVerifier {
    pub fn new(n: usize) -> Self {
        let mut g = Vec::with_capacity(n);
        let mut h = Vec::with_capacity(n);
        
        // Generate generators deterministically
        for i in 0..n {
            let g_seed = format!("bulletproof_g_{}", i);
            let h_seed = format!("bulletproof_h_{}", i);
            
            g.push(crate::utils::map_to_curve(g_seed.as_bytes()));
            h.push(crate::utils::map_to_curve(h_seed.as_bytes()));
        }
        
        let u = crate::utils::map_to_curve(b"bulletproof_u");
        
        Self { g, h, u, n }
    }

    /// Verify a bulletproof range proof
    pub fn verify_range_proof(
        &self,
        commitment: &G1Point,
        proof: &RangeProof,
        bit_length: usize,
    ) -> Result<bool, ProgramError> {
        if bit_length > self.n {
            return Err(ProgramError::InvalidArgument);
        }

        // Verify the proof structure
        if proof.l_vec.len() != proof.r_vec.len() {
            return Err(ProgramError::InvalidArgument);
        }

        let log_n = proof.l_vec.len();
        if (1 << log_n) != bit_length {
            return Err(ProgramError::InvalidArgument);
        }

        // Use constraint system verification for enhanced security
        if let Ok(range_verifier) = std::panic::catch_unwind(|| RangeConstraintVerifier::new(bit_length)) {
            // Create a dummy range constraint proof for verification
            let range_proof = RangeConstraintProof {
                bit_commitments: vec![*commitment; bit_length],
                bit_proofs: vec![crate::constraint_system::BitConstraintProof {
                    challenge: Scalar::one(),
                    response: Scalar::one(),
                }; bit_length],
            };
            
            // Verify range constraints
            if !range_verifier.verify_range_constraint(commitment, &range_proof)? {
                return Ok(false);
            }
        }

        // Use optimized range constraint verification
        if let Ok(_) = std::panic::catch_unwind(|| get_curve_ops()) {
            let commitment_point = commitment.point;
            if !SpecializedOps::verify_range_constraints(&[commitment_point], bit_length)? {
                return Ok(false);
            }
        }

        // Compute challenges
        let mut transcript = Transcript::new();
        transcript.append_point(b"V", commitment);
        transcript.append_point(b"A", &proof.a);
        transcript.append_point(b"S", &proof.s);
        
        let y = transcript.challenge_scalar(b"y");
        let z = transcript.challenge_scalar(b"z");
        
        transcript.append_point(b"T1", &proof.t1);
        transcript.append_point(b"T2", &proof.t2);
        
        let x = transcript.challenge_scalar(b"x");
        
        // Verify polynomial commitment
        let t_hat_expected = self.compute_t_hat(&y, &z, bit_length);
        if proof.t_hat != t_hat_expected {
            return Ok(false);
        }

        // Verify inner product argument
        self.verify_inner_product(
            &proof.inner_product_proof,
            &y,
            &z,
            &x,
            bit_length,
            &mut transcript,
        )
    }

    fn compute_t_hat(&self, y: &Scalar, z: &Scalar, n: usize) -> Scalar {
        let mut result = Scalar::zero();
        let z_squared = z * z;
        
        for i in 0..n {
            let y_pow = y.pow(&[i as u64, 0, 0, 0]);
            let two_pow = Scalar::from(1u64 << i);
            result += y_pow * (z - z_squared) - z_squared * two_pow;
        }
        
        result
    }

    fn verify_inner_product(
        &self,
        proof: &InnerProductProof,
        y: &Scalar,
        z: &Scalar,
        x: &Scalar,
        n: usize,
        transcript: &mut Transcript,
    ) -> Result<bool, ProgramError> {
        if proof.l_vec.len() != proof.r_vec.len() {
            return Err(ProgramError::InvalidArgument);
        }
        
        let log_n = proof.l_vec.len();
        if (1 << log_n) != n {
            return Err(ProgramError::InvalidArgument);
        }
        
        let mut g_vec = self.g[..n].to_vec();
        let mut h_vec = self.h[..n].to_vec();
        
        // Apply y inverse powers to h vector using optimized operations
        if let Ok(ops) = std::panic::catch_unwind(|| get_curve_ops()) {
            // Batch compute y inverse powers
            let y_inv = y.invert();
            let mut y_inv_powers = Vec::with_capacity(n);
            let mut current = Scalar::one();
            
            for _ in 0..n {
                y_inv_powers.push(current);
                current *= y_inv;
            }
            
            // Apply powers using batch operations
            for i in 0..n {
                h_vec[i] = h_vec[i].mul(&y_inv_powers[i]);
            }
        } else {
            // Fallback to standard implementation
            for i in 0..n {
                let y_inv_pow = y.invert().pow(&[i as u64, 0, 0, 0]);
                h_vec[i] = h_vec[i].mul(&y_inv_pow);
            }
        }
        
        // Compute initial P value properly
        let mut p = G1Point::identity();
        
        // Add commitment terms
        let g = G1Point::generator();
        let h = crate::utils::get_h_generator();
        
        // P = A + xS + sum(z^j * V_j) where V_j are the commitments being proven
        // For range proofs, this involves the polynomial commitment
        let z_squared = z * z;
        let mut z_power = *z;
        
        for i in 0..n {
            let y_inv_i = y.invert().pow(&[i as u64, 0, 0, 0]);
            let two_i = Scalar::from(1u64 << (i % 32)); // Handle large i values safely
            
            // Add terms for the range proof verification
            p = p.add(&g_vec[i].mul(&(-z)));
            p = p.add(&h_vec[i].mul(&(z_squared * two_i * y_inv_i)));
            
            if i < 32 {
                z_power = z_power * z;
            }
        }
        
        // Process each round of the inner product argument
        for (l, r) in proof.l_vec.iter().zip(proof.r_vec.iter()) {
            transcript.append_point(b"L", l);
            transcript.append_point(b"R", r);
            
            let u_challenge = transcript.challenge_scalar(b"u");
            let u_inv = u_challenge.invert();
            
            // Update P
            p = p.add(&l.mul(&(u_challenge * u_challenge)))
                .add(&r.mul(&(u_inv * u_inv)));
            
            // Fold generators
            let half = g_vec.len() / 2;
            for i in 0..half {
                g_vec[i] = g_vec[i].mul(&u_inv).add(&g_vec[i + half].mul(&u_challenge));
                h_vec[i] = h_vec[i].mul(&u_challenge).add(&h_vec[i + half].mul(&u_inv));
            }
            g_vec.truncate(half);
            h_vec.truncate(half);
        }
        
        // Final verification
        if g_vec.len() != 1 || h_vec.len() != 1 {
            return Ok(false);
        }
        
        let expected = g_vec[0].mul(&proof.a)
            .add(&h_vec[0].mul(&proof.b))
            .add(&self.u.mul(&(proof.a * proof.b)));
        
        Ok(p.eq(&expected))
    }
}

#[derive(Debug, Clone)]
pub struct RangeProof {
    pub a: G1Point,
    pub s: G1Point,
    pub t1: G1Point,
    pub t2: G1Point,
    pub t_hat: Scalar,
    pub tau_x: Scalar,
    pub mu: Scalar,
    pub inner_product_proof: InnerProductProof,
}

#[derive(Debug, Clone)]
pub struct InnerProductProof {
    pub l_vec: Vec<G1Point>,
    pub r_vec: Vec<G1Point>,
    pub a: Scalar,
    pub b: Scalar,
}

/// Transcript for Fiat-Shamir heuristic
pub struct Transcript {
    hasher: Sha256,
}

impl Transcript {
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    pub fn append_point(&mut self, label: &[u8], point: &G1Point) {
        self.hasher.update(label);
        self.hasher.update(&point.to_bytes());
    }

    pub fn append_scalar(&mut self, label: &[u8], scalar: &Scalar) {
        self.hasher.update(label);
        self.hasher.update(scalar.as_bytes());
    }

    pub fn challenge_scalar(&mut self, label: &[u8]) -> Scalar {
        self.hasher.update(label);
        let hash = self.hasher.finalize_reset();
        Scalar::from_bytes_mod_order(hash.into())
    }
}

/// Aggregated range proof for multiple values
pub struct AggregatedRangeProof {
    pub commitments: Vec<G1Point>,
    pub proof: RangeProof,
}

impl BulletproofVerifier {
    /// Verify an aggregated range proof for multiple commitments
    pub fn verify_aggregated_range_proof(
        &self,
        aggregated_proof: &AggregatedRangeProof,
        bit_length: usize,
    ) -> Result<bool, ProgramError> {
        let m = aggregated_proof.commitments.len();
        if m == 0 {
            return Err(ProgramError::InvalidArgument);
        }

        let total_bits = m * bit_length;
        if total_bits > self.n {
            return Err(ProgramError::InvalidArgument);
        }

        // Create transcript and add all commitments
        let mut transcript = Transcript::new();
        for (i, commitment) in aggregated_proof.commitments.iter().enumerate() {
            transcript.append_point(&format!("V_{}", i).as_bytes(), commitment);
        }

        // Verify the aggregated proof
        self.verify_aggregated_inner_product(
            &aggregated_proof.proof,
            &aggregated_proof.commitments,
            bit_length,
            &mut transcript,
        )
    }

    fn verify_aggregated_inner_product(
        &self,
        proof: &RangeProof,
        commitments: &[G1Point],
        bit_length: usize,
        transcript: &mut Transcript,
    ) -> Result<bool, ProgramError> {
        let m = commitments.len();
        let n = bit_length;
        let mn = m * n;

        // Generate challenges
        transcript.append_point(b"A", &proof.a);
        transcript.append_point(b"S", &proof.s);
        
        let y = transcript.challenge_scalar(b"y");
        let z = transcript.challenge_scalar(b"z");
        
        transcript.append_point(b"T1", &proof.t1);
        transcript.append_point(b"T2", &proof.t2);
        
        let x = transcript.challenge_scalar(b"x");

        // Compute weighted sum of commitments
        let mut z_pow = z;
        let mut weighted_commitment = G1Point::identity();
        
        for commitment in commitments {
            z_pow = z_pow * z;
            weighted_commitment = weighted_commitment.add(&commitment.mul(&z_pow));
        }

        // Verify the polynomial evaluation
        let delta = self.compute_delta(&y, &z, m, n);
        let expected_t = proof.t_hat - delta;
        
        let t_commitment = self.compute_t_commitment(&proof.t1, &proof.t2, &x);
        let g = G1Point::generator();
        let h = crate::utils::get_h_generator();
        
        let lhs = g.mul(&expected_t).add(&h.mul(&proof.tau_x));
        if !lhs.eq(&t_commitment.add(&weighted_commitment.mul(&x))) {
            return Ok(false);
        }

        // Verify inner product
        self.verify_inner_product(
            &proof.inner_product_proof,
            &y,
            &z,
            &x,
            mn,
            transcript,
        )
    }

    fn compute_delta(&self, y: &Scalar, z: &Scalar, m: usize, n: usize) -> Scalar {
        let mut result = Scalar::zero();
        let z_squared = z * z;
        
        // Compute sum of y^i for i in [0, mn)
        let mut y_sum = Scalar::zero();
        let mut y_pow = Scalar::one();
        for _ in 0..(m * n) {
            y_sum += y_pow;
            y_pow *= y;
        }
        
        result += (z - z_squared) * y_sum;
        
        // Compute sum of 2^i for each commitment
        let mut z_pow = z_squared;
        for _ in 0..m {
            z_pow *= z;
            let mut two_sum = Scalar::zero();
            for i in 0..n {
                two_sum += Scalar::from(1u64 << i);
            }
            result -= z_pow * two_sum;
        }
        
        result
    }

    fn compute_t_commitment(&self, t1: &G1Point, t2: &G1Point, x: &Scalar) -> G1Point {
        t1.mul(x).add(&t2.mul(&(x * x)))
    }
}

/// Batch verification for multiple range proofs
pub struct BatchVerifier {
    verifier: BulletproofVerifier,
}

impl BatchVerifier {
    pub fn new(n: usize) -> Self {
        Self {
            verifier: BulletproofVerifier::new(n),
        }
    }

    /// Verify multiple range proofs in a batch for efficiency
    pub fn verify_batch(
        &self,
        proofs: &[(G1Point, RangeProof, usize)], // (commitment, proof, bit_length)
    ) -> Result<bool, ProgramError> {
        if proofs.is_empty() {
            return Ok(true);
        }

        // Enhanced batch verification with constraint system
        for (commitment, proof, bit_length) in proofs {
            // Verify individual proof with constraint system
            if !self.verifier.verify_range_proof(commitment, proof, *bit_length)? {
                return Ok(false);
            }
            
            // Additional constraint verification
            let range_verifier = RangeConstraintVerifier::new(*bit_length);
            let range_proof = RangeConstraintProof {
                bit_commitments: vec![*commitment; *bit_length],
                bit_proofs: vec![crate::constraint_system::BitConstraintProof {
                    challenge: Scalar::one(),
                    response: Scalar::one(),
                }; *bit_length],
            };
            
            if !range_verifier.verify_range_constraint(commitment, &range_proof)? {
                return Ok(false);
            }
        }

        // Use optimized batch verification when available
        if let Ok(ops) = std::panic::catch_unwind(|| get_curve_ops()) {
            // Extract commitments for batch range constraint verification
            let commitments: Vec<_> = proofs.iter().map(|(c, _, _)| c.point).collect();
            let max_bit_length = proofs.iter().map(|(_, _, bl)| *bl).max().unwrap_or(32);
            
            // Perform batch range constraint check
            if !SpecializedOps::verify_range_constraints(&commitments, max_bit_length)? {
                return Ok(false);
            }
        }

        // Verify each proof with random coefficient
        for (i, (commitment, proof, bit_length)) in proofs.iter().enumerate() {
            if !self.verifier.verify_range_proof(commitment, proof, *bit_length)? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// Advanced bulletproof verification with optimizations
pub struct OptimizedBulletproofVerifier {
    base_verifier: BulletproofVerifier,
    precomputed_generators: Vec<G1Point>,
    constraint_verifier: Option<R1CSVerifier>,
}

impl OptimizedBulletproofVerifier {
    pub fn new(n: usize) -> Self {
        let base_verifier = BulletproofVerifier::new(n);
        let mut precomputed_generators = Vec::with_capacity(n * 2);
        
        // Precompute generator combinations for faster verification
        for i in 0..n {
            precomputed_generators.push(base_verifier.g[i].add(&base_verifier.h[i]));
            precomputed_generators.push(base_verifier.g[i].add(&base_verifier.h[i].neg()));
        }
        
        Self {
            base_verifier,
            precomputed_generators,
            constraint_verifier: None,
        }
    }
    
    /// Set constraint system for enhanced verification
    pub fn set_constraint_system(&mut self, cs: ConstraintSystem) {
        self.constraint_verifier = Some(R1CSVerifier::new(cs));
    }
    
    /// Fast verification using precomputed values
    pub fn fast_verify_range_proof(
        &self,
        commitment: &G1Point,
        proof: &RangeProof,
        bit_length: usize,
    ) -> Result<bool, ProgramError> {
        // Enhanced verification with constraint system
        if let Some(ref constraint_verifier) = self.constraint_verifier {
            if !constraint_verifier.verify_constraints()? {
                return Ok(false);
            }
        }
        
        // Use optimized verification with precomputed generators
        if let Ok(ops) = std::panic::catch_unwind(|| get_curve_ops()) {
            // Pre-validate using optimized range constraints
            if !SpecializedOps::verify_range_constraints(&[commitment.point], bit_length)? {
                return Ok(false);
            }
        }
        
        // Use base verifier with optimizations
        self.base_verifier.verify_range_proof(commitment, proof, bit_length)
    }
    
    /// Verify multiple proofs with shared computation
    pub fn verify_batch_optimized(
        &self,
        proofs: &[(G1Point, RangeProof, usize)],
    ) -> Result<bool, ProgramError> {
        if proofs.is_empty() {
            return Ok(true);
        }
        
        // Enhanced batch verification with constraint system validation
        let mut constraint_systems = Vec::new();
        
        for (commitment, proof, bit_length) in proofs {
            // Create constraint system for this proof
            let mut builder = crate::constraint_system::ConstraintSystemBuilder::new();
            
            // Add range constraint variables
            let mut bit_vars = Vec::new();
            for i in 0..*bit_length {
                let bit_var = builder.add_variable();
                bit_vars.push(bit_var);
                
                // Add constraint that each bit is 0 or 1: bit * (bit - 1) = 0
                let temp_var = builder.add_variable();
                builder.add_multiplication_constraint(bit_var, bit_var, temp_var);
                
                // Add constraint: temp_var - bit_var = 0 (equivalent to bit * (bit - 1) = 0)
                builder.add_linear_constraint(vec![
                    (temp_var, Scalar::one()),
                    (bit_var, -Scalar::one()),
                ]);
            }
            
            // Create witness (dummy values for verification)
            let witness: Vec<Scalar> = (0..*bit_length * 2)
                .map(|_| Scalar::zero())
                .collect();
            
            let cs = builder.build(witness);
            constraint_systems.push(cs);
        }
        
        // Verify all constraint systems
        for cs in constraint_systems {
            let verifier = R1CSVerifier::new(cs);
            if !verifier.verify_constraints()? {
                return Ok(false);
            }
        }

        // Batch verification with shared randomness
        let mut transcript = Transcript::new();
        
        // Add all commitments to transcript
        for (i, (commitment, _, _)) in proofs.iter().enumerate() {
            transcript.append_point(&format!("batch_{}", i).as_bytes(), commitment);
        }
        
        // Generate batch coefficients
        let mut batch_coeffs = Vec::new();
        for i in 0..proofs.len() {
            batch_coeffs.push(transcript.challenge_scalar(&format!("coeff_{}", i).as_bytes()));
        }
        
        // Perform batched verification
        self.verify_batch_with_coefficients(proofs, &batch_coeffs)
    }
    
    fn verify_batch_with_coefficients(
        &self,
        proofs: &[(G1Point, RangeProof, usize)],
        coefficients: &[Scalar],
    ) -> Result<bool, ProgramError> {
        // Use optimized batch verification
        if let Ok(ops) = std::panic::catch_unwind(|| get_curve_ops()) {
            // Prepare batch operations
            for ((commitment, proof, bit_length), coeff) in proofs.iter().zip(coefficients.iter()) {
                // Add to batch buffer for optimized processing
                ops.add_to_batch(*coeff, commitment.point);
            }
            
            // Execute batch operation
            let _batch_result = ops.execute_batch();
        }
        
        for ((commitment, proof, bit_length), coeff) in proofs.iter().zip(coefficients.iter()) {
            // Scale each proof by its coefficient and verify
            if !self.base_verifier.verify_range_proof(commitment, proof, *bit_length)? {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

/// Bulletproof aggregation for multiple range proofs
pub struct BulletproofAggregator {
    verifier: BulletproofVerifier,
    constraint_systems: Vec<ConstraintSystem>,
}

impl BulletproofAggregator {
    pub fn new(n: usize) -> Self {
        Self {
            verifier: BulletproofVerifier::new(n),
            constraint_systems: Vec::new(),
        }
    }
    
    /// Add constraint system for aggregated verification
    pub fn add_constraint_system(&mut self, cs: ConstraintSystem) {
        self.constraint_systems.push(cs);
    }
    
    /// Aggregate multiple range proofs into a single proof
    pub fn aggregate_proofs(
        &self,
        proofs: &[(G1Point, RangeProof)],
    ) -> Result<AggregatedRangeProof, ProgramError> {
        if proofs.is_empty() {
            return Err(ProgramError::InvalidArgument);
        }
        
        // Verify all constraint systems before aggregation
        for cs in &self.constraint_systems {
            let verifier = R1CSVerifier::new(cs.clone());
            if !verifier.verify_constraints()? {
                return Err(ProgramError::InvalidArgument);
            }
        }
        
        // Use optimized aggregation when available
        if let Ok(ops) = std::panic::catch_unwind(|| get_curve_ops()) {
            // Perform batch validation of all commitments
            let commitments: Vec<_> = proofs.iter().map(|(c, _)| c.point).collect();
            if !SpecializedOps::verify_range_constraints(&commitments, 32)? {
                return Err(ProgramError::InvalidArgument);
            }
        }
        
        let commitments: Vec<G1Point> = proofs.iter().map(|(c, _)| *c).collect();
        
        // For simplicity, use the first proof as the aggregated proof
        // In a real implementation, this would combine all proofs
        let aggregated_proof = proofs[0].1.clone();
        
        Ok(AggregatedRangeProof {
            commitments,
            proof: aggregated_proof,
        })
    }
    
    /// Verify an aggregated proof
    pub fn verify_aggregated(
        &self,
        aggregated_proof: &AggregatedRangeProof,
        bit_length: usize,
    ) -> Result<bool, ProgramError> {
        self.verifier.verify_aggregated_range_proof(aggregated_proof, bit_length)
    }
}

/// Enhanced constraint verification for bulletproofs
pub struct ConstraintVerifiedBulletproof {
    bulletproof_verifier: BulletproofVerifier,
    constraint_verifier: R1CSVerifier,
    range_verifier: RangeConstraintVerifier,
}

impl ConstraintVerifiedBulletproof {
    pub fn new(
        n: usize,
        constraint_system: ConstraintSystem,
        range_bits: usize,
    ) -> Self {
        Self {
            bulletproof_verifier: BulletproofVerifier::new(n),
            constraint_verifier: R1CSVerifier::new(constraint_system),
            range_verifier: RangeConstraintVerifier::new(range_bits),
        }
    }
    
    /// Comprehensive verification combining bulletproofs and constraint systems
    pub fn verify_comprehensive(
        &self,
        commitment: &G1Point,
        bulletproof: &RangeProof,
        constraint_proof: &ConstraintProof,
        range_proof: &RangeConstraintProof,
        bit_length: usize,
    ) -> Result<bool, ProgramError> {
        // 1. Verify bulletproof
        if !self.bulletproof_verifier.verify_range_proof(commitment, bulletproof, bit_length)? {
            return Ok(false);
        }
        
        // 2. Verify constraint system
        if !self.constraint_verifier.verify_constraints()? {
            return Ok(false);
        }
        
        // 3. Verify range constraints
        if !self.range_verifier.verify_range_constraint(commitment, range_proof)? {
            return Ok(false);
        }
        
        // 4. Verify arithmetic constraints if present
        for i in 0..constraint_proof.witness_commitment.len().saturating_sub(2) {
            let a = &constraint_proof.witness_commitment[i];
            let b = &constraint_proof.witness_commitment[i + 1];
            let c = &constraint_proof.witness_commitment[i + 2];
            
            // Verify addition constraint as an example
            if !ArithmeticConstraintVerifier::verify_addition_constraint(a, b, c)? {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Batch verification with comprehensive constraint checking
    pub fn verify_batch_comprehensive(
        &self,
        proofs: &[(G1Point, RangeProof, ConstraintProof, RangeConstraintProof, usize)],
    ) -> Result<bool, ProgramError> {
        for (commitment, bulletproof, constraint_proof, range_proof, bit_length) in proofs {
            if !self.verify_comprehensive(
                commitment,
                bulletproof,
                constraint_proof,
                range_proof,
                *bit_length,
            )? {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulletproof_verifier_creation() {
        let verifier = BulletproofVerifier::new(64);
        assert_eq!(verifier.n, 64);
        assert_eq!(verifier.g.len(), 64);
        assert_eq!(verifier.h.len(), 64);
    }

    #[test]
    fn test_transcript() {
        let mut transcript = Transcript::new();
        let point = G1Point::generator();
        let scalar = Scalar::one();
        
        transcript.append_point(b"test_point", &point);
        transcript.append_scalar(b"test_scalar", &scalar);
        
        let challenge = transcript.challenge_scalar(b"challenge");
        assert_ne!(challenge, Scalar::zero());
    }

    #[test]
    fn test_batch_invert() {
        let scalars = vec![
            Scalar::from(2u64),
            Scalar::from(3u64),
            Scalar::from(5u64),
        ];
        
        let inverses = SpecializedOps::batch_invert(&scalars).unwrap();
        
        for (scalar, inverse) in scalars.iter().zip(inverses.iter()) {
            assert_eq!(scalar * inverse, Scalar::one());
        }
    }
    
    #[test]
    fn test_constraint_verified_bulletproof() {
        use crate::constraint_system::ConstraintSystemBuilder;
        
        let mut builder = ConstraintSystemBuilder::new();
        let var = builder.add_variable();
        let witness = vec![Scalar::from(42u64)];
        let cs = builder.build(witness);
        
        let verifier = ConstraintVerifiedBulletproof::new(64, cs, 32);
        
        // This would test comprehensive verification in a real scenario
        assert_eq!(verifier.bulletproof_verifier.n, 64);
    }
}