use std::collections::BTreeMap;

use orm::notes_index::NotesIndexInsertDb;
use shared::indexed_tx::IndexedTx;

#[derive(Default, Clone, Debug)]
pub struct TxNoteMap(BTreeMap<IndexedTx, usize>);

impl TxNoteMap {
    pub fn insert(&mut self, indexed_tx: IndexedTx, note_pos: usize) {
        self.0.insert(indexed_tx, note_pos);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_db(&self) -> Vec<NotesIndexInsertDb> {
        self.0
            .iter()
            .map(
                |(
                    &IndexedTx {
                        block_height,
                        block_index,
                        batch_index,
                        ..
                    },
                    &note_pos,
                )| NotesIndexInsertDb {
                    block_index: block_index.0 as i32,
                    note_position: note_pos as i32,
                    block_height: block_height.0 as i32,
                    masp_tx_index: batch_index as i32,
                },
            )
            .collect()
    }
}
