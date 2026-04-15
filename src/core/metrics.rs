use std::collections::HashMap;

pub struct SimMetrics {
    pub total_slots: u64,
    pub validator_count: usize,
    pub total_blocks_authored: u64,
    pub max_height_achieved: u64,
    pub slot_collisions: u64,
    pub node_final_heights: HashMap<String, u64>,
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
        }
    }

    pub fn record_authorship(&mut self) {
        self.total_blocks_authored += 1;
    }

    pub fn record_collision(&mut self) {
        self.slot_collisions += 1;
    }

    pub fn update_max_height(&mut self, height: u64) {
        if height > self.max_height_achieved {
            self.max_height_achieved = height;
        }
    }

    pub fn record_final_state(&mut self, node_id: String, height: u64) {
        self.node_final_heights.insert(node_id, height);
    }

    pub fn report(&self) {
        let inefficiency = (self.total_blocks_authored as f32 / self.max_height_achieved as f32) - 1.0;
        let fork_density = self.slot_collisions as f32 / self.total_slots as f32;

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
        println!("\nPROTOCOL IMPLICATIONS:");
        println!("- Chain Inefficiency:      {:.2}% (wasted work)", inefficiency * 100.0);
        println!("- Fork Density:            {:.2} forks/slot", fork_density);
        println!("- State Divergence:        {} nodes at max height", 
            self.node_final_heights.values().filter(|&&h| h == self.max_height_achieved).count());
        println!("========================================================\n");
    }
}
