use crate::appstate::AppState;
use crate::repository::notes_index::{
    NotesIndexRepository, NotesIndexRepositoryTrait,
};

#[derive(Clone)]
pub struct NotesIndexService {
    notes_index_repo: NotesIndexRepository,
}

impl NotesIndexService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            notes_index_repo: NotesIndexRepository::new(app_state),
        }
    }

    pub async fn get_notes_index(
        &self,
        from_block_height: u64,
    ) -> anyhow::Result<Vec<(u64, u64, u64, u64)>> {
        Ok(self
            .notes_index_repo
            .get_notes_index(from_block_height as i32)
            .await?
            .into_iter()
            .map(|notes_index_entry| {
                (
                    notes_index_entry.block_height as u64,
                    notes_index_entry.block_index as u64,
                    notes_index_entry.masp_tx_index as u64,
                    notes_index_entry.note_position as u64,
                )
            })
            .collect())
    }
}
