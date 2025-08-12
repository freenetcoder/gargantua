use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_POINT,
    ristretto::{RistrettoPoint, CompressedRistretto},
    scalar::Scalar,
    traits::VartimeMultiscalarMul,
};
use sha2::{Digest, Sha256};
use solana_program::program_error::ProgramError;
use crate::curve_ops::{get_curve_ops, get_precomputed_constants, init_curve_ops};

pub const GROUP_ORDER: [u8; 32] = [
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x14, 0xde, 0xf9, 0xde, 0xa2, 0xf7, 0x9c, 0xd6,
    0x58, 0x12, 0x63, 0x1a, 0x5c, 0xf5, 0xd3, 0xed,
];

pub const MAX_TRANSFER_AMOUNT: u64 = 4294967295; // 2^32 - 1

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct G1Point {
    pub point: RistrettoPoint,
}

impl G1Point {
    pub fn new(point: RistrettoPoint) -> Self {
        Self { point }
    }

    pub fn generator() -> Self {
        Self {
            point: RISTRETTO_BASEPOINT_POINT,
        }
    }

    pub fn identity() -> Self {
        Self {
            point: RistrettoPoint::default(),
        }
    }

    pub fn add(&self, other: &G1Point) -> G1Point {
        // Use optimized operations when available
        if let Ok(ops) = std::panic::catch_unwind(|| get_curve_ops()) {
            G1Point {
                point: ops.cached_point_add(&self.point, &other.point),
            }
        } else {
            G1Point {
                point: self.point + other.point,
            }
        }
    }

    pub fn mul(&self, scalar: &Scalar) -> G1Point {
        // Use optimized scalar multiplication when available
        if let Ok(ops) = std::panic::catch_unwind(|| get_curve_ops()) {
            G1Point {
                point: ops.fast_scalar_mul(&self.point, scalar),
            }
        } else {
            G1Point {
                point: self.point * scalar,
            }
        }
    }

    pub fn neg(&self) -> G1Point {
        G1Point {
            point: -self.point,
        }
    }

    pub fn eq(&self, other: &G1Point) -> bool {
        self.point == other.point
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.point.compress().to_bytes()
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, ProgramError> {
        let compressed = CompressedRistretto::from_slice(bytes)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        let point = compressed
            .decompress()
            .ok_or(ProgramError::InvalidAccountData)?;
        Ok(G1Point { point })
    }
}

pub fn scalar_from_bytes(bytes: &[u8; 32]) -> Scalar {
    Scalar::from_bytes_mod_order(*bytes)
}

pub fn scalar_add(a: &Scalar, b: &Scalar) -> Scalar {
    a + b
}

pub fn scalar_sub(a: &Scalar, b: &Scalar) -> Scalar {
    a - b
}

pub fn scalar_mul(a: &Scalar, b: &Scalar) -> Scalar {
    a * b
}

pub fn scalar_inv(a: &Scalar) -> Scalar {
    a.invert()
}

pub fn scalar_neg(a: &Scalar) -> Scalar {
    -a
}

pub fn hash_to_scalar(data: &[u8]) -> Scalar {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    Scalar::from_bytes_mod_order(hash.into())
}

pub fn map_to_curve(seed: &[u8]) -> G1Point {
    let mut hasher = Sha256::new();
    hasher.update(seed);
    let hash = hasher.finalize();
    let scalar = Scalar::from_bytes_mod_order(hash.into());
    G1Point {
        point: RISTRETTO_BASEPOINT_POINT * scalar,
    }
}

pub fn map_to_curve_with_index(input: &str, index: u64) -> G1Point {
    let mut data = input.as_bytes().to_vec();
    data.extend_from_slice(&index.to_le_bytes());
    map_to_curve(&data)
}

// Pedersen commitment: g^value * h^blinding
pub fn pedersen_commit(value: &Scalar, blinding: &Scalar) -> G1Point {
    // Use optimized Pedersen commitment when available
    if let Ok(ops) = std::panic::catch_unwind(|| get_curve_ops()) {
        G1Point {
            point: ops.pedersen_commit(value, blinding),
        }
    } else {
        let g = G1Point::generator();
        let h = get_h_generator();
        g.mul(value).add(&h.mul(blinding))
    }
}

pub fn get_h_generator() -> G1Point {
    // Use a different generator point for h
    let h_bytes = [
        0x2b, 0xda, 0x7d, 0x3a, 0xe6, 0xa5, 0x57, 0xc7,
        0x16, 0x47, 0x7c, 0x10, 0x8b, 0xe0, 0xd0, 0xf9,
        0x4a, 0xbc, 0x6c, 0x4d, 0xc6, 0xb1, 0xbd, 0x93,
        0xca, 0xcc, 0xbc, 0xce, 0xaa, 0xa7, 0x1d, 0x6b,
    ];
    G1Point::from_bytes(&h_bytes).unwrap()
}

pub fn verify_schnorr_signature(
    public_key: &G1Point,
    message: &[u8],
    challenge: &Scalar,
    response: &Scalar,
) -> bool {
    let g = G1Point::generator();
    let k = g.mul(response).add(&public_key.mul(&scalar_neg(challenge)));
    
    let mut hasher = Sha256::new();
    hasher.update(message);
    hasher.update(&public_key.to_bytes());
    hasher.update(&k.to_bytes());
    let computed_challenge = hash_to_scalar(&hasher.finalize());
    
    computed_challenge == *challenge
}

/// Multi-scalar multiplication for efficient bulletproof verification
pub fn multi_scalar_mul(scalars: &[Scalar], points: &[G1Point]) -> G1Point {
    assert_eq!(scalars.len(), points.len());
    
    // Use optimized multi-scalar multiplication when available
    if let Ok(ops) = std::panic::catch_unwind(|| get_curve_ops()) {
        let ristretto_points: Vec<RistrettoPoint> = points.iter().map(|p| p.point).collect();
        match ops.linear_combination(scalars, &ristretto_points) {
            Ok(result_point) => G1Point { point: result_point },
            Err(_) => {
                // Fallback to standard implementation
                let mut result = G1Point::identity();
                for (scalar, point) in scalars.iter().zip(points.iter()) {
                    result = result.add(&point.mul(scalar));
                }
                result
            }
        }
    } else {
        let mut result = G1Point::identity();
        for (scalar, point) in scalars.iter().zip(points.iter()) {
            result = result.add(&point.mul(scalar));
        }
        result
    }
}

/// Compute inner product of two scalar vectors
pub fn inner_product(a: &[Scalar], b: &[Scalar]) -> Scalar {
    assert_eq!(a.len(), b.len());
    
    let mut result = Scalar::zero();
    for (ai, bi) in a.iter().zip(b.iter()) {
        result += ai * bi;
    }
    result
}

/// Generate powers of a scalar: [1, x, x^2, ..., x^(n-1)]
pub fn scalar_powers(x: &Scalar, n: usize) -> Vec<Scalar> {
    // Use precomputed powers of 2 when possible
    if *x == Scalar::from(2u64) {
        if let Ok(constants) = std::panic::catch_unwind(|| get_precomputed_constants()) {
            let mut powers = Vec::with_capacity(n);
            for i in 0..n {
                if let Some(power) = constants.power_of_two(i) {
                    powers.push(power);
                } else {
                    // Fallback for large powers
                    let mut current = Scalar::one();
                    for _ in 0..i {
                        current = current + current;
                    }
                    powers.push(current);
                }
            }
            return powers;
        }
    }
    
    // Standard implementation
    let mut powers = Vec::with_capacity(n);
    let mut current = Scalar::one();
    
    for _ in 0..n {
        powers.push(current);
        current *= x;
    }
    
    powers
}

/// Hadamard product of two scalar vectors
pub fn hadamard_product(a: &[Scalar], b: &[Scalar]) -> Vec<Scalar> {
    assert_eq!(a.len(), b.len());
    
    a.iter().zip(b.iter()).map(|(ai, bi)| ai * bi).collect()
}

/// Vector addition for scalars
pub fn vector_add(a: &[Scalar], b: &[Scalar]) -> Vec<Scalar> {
    assert_eq!(a.len(), b.len());
    
    a.iter().zip(b.iter()).map(|(ai, bi)| ai + bi).collect()
}

/// Vector subtraction for scalars
pub fn vector_sub(a: &[Scalar], b: &[Scalar]) -> Vec<Scalar> {
    assert_eq!(a.len(), b.len());
    
    a.iter().zip(b.iter()).map(|(ai, bi)| ai - bi).collect()
}

/// Scalar multiplication of a vector
pub fn vector_scalar_mul(v: &[Scalar], s: &Scalar) -> Vec<Scalar> {
    // Use precomputed small scalars when possible
    if let Ok(constants) = std::panic::catch_unwind(|| get_precomputed_constants()) {
        if let Some(small_s) = (0..16).find(|&i| constants.small_scalar(i).map_or(false, |sc| sc == *s)) {
            if small_s == 0 {
                return vec![Scalar::zero(); v.len()];
            } else if small_s == 1 {
                return v.to_vec();
            }
        }
    }
    
    v.iter().map(|vi| vi * s).collect()
}

/// Optimized batch scalar multiplication
pub fn batch_scalar_mul(scalars: &[Scalar], points: &[G1Point]) -> Vec<G1Point> {
    assert_eq!(scalars.len(), points.len());
    
    if let Ok(ops) = std::panic::catch_unwind(|| get_curve_ops()) {
        // Use batch operations for better performance
        let mut results = Vec::with_capacity(scalars.len());
        
        for (scalar, point) in scalars.iter().zip(points.iter()) {
            results.push(G1Point {
                point: ops.fast_scalar_mul(&point.point, scalar),
            });
        }
        
        results
    } else {
        // Fallback to standard implementation
        scalars.iter()
            .zip(points.iter())
            .map(|(scalar, point)| point.mul(scalar))
            .collect()
    }
}

/// Initialize optimized curve operations
pub fn init_optimized_curve_ops() {
    init_curve_ops();
}