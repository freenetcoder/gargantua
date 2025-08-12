use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_POINT,
    ristretto::{RistrettoPoint, CompressedRistretto},
    scalar::Scalar,
};
use sha2::{Digest, Sha256};
use solana_program::program_error::ProgramError;

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
        G1Point {
            point: self.point + other.point,
        }
    }

    pub fn mul(&self, scalar: &Scalar) -> G1Point {
        G1Point {
            point: self.point * scalar,
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
    let g = G1Point::generator();
    let h = get_h_generator();
    g.mul(value).add(&h.mul(blinding))
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