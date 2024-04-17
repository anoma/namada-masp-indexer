use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use namada_sdk::masp_primitives::merkle_tree::IncrementalWitness;
use namada_sdk::masp_primitives::sapling::Node;

#[derive(Clone, Debug)]
pub struct WitnessMap(Arc<Mutex<HashMap<usize, IncrementalWitness<Node>>>>);

impl WitnessMap {
    pub fn new(tree: HashMap<usize, IncrementalWitness<Node>>) -> Self {
        Self(Arc::new(Mutex::new(tree)))
    }

    pub fn update(&self, node: Node) -> Result<(), ()> {
        for (_, witness) in self.0.lock().unwrap().iter_mut() {
            witness.append(node)?
        }
        Ok(())
    }

    pub fn insert(&self, note_pos: usize, witness: IncrementalWitness<Node>) {
        self.0.lock().unwrap().insert(note_pos, witness);
    }
}

impl Default for WitnessMap {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(HashMap::default())))
    }
}
