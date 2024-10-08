use crate::appstate::AppState;
use crate::repository::tree::{TreeRepository, TreeRepositoryTrait};

#[derive(Clone)]
pub struct TreeService {
    tree_repo: TreeRepository,
}

impl TreeService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            tree_repo: TreeRepository::new(app_state),
        }
    }

    pub async fn get_at_height(
        &self,
        block_height: u64,
    ) -> anyhow::Result<Option<(Vec<u8>, u64)>> {
        let commiment_tree =
            self.tree_repo.get_at_height(block_height as i32).await?;
        Ok(commiment_tree.map(|tree| (tree.tree, tree.block_height as u64)))
    }
}
