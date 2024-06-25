use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use namada_sdk::borsh::BorshSerializeExt;
use namada_sdk::masp_primitives::merkle_tree::IncrementalWitness;
use namada_sdk::masp_primitives::sapling::Node;
use orm::witness::WitnessInsertDb;
use shared::height::BlockHeight;

#[derive(Clone, Debug)]
pub struct WitnessMap(Arc<Mutex<HashMap<usize, IncrementalWitness<Node>>>>);

impl WitnessMap {
    pub fn new(tree: HashMap<usize, IncrementalWitness<Node>>) -> Self {
        Self(Arc::new(Mutex::new(tree)))
    }

    pub fn update(&self, node: Node) -> Result<(), usize> {
        for (note_pos, witness) in self.0.lock().unwrap().iter_mut() {
            witness.append(node).map_err(|()| *note_pos)?;
        }
        Ok(())
    }

    pub fn insert(&self, note_pos: usize, witness: IncrementalWitness<Node>) {
        self.0.lock().unwrap().insert(note_pos, witness);
    }

    pub fn into_db(&self, block_height: BlockHeight) -> Vec<WitnessInsertDb> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .map(|(idx, witness)| WitnessInsertDb {
                witness_bytes: witness.serialize_to_vec(),
                witness_idx: *idx as i32,
                block_height: block_height.0 as i32,
            })
            .collect()
    }
}

impl Default for WitnessMap {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(HashMap::default())))
    }
}
