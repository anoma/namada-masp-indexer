use std::collections::BTreeMap;

use orm::notes_index::NotesIndexInsertDb;
use shared::indexed_tx::{IndexedTx, MaspIndexedTx};

#[derive(Default, Clone, Debug)]
pub struct TxNoteMap(BTreeMap<MaspIndexedTx, usize>);

impl TxNoteMap {
    pub fn insert(&mut self, indexed_tx: MaspIndexedTx, note_pos: usize) {
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
                    &MaspIndexedTx {
                        indexed_tx:
                            IndexedTx {
                                block_height,
                                block_index,
                                masp_tx_index,
                            },
                        kind,
                    },
                    &note_pos,
                )| NotesIndexInsertDb {
                    block_index: block_index.0 as i32,
                    note_position: note_pos as i32,
                    block_height: block_height.0 as i32,
                    masp_tx_index: masp_tx_index.0 as i32,
                    is_masp_fee_payment: match kind {
                        shared::indexed_tx::MaspEventKind::FeePayment => true,
                        shared::indexed_tx::MaspEventKind::Transfer => false,
                    },
                },
            )
            .collect()
    }
}
