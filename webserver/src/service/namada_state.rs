use shared::height::BlockHeight;

use crate::appstate::AppState;
use crate::repository::namada_state::{
    NamadaStateRepository, NamadaStateRepositoryTrait,
};

#[derive(Clone)]
pub struct NamadaStateService {
    namada_state_repo: NamadaStateRepository,
}

impl NamadaStateService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            namada_state_repo: NamadaStateRepository::new(app_state),
        }
    }

    pub async fn get_latest_height(
        &self,
    ) -> anyhow::Result<Option<BlockHeight>> {
        self.namada_state_repo.get_latest_height().await
    }

    pub async fn get_block_index(
        &self,
    ) -> anyhow::Result<Option<(BlockHeight, xorf::BinaryFuse16)>> {
        self.namada_state_repo
            .get_block_index()
            .await
            .map(|option| {
                option
                    .map(|(height, filter)| (BlockHeight(height as _), filter))
            })
    }
}
