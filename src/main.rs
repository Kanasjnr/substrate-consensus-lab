pub mod primitives;
pub mod core;
pub mod network;

use crate::core::node::Node;
use crate::network::network::{NetworkSimulator, Message};

fn main() {
    env_logger::init();
    log::info!("--- Substrate Consensus Lab: Experimental P2P Simulation ---");

    // Simulation Parameters
    let total_slots = 20;
    let hop_latency = 1; 
    let consensus_threshold = u64::MAX / 3;

    let mut simulator_network = NetworkSimulator::new(hop_latency);
    
    // Topology Setup: Line (0 --- 1 --- 2)
    let node_ids = vec!["node_0".to_string(), "node_1".to_string(), "node_2".to_string()];
    for id in &node_ids {
        simulator_network.register_node(id.clone());
    }
    simulator_network.add_neighbor("node_0", "node_1");
    simulator_network.add_neighbor("node_1", "node_2");

    let mut nodes: Vec<Node> = node_ids.into_iter()
        .map(|id| Node::new(id, consensus_threshold))
        .collect();

    let mut randomness = [0u8; 32];

    // Discrete-Event Simulation Loop
    for slot in 1..=total_slots {
        log::info!("---------------- Slot {} ----------------", slot);
        
        randomness[0] = (slot % 256) as u8;

        for node in nodes.iter_mut() {
            // 1. Ingress: Poll network for arrived messages
            let messages = simulator_network.poll_ingress(&node.id, slot);
            for msg in messages {
                match msg {
                    Message::Block(b) => {
                        let hash = b.hash();
                        // import_block returns true only if the block is new to this node.
                        if node.import_block(b.clone()) {
                            log::debug!("[{}] 📥 Discovered new block: {}. Gossiping to neighbors.", node.id, hash);
                            // Recursive Propagation: Gossip to neighbors (Flood protection handled by seen_blocks)
                            simulator_network.gossip_send(&node.id, Message::Block(b), slot);
                        }
                    }
                    _ => {}
                }
            }

            // 2. Authorship: Attempt to propose a new block candidate
            if let Some(block) = node.propose_block(slot, randomness) {
                log::info!("[{}] ⚡ Authored block at height {} (hash: {})", 
                    node.id, block.header.number, block.hash());
                
                // Initial Propagation
                simulator_network.gossip_send(&node.id, Message::Block(block), slot);
            }
        }
    }

    log::info!("Simulation complete.");
    for node in nodes {
        log::info!("Node {} canonical head: {} (Blocks discovered: {})", 
            node.id, node.best_head_hash, node.blocks.len());
    }
}
