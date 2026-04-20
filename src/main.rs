pub mod primitives;
pub mod core;
pub mod network;

use crate::core::node::Node;
use crate::core::metrics::SimMetrics;
use crate::network::network::{NetworkSimulator, Message};

/// Configures a single simulation run.
struct SimConfig {
    label: &'static str,
    total_slots: u64,
    hop_latency: u64,
    consensus_threshold: u64,
    total_validators: usize,
    /// Slot at which the link between node_1 and node_2 is severed.
    partition_start: u64,
    /// Slot at which the link is restored. 0 = no heal within the run.
    partition_end: u64,
}

fn run_experiment(cfg: SimConfig) -> SimMetrics {
    println!("\n════════════════════════════════════════════════════════");
    println!("  EXPERIMENT: {}", cfg.label);
    println!("  Partition: slots {}–{}", cfg.partition_start, cfg.partition_end);
    println!("════════════════════════════════════════════════════════");

    let mut net = NetworkSimulator::new(cfg.hop_latency);
    let mut metrics = SimMetrics::new(cfg.total_slots, 3);

    // Topology: Line (node_0 -- node_1 -- node_2)
    let node_ids = vec!["node_0".to_string(), "node_1".to_string(), "node_2".to_string()];
    for id in &node_ids {
        net.register_node(id.clone());
    }
    net.add_neighbor("node_0", "node_1");
    net.add_neighbor("node_1", "node_2");

    let mut nodes: Vec<Node> = node_ids.into_iter()
        .map(|id| Node::new(id, cfg.consensus_threshold, cfg.total_validators))
        .collect();

    let mut randomness = [0u8; 32];

    for slot in 1..=cfg.total_slots {
        // Network partition events
        if slot == cfg.partition_start {
            log::warn!("[Slot {}] PARTITION: severing node_1 <-> node_2", slot);
            net.disconnect("node_1", "node_2");
        }
        if cfg.partition_end > 0 && slot == cfg.partition_end {
            log::warn!("[Slot {}] HEAL: restoring node_1 <-> node_2", slot);
            net.connect("node_1", "node_2");
        }

        randomness[0] = (slot % 256) as u8;
        let mut authors_this_slot = 0;

        for node in nodes.iter_mut() {
            // Drain ingress queue
            let messages = net.poll_ingress(&node.id, slot);
            for msg in messages {
                match msg {
                    Message::Block(b) => {
                        let hash = b.hash();
                        if let Some(reorg_depth) = node.import_block(b.clone()) {
                            log::debug!("[{}] Re-org: old head height was {}", node.id, reorg_depth);
                            metrics.record_reorg(reorg_depth);
                            net.gossip_send(&node.id, Message::Block(b), slot as u64);
                        } else if node.seen_blocks.contains(&hash) {
                            // Flood-control: already known, do not re-gossip.
                        }
                    }
                    Message::Precommit(precommit) => {
                        metrics.record_precommit_received();
                        node.handle_precommit(precommit.clone());
                        // Forward precommit to neighbors
                        net.gossip_send(&node.id, Message::Precommit(precommit), slot as u64);
                    }
                    _ => {}
                }
            }


            // Propose
            if let Some(block) = node.propose_block(slot, randomness) {
                log::info!("[{}] Block at height {} (slot {})", node.id, block.header.number, slot);
                metrics.record_authorship();
                metrics.update_max_height(block.header.number);
                authors_this_slot += 1;
                net.gossip_send(&node.id, Message::Block(block), slot as u64);
            }

            // Every slot, active nodes broadcast their GRANDPA vote for their best head.
            let old_finalized = node.finalized_height;
            let precommit = node.create_precommit(slot);
            metrics.record_precommit_broadcast();
            
            // Node implicitly handles its own precommit
            node.handle_precommit(precommit.clone());
            net.gossip_send(&node.id, Message::Precommit(precommit), slot as u64);

            if node.finalized_height > old_finalized {
                metrics.record_finalization_round();
            }
        }


        if authors_this_slot > 1 {
            metrics.record_collision(slot as u64);
        }

        let heads: Vec<_> = nodes.iter().map(|n| n.best_head_hash).collect();
        metrics.observe_convergence(slot as u64, &heads);
    }

    for node in &nodes {
        metrics.record_final_state(node.id.clone(), node.best_height(), node.finalized_height);
    }

    metrics.report();
    metrics
}

fn main() {
    env_logger::init();

    // ── Experiment A: Short partition (5 slots of isolation) ──────────────────
    run_experiment(SimConfig {
        label:               "Short Partition (5 slots isolated: slots 15-20)",
        total_slots:         40,
        hop_latency:         1,
        consensus_threshold: u64::MAX / 3,
        total_validators:    3,
        partition_start:     15,
        partition_end:       20,
    });

    // ── Experiment B: Long partition (15 slots of isolation) ─────────────────
    run_experiment(SimConfig {
        label:               "Long Partition (15 slots isolated: slots 5-20)",
        total_slots:         40,
        hop_latency:         1,
        consensus_threshold: u64::MAX / 3,
        total_validators:    3,
        partition_start:     5,
        partition_end:       20,
    });

    // ── Experiment C: Long partition (15 slots) WITH GRANDPA-lite ────────────────
    run_experiment(SimConfig {
        label:               "Long Partition WITH FINALITY GADGET (15 slots isolated)",
        total_slots:         40,
        hop_latency:         1,
        consensus_threshold: u64::MAX / 3,
        total_validators:    3,
        partition_start:     5,
        partition_end:       20,
    });
}
