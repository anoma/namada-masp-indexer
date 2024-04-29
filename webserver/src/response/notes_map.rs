use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct NotesMapResponse {
    pub notes_map: Vec<Note>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Note {
    pub index: u64,
    pub is_fee_unshielding: bool,
    pub note_position: u64,
    pub block_height: u64,
}

impl NotesMapResponse {
    pub fn new(notes_map: Vec<(u64, bool, u64, u64)>) -> Self {
        Self {
            notes_map: notes_map
                .into_iter()
                .map(
                    |(
                        index,
                        is_fee_unshielding,
                        note_position,
                        block_height,
                    )| {
                        Note {
                            index,
                            is_fee_unshielding,
                            note_position,
                            block_height,
                        }
                    },
                )
                .collect(),
        }
    }
}
