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
        }
    }

    /// Integrates a block into the local DAG.
    ///
    /// Returns `true` only for previously unobserved blocks. 
    /// INVARIANT: Every import triggers a potential tip re-evaluation.
    pub fn import_block(&mut self, block: Block) -> bool {
        let hash = block.hash();
        if self.seen_blocks.contains(&hash) {
            return false;
        }

        self.seen_blocks.insert(hash);
        self.blocks.insert(hash, block);
        self.reorg_chain();
        true
    }

    /// Evaluates canonical head based on fork-choice rules.
    fn reorg_chain(&mut self) {
        let headers: Vec<Header> = self.blocks.values().map(|b| b.header.clone()).collect();
        if let Some(best) = self.consensus.find_best_head(&headers) {
            self.best_head_hash = best.hash();
        }
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
            
            Some(block)
        } else {
            None
        }
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
