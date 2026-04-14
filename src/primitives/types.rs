use parity_scale_codec::{Encode, Decode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A 32-byte hash identifying protocol entities.
///
/// Uses Blake3 for performance and collision resistance. 
/// SCALE-compatible for wire-ready transmission.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, TypeInfo, Serialize, Deserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn zero() -> Self {
        Hash([0u8; 32])
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Hash(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = hex::encode(self.0);
        write!(f, "{}..{}", &s[0..6], &s[s.len() - 4..])
    }
}

pub type Slot = u64;
pub type BlockNumber = u64;

/// The block header, representing the cryptographic link in the chain.
///
/// INVARIANT: The `state_root` must represent the Post-Execution state 
/// of the world after all extrinsics in the block are processed.
#[derive(Debug, Clone, Encode, Decode, TypeInfo, Serialize, Deserialize, PartialEq, Eq)]
pub struct Header {
    /// Hash of the immediate parent block.
    pub parent_hash: Hash,
    /// Absolute height of the block.
    #[codec(compact)]
    pub number: BlockNumber,
    /// Cryptographic commitment to the post-execution state.
    pub state_root: Hash,
    /// Commitment to the set of extrinsics contained in the body.
    pub extrinsics_root: Hash,
    /// The discrete time unit this block claims to be authored in.
    pub slot: Slot,
    /// Identifying string for authorship. 
    /// // TODO: Replace with AccountId32 and Sr25519 signature for production fidelity.
    pub author: String,
}

impl Header {
    /// Compute the Blake3 hash of the SCALE-encoded header.
    pub fn hash(&self) -> Hash {
        let bytes = self.encode();
        Hash::from_bytes(blake3::hash(&bytes).into())
    }
}

/// Domain-specific transactions (extrinsics).
///
/// Includes both state-mutating system calls and simple transfers.
#[derive(Debug, Clone, Encode, Decode, TypeInfo, Serialize, Deserialize, PartialEq, Eq)]
pub enum Extrinsic {
    Transfer { from: String, to: String, amount: u64 },
    SetState { key: Vec<u8>, value: Vec<u8> },
}

/// A protocol-level container bundling proof (header) and data (body).
#[derive(Debug, Clone, Encode, Decode, TypeInfo, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    pub header: Header,
    pub extrinsics: Vec<Extrinsic>,
}

impl Block {
    pub fn hash(&self) -> Hash {
        self.header.hash()
    }
}

/// Hex utilities for protocol diagnostics.
mod hex {
    pub fn encode(bytes: [u8; 32]) -> String {
        let mut s = String::with_capacity(64);
        for byte in bytes {
            s.push_str(&format!("{:02x}", byte));
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_scale_codec::{Encode, Decode};

    #[test]
    fn test_header_scale_codec() {
        let header = Header {
            parent_hash: Hash::zero(),
            number: 42,
            state_root: Hash::from_bytes([1u8; 32]),
            extrinsics_root: Hash::from_bytes([2u8; 32]),
            slot: 100,
            author: "alice".to_string(),
        };

        let encoded = header.encode();
        let decoded = Header::decode(&mut &encoded[..]).unwrap();
        assert_eq!(header, decoded);
    }

    #[test]
    fn test_header_hashing_determinism() {
        let h1 = Header {
            parent_hash: Hash::zero(),
            number: 1,
            state_root: Hash::zero(),
            extrinsics_root: Hash::zero(),
            slot: 1,
            author: "validator_1".to_string(),
        };
        let h2 = h1.clone();
        assert_eq!(h1.hash(), h2.hash());
        
        let mut h3 = h1.clone();
        h3.author = "validator_2".to_string();
        assert_ne!(h1.hash(), h3.hash());
    }
}
