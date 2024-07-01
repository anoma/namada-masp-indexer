use anyhow::Context;
use diesel::dsl::max;
use diesel::{QueryDsl, RunQueryDsl};
use orm::schema::chain_state;
use shared::error::ContextDbInteractError;
use shared::height::BlockHeight;

use crate::appstate::AppState;

#[derive(Clone)]
pub struct NamadaStateRepository {
    pub(crate) app_state: AppState,
}

pub trait NamadaStateRepositoryTrait {
    fn new(app_state: AppState) -> Self;

    async fn get_latest_height(&self) -> anyhow::Result<Option<BlockHeight>>;
}

impl NamadaStateRepositoryTrait for NamadaStateRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn get_latest_height(&self) -> anyhow::Result<Option<BlockHeight>> {
        let conn = self.app_state.get_db_connection().await.context(
            "Failed to retrieve connection from the pool of database \
             connections",
        )?;

        let block_height = conn
            .interact(move |conn| {
                chain_state::dsl::chain_state
                    .select(max(chain_state::dsl::block_height))
                    .first::<Option<i32>>(conn)
            })
            .await
            .context_db_interact_error()?
            .context("Failed to get latest block height from db")?;

        Ok(block_height.map(BlockHeight::from))
    }
}
