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
}

impl NotesIndexResponse {
    pub fn new(notes_index: Vec<(u64, u64, u64, u64)>) -> Self {
        Self {
            notes_index: notes_index
                .into_iter()
                .map(
                    |(
                        block_height,
                        block_index,
                        masp_tx_index,
                        note_position,
                    )| {
                        Note {
                            block_height,
                            block_index,
                            masp_tx_index,
                            note_position,
                        }
                    },
                )
                .collect(),
        }
    }
}
