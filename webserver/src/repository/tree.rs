use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use orm::schema::commitment_tree;
use orm::tree::TreeDb;

use crate::appstate::AppState;
use crate::utils::sql::abs;

#[derive(Clone)]
pub struct TreeRepository {
    pub(crate) app_state: AppState,
}

pub trait TreeRepositoryTrait {
    fn new(app_state: AppState) -> Self;
    async fn get_at_height(
        &self,
        block_height: i32,
    ) -> Result<Option<TreeDb>, String>;
}

impl TreeRepositoryTrait for TreeRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn get_at_height(
        &self,
        block_height: i32,
    ) -> Result<Option<TreeDb>, String> {
        let conn = self.app_state.get_db_connection().await.unwrap();

        conn.interact(move |conn| {
            commitment_tree::table
                .order(
                    abs(commitment_tree::dsl::block_height - block_height)
                        .asc(),
                )
                .limit(1)
                .select(TreeDb::as_select())
                .first(conn)
                .ok()
        })
        .await
        .map_err(|e| e.to_string())
    }
}
