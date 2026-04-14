use crate::primitives::types::{Header, Slot};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

pub struct Consensus {
    pub validator_id: String,
    pub threshold: u64,
}

impl Consensus {
    pub fn new(validator_id: String, threshold: u64) -> Self {
        Self {
            validator_id,
            threshold,
        }
    }

    /// Probabilistic slot leadership lottery.
    ///
    /// NOTE: Simplified VRF approximation for research purposes.
    pub fn claim_slot(&self, slot: Slot, randomness: [u8; 32]) -> bool {
        let mut seed = [0u8; 32];
        let id_bytes = self.validator_id.as_bytes();
        let len = id_bytes.len().min(16);
        seed[..len].copy_from_slice(&id_bytes[..len]);
        seed[16..].copy_from_slice(&randomness[..16]);

        let mut rng = ChaCha20Rng::from_seed(seed);
        let val: u64 = rng.r#gen();
        val < self.threshold
    }

    /// Selects the canonical tip using the Longest Chain rule.
    ///
    /// TIE-BREAKING: Favors earlier slots for height-identical forks.
    pub fn find_best_head<'a>(&self, headers: &'a [Header]) -> Option<&'a Header> {
        headers.iter().max_by_key(|h| (h.number, -(h.slot as i64)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::types::Hash;

    #[test]
    fn test_longest_chain_rule() {
        let consensus = Consensus::new("test".to_string(), 0);

        let h1 = Header {
            parent_hash: Hash::zero(),
            number: 1,
            state_root: Hash::zero(),
            extrinsics_root: Hash::zero(),
            slot: 1,
            author: "a".to_string(),
        };

        let h2 = Header {
            parent_hash: Hash::zero(),
            number: 2,
            state_root: Hash::zero(),
            extrinsics_root: Hash::zero(),
            slot: 2,
            author: "b".to_string(),
        };

        let headers = vec![h1.clone(), h2.clone()];
        let best = consensus.find_best_head(&headers).unwrap();
        assert_eq!(best.number, 2);
    }
}
