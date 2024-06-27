use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use orm::notes_map::NotesMapInsertDb;
use shared::indexed_tx::IndexedTx;

#[derive(Clone, Debug)]
pub struct TxNoteMap(Arc<Mutex<BTreeMap<IndexedTx, (bool, usize)>>>);

impl TxNoteMap {
    pub fn insert(
        &self,
        indexed_tx: IndexedTx,
        is_fee_unshielding: bool,
        note_pos: usize,
    ) {
        self.0
            .lock()
            .unwrap()
            .insert(indexed_tx, (is_fee_unshielding, note_pos));
    }

    pub fn into_db(&self) -> Vec<NotesMapInsertDb> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .map(
                |(
                    &IndexedTx {
                        block_height,
                        block_index,
                        masp_tx_index,
                    },
                    &(is_fee_unshielding, note_pos),
                )| NotesMapInsertDb {
                    is_fee_unshielding,
                    note_index: block_index.0 as i32,
                    note_position: note_pos as i32,
                    block_height: block_height.0 as i32,
                    masp_tx_index: masp_tx_index.0 as i32,
                },
            )
            .collect()
    }
}

impl Default for TxNoteMap {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(BTreeMap::default())))
    }
}
