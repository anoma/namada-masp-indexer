use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use shared::indexed_tx::IndexedTx;

#[derive(Clone, Debug)]
pub struct TxNoteMap(Arc<Mutex<BTreeMap<IndexedTx, usize>>>);

impl TxNoteMap {
    pub fn new(tree: BTreeMap<IndexedTx, usize>) -> Self {
        Self(Arc::new(Mutex::new(tree)))
    }

    pub fn insert(&self, indexed_tx: IndexedTx, note_pos: usize) {
        self.0.lock().unwrap().insert(indexed_tx, note_pos);
    }
}

impl Default for TxNoteMap {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(BTreeMap::default())))
    }
}
