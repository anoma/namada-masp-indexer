use anyhow::Context;
use diesel::{
    ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use orm::schema::commitment_tree;
use orm::tree::TreeDb;
use shared::error::ContextDbInteractError;

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
    ) -> anyhow::Result<Option<TreeDb>>;
}

impl TreeRepositoryTrait for TreeRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn get_at_height(
        &self,
        block_height: i32,
    ) -> anyhow::Result<Option<TreeDb>> {
        let conn = self.app_state.get_db_connection().await.context(
            "Failed to retrieve connection from the pool of database \
             connections",
        )?;

        conn.interact(move |conn| {
            commitment_tree::table
                .order(
                    abs(commitment_tree::dsl::block_height - block_height)
                        .asc(),
                )
                .filter(commitment_tree::dsl::block_height.le(block_height))
                .select(TreeDb::as_select())
                .first(conn)
                .optional()
                .with_context(|| {
                    format!(
                        "Failed to look-up commitment tree in the database \
                         closest to the provided height {block_height}"
                    )
                })
        })
        .await
        .context_db_interact_error()?
    }
}
