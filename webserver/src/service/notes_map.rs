use crate::appstate::AppState;
use crate::repository::notes_map::{
    NotesMapRepository, NotesMapRepositoryTrait,
};

#[derive(Clone)]
pub struct NotesMapService {
    notes_map_repo: NotesMapRepository,
}

impl NotesMapService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            notes_map_repo: NotesMapRepository::new(app_state),
        }
    }

    pub async fn get_notes_map(
        &self,
        from_block_height: u64,
    ) -> Vec<(u64, bool, u64, u64)> {
        self.notes_map_repo
            .get_notes_map(from_block_height as i32)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|notes_map_entry| {
                (
                    notes_map_entry.note_index as u64,
                    notes_map_entry.is_fee_unshielding,
                    notes_map_entry.note_position as u64,
                    notes_map_entry.block_height as u64,
                )
            })
            .collect()
    }
}
