use crate::primitives::types::{Hash, Slot};
use crate::core::node::Node;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Precommit {
    pub target_hash: Hash,
    pub target_height: u64,
    pub voter_id: String,
    pub slot: Slot,
}

impl Precommit {
    /// A precommit is considered valid for a node if the target block is currently 
    /// part of that node's known ancestry (i.e. it is an ancestor of the node's best head).
    pub fn is_valid_for_node(&self, node: &Node) -> bool {
        // If the node doesn't even have the block, it can't validate it yet.
        if !node.blocks.contains_key(&self.target_hash) {
            return false;
        }

        // Standard ancestry check: common ancestor must be the target itself.
        node.find_common_ancestor(node.best_head_hash, self.target_hash) == self.target_hash
    }
}
