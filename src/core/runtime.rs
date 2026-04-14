use crate::primitives::types::{Extrinsic, Hash};
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use parity_scale_codec::Encode;

#[derive(Debug, Clone, Serialize, Deserialize, Default, Encode)]
pub struct State(pub BTreeMap<Vec<u8>, Vec<u8>>);

impl State {
    /// Commits world state to a unique Blake3 hash.
    /// 
    /// INVARIANT: BTreeMap's sorted nature ensures SCALE encoding 
    /// results in a deterministic root across all nodes.
    pub fn root(&self) -> Hash {
        let bytes = self.encode();
        Hash::from_bytes(blake3::hash(&bytes).into())
    }
}

/// State Transition Function (STF).
pub struct Runtime {
    pub state: State,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            state: State::default(),
        }
    }

    /// Mutates the world state based on extrinsic payload.
    ///
    /// SAFETY: Atomic operation within the discrete simulation slot.
    pub fn execute_transaction(&mut self, extrinsic: Extrinsic) {
        match extrinsic {
            Extrinsic::SetState { key, value } => {
                self.state.0.insert(key, value);
            }
            Extrinsic::Transfer { from, to, amount } => {
                let from_key = format!("balance:{}", from).into_bytes();
                let to_key = format!("balance:{}", to).into_bytes();

                let from_balance = self.get_read_balance(&from_key);
                if from_balance >= amount {
                    self.set_write_balance(&from_key, from_balance - amount);
                    let to_balance = self.get_read_balance(&to_key);
                    self.set_write_balance(&to_key, to_balance + amount);
                }
            }
        }
    }

    fn get_read_balance(&self, key: &[u8]) -> u64 {
        self.state.0.get(key)
            .and_then(|v| bincode_deserialize(v))
            .unwrap_or(0)
    }

    fn set_write_balance(&mut self, key: &[u8], balance: u64) {
        self.state.0.insert(key.to_vec(), bincode_serialize(balance));
    }
}

fn bincode_serialize(val: u64) -> Vec<u8> {
    val.to_le_bytes().to_vec()
}

fn bincode_deserialize(bytes: &[u8]) -> Option<u64> {
    if bytes.len() == 8 {
        let mut b = [0u8; 8];
        b.copy_from_slice(bytes);
        Some(u64::from_le_bytes(b))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::types::Extrinsic;

    #[test]
    fn test_state_determinism() {
        let mut r1 = Runtime::new();
        let mut r2 = Runtime::new();

        let ext = Extrinsic::SetState {
            key: b"foo".to_vec(),
            value: b"bar".to_vec(),
        };

        r1.execute_transaction(ext.clone());
        r2.execute_transaction(ext);

        assert_eq!(r1.state.root(), r2.state.root());
    }

    #[test]
    fn test_transfer_logic() {
        let mut runtime = Runtime::new();
        let alice = "alice".to_string();
        let bob = "bob".to_string();

        let alice_key = format!("balance:{}", alice).into_bytes();
        runtime.state.0.insert(alice_key, 100u64.to_le_bytes().to_vec());

        runtime.execute_transaction(Extrinsic::Transfer {
            from: alice.clone(),
            to: bob.clone(),
            amount: 40,
        });

        assert_eq!(runtime.get_read_balance(&format!("balance:{}", alice).into_bytes()), 60);
        assert_eq!(runtime.get_read_balance(&format!("balance:{}", bob).into_bytes()), 40);
    }
}
