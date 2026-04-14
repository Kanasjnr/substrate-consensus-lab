use crate::primitives::types::{Block, Hash, Header, Slot};
use crate::core::runtime::Runtime;
use crate::core::consensus::Consensus;
use std::collections::HashMap;

/// A protocol-grade node actor responsible for chain synchronization and block production.
pub struct Node {
    pub id: String,
    pub runtime: Runtime,
    pub consensus: Consensus,
    /// Local database of all observed blocks, indexed by their cryptographic hash.
    pub blocks: HashMap<Hash, Block>,
    /// The current canonical head (tip) of the longest chain.
    pub best_head_hash: Hash,
}

impl Node {
    pub fn new(id: String, threshold: u64) -> Self {
        let mut blocks = HashMap::new();
        
        // Initialize the chain with a unique genesis block.
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

        Self {
            id: id.clone(),
            runtime: Runtime::new(),
            consensus: Consensus::new(id, threshold),
            blocks,
            best_head_hash: genesis_hash,
        }
    }

    /// Processes an incoming block from the network and attempts to integrate it into the local DAG.
    ///
    /// // TODO: Add signature and state-root verification before insertion.
    pub fn import_block(&mut self, block: Block) {
        let hash = block.hash();
        if self.blocks.contains_key(&hash) {
            return;
        }
        self.blocks.insert(hash, block);
        self.reorg_chain();
    }

    /// Evaluates the known block set and updates the canonical head based on the fork-choice rule.
    fn reorg_chain(&mut self) {
        let headers: Vec<Header> = self.blocks.values().map(|b| b.header.clone()).collect();
        if let Some(best) = self.consensus.find_best_head(&headers) {
            self.best_head_hash = best.hash();
        }
    }

    /// Attempts to author a new block candidate for the current slot.
    ///
    /// IMPLEMENTATION: Checks slot ownership via the consensus engine. 
    /// If successful, seals a new block on top of the current canonical head.
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
            self.best_head_hash = hash;
            
            Some(block)
        } else {
            None
        }
    }
}
