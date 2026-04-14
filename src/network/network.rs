use crate::primitives::types::{Block, Header, Slot};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub enum Message {
    Block(Block),
    Header(Header),
}

/// A discrete-event network simulator for protocol research.
///
/// Models propagation delays and message queues across a set of nodes.
pub struct NetworkSimulator {
    /// Mapping of NodeID to their incoming message queue. 
    /// Each entry contains the (ArrivalSlot, Message) tuple.
    pub nodes: HashMap<String, VecDeque<(Slot, Message)>>,
    /// Global propagation delay in slots.
    pub default_latency: Slot,
}

impl NetworkSimulator {
    pub fn new(default_latency: Slot) -> Self {
        Self {
            nodes: HashMap::new(),
            default_latency,
        }
    }

    pub fn register_node(&mut self, node_id: String) {
        self.nodes.insert(node_id, VecDeque::new());
    }

    /// Dispatches a message to all nodes except the sender.
    ///
    /// // TODO: Replace with a Gossip protocol that uses peer-to-peer neighbor mapping.
    pub fn gossip_broadcast(&mut self, sender_id: &str, message: Message, current_slot: Slot) {
        let arrival_slot = current_slot + self.default_latency;
        for (node_id, queue) in self.nodes.iter_mut() {
            if node_id != sender_id {
                queue.push_back((arrival_slot, message.clone()));
            }
        }
    }

    /// Polls the ingress queue for a specific node and returns messages that have met their arrival slot.
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
}
