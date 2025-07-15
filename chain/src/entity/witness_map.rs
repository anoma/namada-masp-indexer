use std::collections::HashMap;

use namada_sdk::borsh::BorshSerializeExt;
use namada_sdk::masp_primitives::merkle_tree::IncrementalWitness;
use namada_sdk::masp_primitives::sapling::Node;
use orm::witness::WitnessInsertDb;
use rayon::prelude::*;
use shared::height::BlockHeight;
use shared::transactional::Transactional;

#[derive(Default, Debug)]
pub struct WitnessMap {
    transactional: Transactional<HashMap<usize, IncrementalWitness<Node>>>,
}

impl WitnessMap {
    pub const fn new(
        witness_map: HashMap<usize, IncrementalWitness<Node>>,
    ) -> Self {
        Self {
            transactional: Transactional::new(witness_map),
        }
    }

    pub fn roots(&self, number_of_roots: usize) -> Vec<(usize, Node)> {
        self.transactional
            .as_ref()
            .iter()
            .take(number_of_roots)
            .map(|(note_index, witness)| (*note_index, witness.root()))
            .collect()
    }

    pub fn size(&self) -> usize {
        self.transactional.as_ref().len()
    }

    pub fn rollback(&mut self) {
        self.transactional.rollback();
    }

    pub fn update(&mut self, node: Node) -> Result<(), usize> {
        self.transactional
            .as_mut()
            .iter_mut()
            .par_bridge()
            .try_for_each(|(note_pos, witness)| {
                witness.append(node).map_err(|()| *note_pos)
            })
    }

    pub fn insert(
        &mut self,
        note_pos: usize,
        witness: IncrementalWitness<Node>,
    ) {
        self.transactional.as_mut().insert(note_pos, witness);
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn into_db(
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
