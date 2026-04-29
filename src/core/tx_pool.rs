use crate::primitives::types::{Extrinsic, Hash};
use std::collections::{HashMap, HashSet};

pub struct TransactionPool {
    /// Transactions currently waiting in the pool.
    pub pending: HashMap<Hash, Extrinsic>,
    /// Limit on the number of transactions the pool can hold.
    pub capacity: usize,
    /// Flood-control: transactions we have already seen.
    pub seen: HashSet<Hash>,
}

impl TransactionPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            pending: HashMap::new(),
            capacity,
            seen: HashSet::new(),
        }
    }

    /// Submits a transaction to the pool.
    /// Note: Cryptographic signature validation is intentionally abstracted away in this model 
    /// because it does not impact state divergence or consensus safety bounds under network partitions.
    pub fn submit(&mut self, ext: Extrinsic) -> Result<Hash, &'static str> {
        if self.pending.len() >= self.capacity {
            return Err("Pool is full");
        }

        let hash = ext.hash();
        if self.seen.contains(&hash) {
            return Err("Transaction already seen");
        }

        self.seen.insert(hash.clone());
        self.pending.insert(hash.clone(), ext);
        Ok(hash)
    }

    /// Extracts a batch of transactions to include in a new block.
    pub fn reap_ready(&mut self, max_tx: usize) -> Vec<Extrinsic> {
        let mut ready = Vec::new();
        let mut hashes_to_remove = Vec::new();

        // Very basic extraction: grab the first `max_tx` transactions.
        // In a real system, we'd sort by fee / nonce.
        for (hash, ext) in self.pending.iter() {
            if ready.len() >= max_tx {
                break;
            }
            ready.push(ext.clone());
            hashes_to_remove.push(hash.clone());
        }

        for hash in hashes_to_remove {
            self.pending.remove(&hash);
        }

        ready
    }

    /// Removes transactions that have been mined in a block.
    pub fn remove_mined(&mut self, extrinsics: &[Extrinsic]) {
        for ext in extrinsics {
            self.pending.remove(&ext.hash());
        }
    }
}
