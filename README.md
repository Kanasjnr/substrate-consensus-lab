# Substrate Consensus Lab

An experimental, discrete-event simulation framework for researching Substrate-based consensus and finality protocols.

## Architecture

The lab is organized into a modular hierarchy mirroring the Polkadot SDK's architectural boundaries:

- **`primitives/`**: Zero-dependency protocol types (Headers, Blocks, Hashing).
- **`core/`**:
  - **`runtime`**: The State Transition Function (STF) managing the world state and deterministic state roots.
  - **`consensus`**: Slot-based leadership (BABE approximation) and Longest-Chain fork selection.
  - **`node`**: The actor implementation responsible for block import, proposal, and chain re-orgs.
- **`network/`**: A discrete-event P2P gossip simulator modeling propagation latency, neighbor topology, and multi-hop flood control.
- **`finality/`**: GRANDPA-lite implementation providing prefix-agreement based finality and safety bounds.

## Technology Stack

- **Serialization**: [Parity SCALE Codec](https://github.com/paritytech/parity-scale-codec) for 1:1 binary protocol fidelity.
- **Cryptography**: [Blake3](https://github.com/BLAKE3-team/BLAKE3) for high-performance, collision-resistant hashing.
- **Determinism**: Threshold-based probabilistic leadership using internal discrete time slots.

## Verification

The code includes a rigorous test suite and a real-time simulation orchestrator.

### Unit Tests

Verify the mathematical and logical integrity of the codec, STF, and P2P gossip engine:

```bash
cargo test
```

### Protocol Simulation

Run the multi-node consensus simulation with structured logging:

```bash
RUST_LOG=info cargo run
```

## Model & Assumptions

1. **Network Topology**: Nodes communicate over a modeled P2P layer with defined discrete hop latency.
2. **Consensus Algorithm**: A threshold-based probabilistic leadership model approximating BABE (Blind Assignment for Blockchain Extension).
3. **Fork Choice**: Recursive longest-chain rule. Ties are resolved by arrival sequence.
4. **Finality Gadget**: A GRANDPA-lite implementation where nodes broadcast votes for their best head. Blocks are finalized when they amass votes from $\ge 2/3$ of the validator set (Prefix Agreement).
5. **Partition Tolerance**: The system is designed to simulate network splits and evaluate convergence latency post-heal. It specifically measures if finality prevents long-range re-orgs during recovery.

## Research & Publications

This simulator has been developed as part of a technical research series on the Substrate consensus model. Each part investigates a different failure mode or safety mechanism:

1. **[The Anatomy of a Fork: Simulating Slot Collisions in Substrate](https://forum.polkadot.network/t/the-anatomy-of-a-fork-simulating-slot-collisions-in-substrate/17514)**  
   _Investigation into how discrete slot leadership creates unavoidable forks even in ideal network conditions._

2. **[Beyond the Broadcast: Simulating P2P Gossip and Visibility Lag](https://forum.polkadot.network/t/beyond-the-broadcast-simulating-p2p-gossip-and-visibility-lag/17517)**  
   _Analysis of how network latency and P2P hop counts exacerbate fork persistence and visibility lag._

3. **[Partition-Induced Re-org Depth: A Comparative Study](https://forum.polkadot.network/t/partition-induced-re-org-depth-a-comparative-study-in-a-babe-like-model/17542)**  
   _Pressure-testing BABE-lite during 15-slot network partitions, observing unbounded re-org growth._

4. **[The Immutable Wall: Bounding Re-org Depth with GRANDPA-lite](https://forum.polkadot.network/t/the-immutable-wall-bounding-re-org-depth-with-grandpa-lite/17572)**  
   _The conclusion of the series: Implementing prefix agreement to create a non-negotiable safety wall that stops deep re-orgs after recovery._

5. **[Under Pressure: Simulating Mempool Flood Control and State Determinism](https://forum.polkadot.network/t/under-pressure-simulating-mempool-flood-control-and-state-determinism/17618)**  
   _Shifting focus to the state machine: Proving that simple flood control prevents P2P routing storms, and deterministic state root verification neutralizes malicious block authors._

## Invariants & Design Philosophy

_This repository is part of a research initiative._

1. **High Fidelity**: If the logic is written here, it should be representationally accurate enough to port into a real Substrate Pallet.
2. **Determinism**: Every run with the same seed must result in identical state roots and chain branches across all simulated nodes.
3. **Safety First**: Finalized blocks are irreversible. Any fork that attempts to revert past a finalized height is rejected, ensuring deterministic safety even in partitioned environments.
4. **Research First**: The focus is on "breaking" the protocol via latency and partitions to study recovery and convergence.
