use std::collections::HashMap;
use crate::primitives::types::Hash;

pub struct SimMetrics {
    pub total_slots: u64,
    pub validator_count: usize,
    pub total_blocks_authored: u64,
    pub max_height_achieved: u64,
    pub slot_collisions: u64,
    pub node_final_heights: HashMap<String, u64>,
    pub node_finalized_heights: HashMap<String, u64>,
    /// Slot at which the most-recently-opened, unresolved fork began.
    open_fork_slot: Option<u64>,
    /// Recorded convergence latencies (in slots) for each resolved fork.
    convergence_latencies: Vec<u64>,
    /// Re-org depths recorded on import-triggered chain switches.
    reorg_depths: Vec<u64>,
}

impl SimMetrics {
    pub fn new(total_slots: u64, validator_count: usize) -> Self {
        Self {
            total_slots,
            validator_count,
            total_blocks_authored: 0,
            max_height_achieved: 0,
            slot_collisions: 0,
            node_final_heights: HashMap::new(),
            node_finalized_heights: HashMap::new(),
            open_fork_slot: None,
            convergence_latencies: Vec::new(),
            reorg_depths: Vec::new(),
        }
    }

    pub fn record_authorship(&mut self) {
        self.total_blocks_authored += 1;
    }

    /// Records a slot collision and marks the fork as open.
    pub fn record_collision(&mut self, slot: u64) {
        self.slot_collisions += 1;
        // Only open a new fork window if one isn't already open.
        if self.open_fork_slot.is_none() {
            self.open_fork_slot = Some(slot);
        }
    }

    pub fn update_max_height(&mut self, height: u64) {
        if height > self.max_height_achieved {
            self.max_height_achieved = height;
        }
    }

    /// Records the depth (old head height) of a re-org triggered by block import.
    pub fn record_reorg(&mut self, old_head_height: u64) {
        self.reorg_depths.push(old_head_height);
    }

    /// Called every slot with the current canonical heads of all live nodes.
    /// If all heads agree and a fork was open, the convergence latency is sampled.
    pub fn observe_convergence(&mut self, current_slot: u64, heads: &[Hash]) {
        if let Some(fork_start) = self.open_fork_slot {
            let all_agree = heads.windows(2).all(|w| w[0] == w[1]);
            if all_agree && !heads.is_empty() {
                let latency = current_slot.saturating_sub(fork_start);
                self.convergence_latencies.push(latency);
                self.open_fork_slot = None;
            }
        }
    }

    pub fn record_final_state(&mut self, node_id: String, height: u64, finalized_height: u64) {
        self.node_final_heights.insert(node_id.clone(), height);
        self.node_finalized_heights.insert(node_id, finalized_height);
    }

    pub fn report(&self) {
        let inefficiency = (self.total_blocks_authored as f32 / self.max_height_achieved as f32) - 1.0;
        let fork_density = self.slot_collisions as f32 / self.total_slots as f32;
        let avg_convergence = if self.convergence_latencies.is_empty() {
            f32::NAN
        } else {
            self.convergence_latencies.iter().sum::<u64>() as f32
                / self.convergence_latencies.len() as f32
        };
        let max_reorg_depth = self.reorg_depths.iter().copied().max().unwrap_or(0);

        println!("\n========================================================");
        println!("  SUBSTRATE CONSENSUS LAB: RESEARCH REPORT ");
        println!("========================================================");
        println!("MODEL DEFINITION:");
        println!("- Slots Simulated:   {}", self.total_slots);
        println!("- Validator Nodes:   {}", self.validator_count);
        println!("- Model Type:        Probabilistic BABE-lite");
        println!("- Fork Choice:       Recursive Longest-Chain");
        println!("\nQUANTIFIED OBSERVATIONS:");
        println!("- Total Blocks Authored:   {}", self.total_blocks_authored);
        println!("- Max Chain Height:        {}", self.max_height_achieved);
        println!("- Slot Collisions (Forks): {}", self.slot_collisions);
        println!("- Forks Resolved:          {}", self.convergence_latencies.len());
        println!("- Re-org Events:           {}", self.reorg_depths.len());
        println!("\nPROTOCOL IMPLICATIONS:");
        println!("- Chain Inefficiency:      {:.2}% (wasted work)", inefficiency * 100.0);
        println!("- Fork Density:            {:.2} forks/slot", fork_density);
        if avg_convergence.is_nan() {
            println!("- Avg Convergence Latency: N/A (no forks resolved)");
        } else {
            println!("- Avg Convergence Latency: {:.2} slots/fork", avg_convergence);
        }
        println!("- Max Re-org Depth:        {} blocks", max_reorg_depth);
        println!("- State Divergence:        {} nodes at max height",
            self.node_final_heights.values().filter(|&&h| h == self.max_height_achieved).count());
        
        let max_finalized = self.node_finalized_heights.values().max().copied().unwrap_or(0);
        println!("- Max Finalized Height:    {} blocks", max_finalized);
        for id in ["node_0", "node_1", "node_2"] {
            if let Some(fh) = self.node_finalized_heights.get(id) {
                println!("  > {} finalized height: {}", id, fh);
            }
        }
        println!("========================================================\n");
    }
}
