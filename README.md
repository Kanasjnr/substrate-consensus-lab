# Substrate Consensus Lab

An experimental, discrete-event simulation framework for researching Substrate-based consensus and finality protocols.

## Architecture

The lab is organized into a modular hierarchy mirroring the Polkadot SDK's architectural boundaries:

- **`primitives/`**: Zero-dependency protocol types (Headers, Blocks, Hashing).
- **`core/`**:
  - **`runtime`**: The State Transition Function (STF) managing the world state and deterministic state roots.
  - **`consensus`**: Slot-based leadership (BABE approximation) and Longest-Chain fork selection.
  - **`node`**: The actor implementation responsible for block import, proposal, and chain re-orgs.
- **`network/`**: A discrete-event message simulator modeling propagation latency and discrete slots.

## Technology Stack

- **Serialization**: [Parity SCALE Codec](https://github.com/paritytech/parity-scale-codec) for 1:1 binary protocol fidelity.
- **Cryptography**: [Blake3](https://github.com/BLAKE3-team/BLAKE3) for high-performance, collision-resistant hashing.
- **Determinism**: Threshold-based probabilistic leadership using internal discrete time slots.

## Verification

The lab includes a rigorous test suite and a real-time simulation orchestrator.

### Unit Tests

Verify the mathematical and logical integrity of the codec and STF:

```bash
cargo test
```

### Protocol Simulation

Run the multi-node consensus simulation with structured logging:

```bash
RUST_LOG=info cargo run
```

## Invariants & Design Philosophy

_This repository is part of a protocol engineering research initiative._

1. **High Fidelity**: If the logic is written here, it should be representationally accurate enough to port into a real Substrate Pallet.
2. **Determinism**: Every run with the same seed must result in identical state roots and chain branches across all simulated nodes.
3. **Research First**: The focus is on "breaking" the protocol via latency and partitions to study recovery and convergence.

_This repository is part of a protocol engineering research initiative._
