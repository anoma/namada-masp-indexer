use std::collections::BTreeMap;

use orm::notes_map::NotesMapInsertDb;
use shared::indexed_tx::IndexedTx;

#[derive(Default, Clone, Debug)]
pub struct TxNoteMap(BTreeMap<IndexedTx, (bool, usize)>);

impl TxNoteMap {
    pub fn insert(
        &mut self,
        indexed_tx: IndexedTx,
        is_fee_unshielding: bool,
        note_pos: usize,
    ) {
        self.0.insert(indexed_tx, (is_fee_unshielding, note_pos));
    }

    pub fn into_db(&self) -> Vec<NotesMapInsertDb> {
        self.0
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
