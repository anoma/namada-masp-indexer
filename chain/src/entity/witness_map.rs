use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use namada_sdk::borsh::BorshSerializeExt;
use namada_sdk::masp_primitives::merkle_tree::IncrementalWitness;
use namada_sdk::masp_primitives::sapling::Node;
use orm::witness::WitnessInsertDb;
use shared::height::BlockHeight;
use shared::transactional::Transactional;

#[derive(Default, Debug)]
struct InnerWitnessMap {
    transactional: Transactional<HashMap<usize, IncrementalWitness<Node>>>,
}

impl InnerWitnessMap {
    const fn new(
        witness_map: HashMap<usize, IncrementalWitness<Node>>,
    ) -> Self {
        Self {
            transactional: Transactional::new(witness_map),
        }
    }

    fn size(&self) -> usize {
        self.transactional.as_ref().len()
    }

    fn rollback(&mut self) {
        self.transactional.rollback();
    }

    fn update(&mut self, node: Node) -> Result<(), usize> {
        for (note_pos, witness) in self.transactional.as_mut().iter_mut() {
            witness.append(node).map_err(|()| *note_pos)?;
        }
        Ok(())
    }

    fn insert(&mut self, note_pos: usize, witness: IncrementalWitness<Node>) {
        self.transactional.as_mut().insert(note_pos, witness);
    }

    #[allow(clippy::wrong_self_convention)]
    fn into_db(
        &mut self,
        block_height: BlockHeight,
    ) -> Option<Vec<WitnessInsertDb>> {
        if !self.transactional.commit() {
            return None;
        }
        Some(
            self.transactional
                .as_ref()
                .iter()
                .map(|(idx, witness)| WitnessInsertDb {
                    witness_bytes: witness.serialize_to_vec(),
                    witness_idx: *idx as i32,
                    block_height: block_height.0 as i32,
                })
                .collect(),
        )
    }
}

#[derive(Default, Clone, Debug)]
pub struct WitnessMap(Arc<Mutex<InnerWitnessMap>>);

impl WitnessMap {
    pub fn new(witness_map: HashMap<usize, IncrementalWitness<Node>>) -> Self {
        Self(Arc::new(Mutex::new(InnerWitnessMap::new(witness_map))))
    }

    pub fn size(&self) -> usize {
        self.0.lock().unwrap().size()
    }

    pub fn rollback(&self) {
        self.0.lock().unwrap().rollback()
    }

    pub fn update(&self, node: Node) -> Result<(), usize> {
        self.0.lock().unwrap().update(node)
    }

    pub fn insert(&self, note_pos: usize, witness: IncrementalWitness<Node>) {
        self.0.lock().unwrap().insert(note_pos, witness)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn into_db(
        &self,
        block_height: BlockHeight,
    ) -> Option<Vec<WitnessInsertDb>> {
        self.0.lock().unwrap().into_db(block_height)
    }
}
