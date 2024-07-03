use anyhow::Context;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use orm::schema::tx;
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
            tx::table
                .filter(
                    tx::dsl::block_height
                        .ge(from_block_height)
                        .and(tx::dsl::block_height.le(to_block_height)),
                )
                .select(TxDb::as_select())
                .get_results(conn)
                .with_context(|| {
                    format!(
                        "Failed to get transations from the database in the \
                         range {from_block_height}-{to_block_height}"
                    )
                })
        })
        .await
        .context_db_interact_error()?
    }
}
