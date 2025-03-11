use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct NotesIndexResponse {
    pub notes_index: Vec<Note>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Note {
    pub block_height: u64,
    pub block_index: u64,
    pub masp_tx_index: u64,
    pub note_position: u64,
    pub is_masp_fee_payment: bool,
}

impl NotesIndexResponse {
    pub fn new(notes_index: Vec<(u64, u64, u64, u64, bool)>) -> Self {
        Self {
            notes_index: notes_index
                .into_iter()
                .map(
                    |(
                        block_height,
                        block_index,
                        masp_tx_index,
                        note_position,
                        is_masp_fee_payment,
                    )| {
                        Note {
                            block_height,
                            block_index,
                            masp_tx_index,
                            note_position,
                            is_masp_fee_payment,
                        }
                    },
                )
                .collect(),
        }
    }
}
