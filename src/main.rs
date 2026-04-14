pub mod primitives;
pub mod core;
pub mod network;

use crate::core::node::Node;
use crate::network::network::{NetworkSimulator, Message};

fn main() {
    env_logger::init();
    log::info!("--- Substrate Consensus Lab: Simulation ---");

    // Simulation Parameters
    let num_nodes = 3;
    let total_slots = 20;
    let network_latency = 1; 
    let consensus_threshold = u64::MAX / 3;

    let mut simulator_network = NetworkSimulator::new(network_latency);
    let mut nodes: Vec<Node> = (0..num_nodes)
        .map(|i| {
            let id = format!("node_{}", i);
            simulator_network.register_node(id.clone());
            Node::new(id, consensus_threshold)
        })
        .collect();

    let mut randomness = [0u8; 32];

    // Discrete-Event Simulation Loop
    for slot in 1..=total_slots {
        log::info!("---------------- Slot {} ----------------", slot);
        
        randomness[0] = (slot % 256) as u8;
        let mut blocks_proposed = Vec::new();

        for node in nodes.iter_mut() {
            // 1. Ingress: Poll network for arrived messages
            let messages = simulator_network.poll_ingress(&node.id, slot);
            for msg in messages {
                match msg {
                    Message::Block(b) => {
                        log::debug!("[{}] Importing block: {}", node.id, b.hash());
                        node.import_block(b);
                    }
                    _ => {}
                }
            }

            // 2. Authorship: Attempt to propose a new block candidate
            if let Some(block) = node.propose_block(slot, randomness) {
                log::info!("[{}] ⚡ Proposed block at height {} (hash: {})", 
                    node.id, block.header.number, block.hash());
                blocks_proposed.push((node.id.clone(), block));
            }
        }

        // 3. Propagation: Gossip newly proposed blocks across the network
        for (sender_id, block) in blocks_proposed {
            simulator_network.gossip_broadcast(&sender_id, Message::Block(block), slot);
        }
    }

    log::info!("Simulation complete.");
    for node in nodes {
        log::info!("Node {} canonical head: {} (Blocks discovered: {})", 
            node.id, node.best_head_hash, node.blocks.len());
    }
}
