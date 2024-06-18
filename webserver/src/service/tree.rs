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

    pub async fn get_latest(&self) -> Option<(Vec<u8>, u64)> {
        let commiment_tree = self.tree_repo.get_latest().await.unwrap();
        if let Some(tree) = commiment_tree {
            Some((tree.tree, tree.block_height as u64))
        } else {
            None
        }
    }

    pub async fn get_at_height(
        &self,
        block_height: u64,
    ) -> Option<(Vec<u8>, u64)> {
        let commiment_tree = self
            .tree_repo
            .get_at_height(block_height as i32)
            .await
            .unwrap();
        if let Some(tree) = commiment_tree {
            Some((tree.tree, tree.block_height as u64))
        } else {
            None
        }
    }
}
