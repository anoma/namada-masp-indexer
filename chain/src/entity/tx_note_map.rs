use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use orm::notes_map::NotesMapInsertDb;
use shared::height::BlockHeight;
use shared::indexed_tx::IndexedTx;

#[derive(Clone, Debug)]
pub struct TxNoteMap(Arc<Mutex<BTreeMap<IndexedTx, usize>>>);

impl TxNoteMap {
    pub fn new(tree: BTreeMap<IndexedTx, usize>) -> Self {
        Self(Arc::new(Mutex::new(tree)))
    }

    pub fn len(&self) -> usize {
        self.0.lock().unwrap().len()
    }

    pub fn insert(&self, indexed_tx: IndexedTx, note_pos: usize) {
        self.0.lock().unwrap().insert(indexed_tx, note_pos);
    }

    pub fn into_db(&self, block_height: BlockHeight) -> Vec<NotesMapInsertDb> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .map(|(indexed_tx, idx)| NotesMapInsertDb {
                note_index: indexed_tx.index.0 as i32,
                is_fee_unshielding: indexed_tx.is_fee_unshielding,
                note_position: *idx as i32,
                block_height: block_height.0 as i32,
            })
            .collect()
    }
}

impl Default for TxNoteMap {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(BTreeMap::default())))
    }
}
