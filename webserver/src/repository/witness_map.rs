use diesel::dsl::exists;
use diesel::{
    select, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper,
};
use orm::schema::witness;
use orm::witness::WitnessDb;

use crate::appstate::AppState;

#[derive(Clone)]
pub struct WitnessMapRepository {
    pub(crate) app_state: AppState,
}

pub trait WitnessMapRepositoryTrait {
    fn new(app_state: AppState) -> Self;
    async fn get_witnesses(
        &self,
        block_height: i32,
    ) -> Result<Vec<WitnessDb>, String>;
    async fn block_height_exist(&self, block_height: i32) -> bool;
}

impl WitnessMapRepositoryTrait for WitnessMapRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn get_witnesses(
        &self,
        block_height: i32,
    ) -> Result<Vec<WitnessDb>, String> {
        let conn = self.app_state.get_db_connection().await.unwrap();

        conn.interact(move |conn| {
            witness::table
                .filter(witness::dsl::block_height.eq(block_height))
                .select(WitnessDb::as_select())
                .get_results(conn)
                .unwrap_or_default()
        })
        .await
        .map_err(|e| e.to_string())
    }

    async fn block_height_exist(&self, block_height: i32) -> bool {
        let conn = self.app_state.get_db_connection().await.unwrap();

        conn.interact(move |conn| {
            select(exists(
                witness::table
                    .filter(witness::dsl::block_height.eq(block_height)),
            ))
            .get_result(conn)
            .unwrap_or_default()
        })
        .await
        .unwrap_or_default()
    }
}
