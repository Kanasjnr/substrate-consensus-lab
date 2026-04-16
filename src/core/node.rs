use crate::primitives::types::{Block, Hash, Header, Slot};
use crate::core::runtime::Runtime;
use crate::core::consensus::Consensus;
use std::collections::{HashMap, HashSet};

pub struct Node {
    pub id: String,
    pub runtime: Runtime,
    pub consensus: Consensus,
    pub blocks: HashMap<Hash, Block>,
    /// SAFETY: primary flood-control invariant for P2P gossip.
    pub seen_blocks: HashSet<Hash>,
    pub best_head_hash: Hash,
    pub proposed_blocks: u64,
    pub imported_blocks: u64,
}

impl Node {
    pub fn new(id: String, threshold: u64) -> Self {
        let mut blocks = HashMap::new();
        let mut seen_blocks = HashSet::new();
        
        // Protocol genesis.
        let genesis_header = Header {
            parent_hash: Hash::zero(),
            number: 0,
            state_root: Hash::zero(),
            extrinsics_root: Hash::zero(),
            slot: 0,
            author: "genesis".to_string(),
        };
        let genesis_block = Block {
            header: genesis_header,
            extrinsics: vec![],
        };
        let genesis_hash = genesis_block.hash();
        blocks.insert(genesis_hash, genesis_block);
        seen_blocks.insert(genesis_hash);

        Self {
            id: id.clone(),
            runtime: Runtime::new(),
            consensus: Consensus::new(id, threshold),
            blocks,
            seen_blocks,
            best_head_hash: genesis_hash,
            proposed_blocks: 0,
            imported_blocks: 0,
        }
    }

    /// Integrates a block into the local DAG.
    ///
    /// Returns `Some(reorg_depth)` when a chain switch occurs (blocks discarded),
    /// `None` if the block was already seen or caused no head change.
    /// INVARIANT: Every import triggers a potential tip re-evaluation.
    pub fn import_block(&mut self, block: Block) -> Option<u64> {
        let hash = block.hash();
        if self.seen_blocks.contains(&hash) {
            return None;
        }

        self.seen_blocks.insert(hash);
        self.blocks.insert(hash, block);
        self.imported_blocks += 1;
        self.reorg_chain()
    }

    /// Evaluates canonical head based on fork-choice rules.
    ///
    /// Returns `Some(reorg_depth)` only on a genuine branch switch (re-org):
    /// when the new best head is NOT a direct descendant of the previous best.
    /// Returns `None` for normal chain extension or no change.
    fn reorg_chain(&mut self) -> Option<u64> {
        let old_hash = self.best_head_hash;
        let old_height = self.best_height();
        let headers: Vec<Header> = self.blocks.values().map(|b| b.header.clone()).collect();
        if let Some(best) = self.consensus.find_best_head(&headers) {
            self.best_head_hash = best.hash();
        }
        if self.best_head_hash == old_hash {
            return None; // No head change.
        }

        // Determine whether the switch is a simple extension (parent == old_hash)
        // or a genuine branch switch (re-org).
        let is_direct_extension = self.blocks
            .get(&self.best_head_hash)
            .map(|b| b.header.parent_hash == old_hash)
            .unwrap_or(false);

        if is_direct_extension {
            None
        } else {
            // Genuine branch switch: calculate depth to common ancestor.
            let ancestor_hash = self.find_common_ancestor(old_hash, self.best_head_hash);
            let ancestor_height = self.blocks.get(&ancestor_hash)
                .map(|b| b.header.number)
                .unwrap_or(0);
            
            // Depth is the number of blocks on the old chain that were discarded.
            Some(old_height.saturating_sub(ancestor_height))
        }
    }

    /// Walks back the DAG starting from two hashes until a common ancestor is reached.
    /// INVARIANT: Genesis is the ultimate fallback (parent_hash == Hash::zero()).
    fn find_common_ancestor(&self, mut h1: Hash, mut h2: Hash) -> Hash {
        let mut path1 = vec![h1];
        let mut path2 = vec![h2];

        // Walk h1 back to genesis
        while h1 != Hash::zero() {
            if let Some(b) = self.blocks.get(&h1) {
                h1 = b.header.parent_hash;
                path1.push(h1);
            } else {
                break;
            }
        }

        // Walk h2 back to genesis
        while h2 != Hash::zero() {
            if let Some(b) = self.blocks.get(&h2) {
                h2 = b.header.parent_hash;
                path2.push(h2);
            } else {
                break;
            }
        }

        // Find first intersection
        let set1: HashSet<Hash> = path1.into_iter().collect();
        for hash in path2 {
            if set1.contains(&hash) {
                return hash;
            }
        }

        Hash::zero() // Should never happen in a connected DAG.
    }

    /// Authors a new block candidate for the target slot.
    ///
    /// Requires slot claim rights from the consensus engine.
    pub fn propose_block(&mut self, slot: Slot, randomness: [u8; 32]) -> Option<Block> {
        if self.consensus.claim_slot(slot, randomness) {
            let parent = self.blocks.get(&self.best_head_hash)
                .expect("INVARIANT: Canonical head must exist in block database");
            
            let header = Header {
                parent_hash: self.best_head_hash,
                number: parent.header.number + 1,
                state_root: self.runtime.state.root(),
                extrinsics_root: Hash::zero(),
                slot,
                author: self.id.clone(),
            };

            let block = Block {
                header,
                extrinsics: vec![],
            };
            
            let hash = block.hash();
            self.blocks.insert(hash, block.clone());
            self.seen_blocks.insert(hash);
            self.best_head_hash = hash;
            self.proposed_blocks += 1;
            
            Some(block)
        } else {
            None
        }
    }

    /// Returns the height of the current canonical head.
    pub fn best_height(&self) -> u64 {
        self.blocks.get(&self.best_head_hash)
            .map(|b| b.header.number)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seen_blocks_flood_protection() {
        let mut node = Node::new("node_0".to_string(), u64::MAX);
        
        let header = Header {
            parent_hash: Hash::zero(),
            number: 1,
            state_root: Hash::zero(),
            extrinsics_root: Hash::zero(),
            slot: 1,
            author: "A".to_string(),
        };
        let block = Block { header, extrinsics: vec![] };

        assert!(node.import_block(block.clone()));
        assert!(node.seen_blocks.contains(&block.hash()));
        assert!(!node.import_block(block));
    }

    #[test]
    fn test_genesis_already_seen() {
        let node = Node::new("node_0".to_string(), u64::MAX);
        
        let genesis_header = Header {
            parent_hash: Hash::zero(),
            number: 0,
            state_root: Hash::zero(),
            extrinsics_root: Hash::zero(),
            slot: 0,
            author: "genesis".to_string(),
        };
        let genesis_block = Block { header: genesis_header, extrinsics: vec![] };
        
        assert!(node.seen_blocks.contains(&genesis_block.hash()));
    }
}
