use crate::primitives::types::{Block, Header, Slot};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub enum Message {
    Block(Block),
    Header(Header),
}


pub struct NetworkSimulator {
    /// Mapping of NodeID to their incoming message queue. 
    pub nodes: HashMap<String, VecDeque<(Slot, Message)>>,
    /// Mapping of NodeID to their connected neighbors (P2P Topology).
    pub neighbors: HashMap<String, Vec<String>>,
    /// Propagation delay per hop in slots.
    pub hop_latency: Slot,
}

impl NetworkSimulator {
    pub fn new(hop_latency: Slot) -> Self {
        Self {
            nodes: HashMap::new(),
            neighbors: HashMap::new(),
            hop_latency,
        }
    }

    pub fn register_node(&mut self, node_id: String) {
        self.nodes.insert(node_id.clone(), VecDeque::new());
        self.neighbors.insert(node_id, Vec::new());
    }

    /// Establishes bidirectional adjacency.
    pub fn add_neighbor(&mut self, a: &str, b: &str) {
        if let Some(neighbors) = self.neighbors.get_mut(a) {
            neighbors.push(b.to_string());
        }
        if let Some(neighbors) = self.neighbors.get_mut(b) {
            neighbors.push(a.to_string());
        }
    }

    /// Gossips a message to all direct neighbors of the sender.
    pub fn gossip_send(&mut self, sender_id: &str, message: Message, current_slot: Slot) {
        let arrival_slot = current_slot + self.hop_latency;
        if let Some(peers) = self.neighbors.get(sender_id) {
            for peer in peers {
                if let Some(queue) = self.nodes.get_mut(peer) {
                    queue.push_back((arrival_slot, message.clone()));
                }
            }
        }
    }

    /// Drains arrival-ready messages from the ingress queue.
    pub fn poll_ingress(&mut self, node_id: &str, current_slot: Slot) -> Vec<Message> {
        let mut messages = Vec::new();
        if let Some(queue) = self.nodes.get_mut(node_id) {
            while let Some((arrival_slot, _)) = queue.front() {
                if *arrival_slot <= current_slot {
                    let (_, msg) = queue.pop_front().expect("Queue front must exist");
                    messages.push(msg);
                } else {
                    break;
                }
            }
        }
        messages
    }

    /// Sever a bidirectional connection between two nodes.
    pub fn disconnect(&mut self, a: &str, b: &str) {
        if let Some(peers) = self.neighbors.get_mut(a) {
            peers.retain(|p| p != b);
        }
        if let Some(peers) = self.neighbors.get_mut(b) {
            peers.retain(|p| p != a);
        }
    }

    /// Re-establish a bidirectional connection.
    pub fn connect(&mut self, a: &str, b: &str) {
        self.add_neighbor(a, b);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::types::{Block, Header, Hash};

    #[test]
    fn test_topology_neighbors() {
        let mut sim = NetworkSimulator::new(1);
        sim.register_node("A".to_string());
        sim.register_node("B".to_string());
        sim.register_node("C".to_string());

        sim.add_neighbor("A", "B");
        sim.add_neighbor("B", "C");

        assert_eq!(sim.neighbors.get("A").unwrap(), &vec!["B".to_string()]);
        assert_eq!(sim.neighbors.get("B").unwrap(), &vec!["A".to_string(), "C".to_string()]);
        assert_eq!(sim.neighbors.get("C").unwrap(), &vec!["B".to_string()]);
    }

    #[test]
    fn test_gossip_propagation() {
        let mut sim = NetworkSimulator::new(1);
        sim.register_node("A".to_string());
        sim.register_node("B".to_string());
        sim.add_neighbor("A", "B");

        let header = Header {
            parent_hash: Hash::zero(),
            number: 1,
            state_root: Hash::zero(),
            extrinsics_root: Hash::zero(),
            slot: 1,
            author: "A".to_string(),
        };
        let block = Block { header, extrinsics: vec![] };
        
        // Gossip from A to B
        sim.gossip_send("A", Message::Block(block), 1);

        // B should have nothing at Slot 1 (due to latency)
        assert!(sim.poll_ingress("B", 1).is_empty());
        
        // B should receive the block at Slot 2
        let messages = sim.poll_ingress("B", 2);
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_dynamic_topology() {
        let mut sim = NetworkSimulator::new(1);
        sim.register_node("A".to_string());
        sim.register_node("B".to_string());
        sim.add_neighbor("A", "B");
        assert!(sim.neighbors.get("A").unwrap().contains(&"B".to_string()));

        sim.disconnect("A", "B");
        assert!(sim.neighbors.get("A").unwrap().is_empty());
        assert!(sim.neighbors.get("B").unwrap().is_empty());

        sim.connect("A", "B");
        assert!(sim.neighbors.get("A").unwrap().contains(&"B".to_string()));
    }
}
