use anyhow::Context;
use diesel::dsl::max;
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use shared::error::ContextDbInteractError;
use shared::height::BlockHeight;
use xorf::BinaryFuse16;

use crate::appstate::AppState;

#[derive(Clone)]
pub struct NamadaStateRepository {
    pub(crate) app_state: AppState,
}

pub trait NamadaStateRepositoryTrait {
    fn new(app_state: AppState) -> Self;

    async fn get_latest_height(&self) -> anyhow::Result<Option<BlockHeight>>;

    async fn get_block_index(&self) -> anyhow::Result<Option<BinaryFuse16>>;
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
                use orm::schema::chain_state;

                chain_state::dsl::chain_state
                    .select(max(chain_state::dsl::block_height))
                    .first::<Option<i32>>(conn)
            })
            .await
            .context_db_interact_error()?
            .context("Failed to get latest block height from db")?;

        Ok(block_height.map(BlockHeight::from))
    }

    async fn get_block_index(&self) -> anyhow::Result<Option<BinaryFuse16>> {
        let conn = self.app_state.get_db_connection().await.context(
            "Failed to retrieve connection from the pool of database \
             connections",
        )?;

        let maybe_serialized_data = conn
            .interact(move |conn| {
                use orm::block_index::BlockIndex;
                use orm::schema::block_index::dsl::block_index;

                anyhow::Ok(
                    block_index
                        .select(BlockIndex::as_select())
                        .first::<BlockIndex>(conn)
                        .optional()
                        .context("Failed to get latest block index from db")?
                        .map(
                            |BlockIndex {
                                 serialized_data, ..
                             }| serialized_data,
                        ),
                )
            })
            .await
            .context_db_interact_error()??;

        tokio::task::block_in_place(|| {
            maybe_serialized_data
                .map(|data| {
                    bincode::deserialize(&data).context(
                        "Failed to deserialize block index data returned from \
                         db",
                    )
                })
                .transpose()
        })
    }
}
