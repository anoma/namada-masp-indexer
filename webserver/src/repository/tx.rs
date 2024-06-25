use diesel::{
    BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use orm::schema::tx;
use orm::tx::TxDb;

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
    ) -> Result<Vec<TxDb>, String>;
}

impl TxRepositoryTrait for TxRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn get_txs(
        &self,
        from_block_height: i32,
        to_block_height: i32,
    ) -> Result<Vec<TxDb>, String> {
        let conn = self.app_state.get_db_connection().await.unwrap();

        conn.interact(move |conn| {
            tx::table
                .filter(
                    tx::dsl::block_height
                        .ge(from_block_height)
                        .and(tx::dsl::block_height.le(to_block_height)),
                )
                .select(TxDb::as_select())
                .get_results(conn)
                .unwrap_or_default()
        })
        .await
        .map_err(|e| e.to_string())
    }
}
