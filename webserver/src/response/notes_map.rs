use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct NotesMapResponse {
    pub notes_map: Vec<Note>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Note {
    pub is_fee_unshielding: bool,
    pub block_height: u64,
    pub block_index: u64,
    pub masp_tx_index: u64,
    pub note_position: u64,
}

impl NotesMapResponse {
    pub fn new(notes_map: Vec<(bool, u64, u64, u64, u64)>) -> Self {
        Self {
            notes_map: notes_map
                .into_iter()
                .map(
                    |(
                        is_fee_unshielding,
                        block_index,
                        block_height,
                        masp_tx_index,
                        note_position,
                    )| {
                        Note {
                            is_fee_unshielding,
                            block_index,
                            block_height,
                            masp_tx_index,
                            note_position,
                        }
                    },
                )
                .collect(),
        }
    }
}
