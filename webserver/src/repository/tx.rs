use anyhow::Context;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl,
    RunQueryDsl, SelectableHelper,
};
use orm::schema::{chain_state, tx};
use orm::tx::TxDb;
use shared::error::ContextDbInteractError;

use crate::appstate::AppState;

#[derive(Clone)]
pub struct TxRepository {
    pub(crate) app_state: AppState,
}

pub trait TxRepositoryTrait {
    fn new(app_state: AppState) -> Self;
    async fn get_txs(
        &self,
        from_block_height: i32,
        to_block_height: i32,
    ) -> anyhow::Result<Vec<TxDb>>;

    async fn get_txs_by_indices(
        &self,
        indices: Vec<[i32; 2]>,
    ) -> anyhow::Result<Vec<TxDb>>;
}

impl TxRepositoryTrait for TxRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn get_txs(
        &self,
        from_block_height: i32,
        to_block_height: i32,
    ) -> anyhow::Result<Vec<TxDb>> {
        let conn = self.app_state.get_db_connection().await.context(
            "Failed to retrieve connection from the pool of database \
             connections",
        )?;

        conn.interact(move |conn| {
            conn.build_transaction().read_only().run(move |conn| {
                let block_height: i32 = chain_state::table
                    .select(chain_state::dsl::block_height)
                    .get_result(conn)
                    .optional()
                    .with_context(|| {
                        "Failed to get the latest block height from the \
                         database"
                    })?
                    .unwrap_or_default();
                if block_height < to_block_height {
                    anyhow::bail!(
                        "Requested range {from_block_height} -- \
                         {to_block_height} exceeds latest block height \
                         ({block_height})."
                    )
                }
                tx::table
                    .filter(
                        tx::dsl::block_height
                            .ge(from_block_height)
                            .and(tx::dsl::block_height.le(to_block_height)),
                    )
                    .order_by((
                        tx::dsl::block_height.asc(),
                        tx::dsl::block_index.asc(),
                        tx::dsl::masp_tx_index.asc(),
                    ))
                    .select(TxDb::as_select())
                    .get_results(conn)
                    .with_context(|| {
                        format!(
                            "Failed to get transations from the database in \
                             the range {from_block_height}-{to_block_height}"
                        )
                    })
            })
        })
        .await
        .context_db_interact_error()?
    }

    async fn get_txs_by_indices(
        &self,
        indices: Vec<[i32; 2]>,
    ) -> anyhow::Result<Vec<TxDb>> {
        let conn = self.app_state.get_db_connection().await.context(
            "Failed to retrieve connection from the pool of database \
             connections",
        )?;
        let to_block_height =
            indices.iter().map(|ix| ix[0]).max().unwrap_or_default();

        conn.interact(move |conn| {
            conn.build_transaction().read_only().run(move |conn| {
                let block_height: i32 = chain_state::table
                    .select(chain_state::dsl::block_height)
                    .get_result(conn)
                    .optional()
                    .with_context(|| {
                        "Failed to get the latest block height from the \
                         database"
                    })?
                    .unwrap_or_default();
                if block_height < to_block_height {
                    anyhow::bail!(
                        "Requested indices contains {to_block_height}, which \
                         exceeds latest block height ({block_height})."
                    )
                }
                let mut query = tx::table.into_boxed();
                for [height, block_index] in &indices {
                    query = query.or_filter(
                        tx::dsl::block_height
                            .eq(height)
                            .and(tx::dsl::block_index.eq(block_index)),
                    );
                }
                query
                    .order_by((
                        tx::dsl::block_height.asc(),
                        tx::dsl::block_index.asc(),
                        tx::dsl::masp_tx_index.asc(),
                    ))
                    .select(TxDb::as_select())
                    .get_results(conn)
                    .with_context(|| {
                        format!(
                            "Failed to get transations from the database with \
                             indices {:?}",
                            indices
                        )
                    })
            })
        })
        .await
        .context_db_interact_error()?
    }
}
