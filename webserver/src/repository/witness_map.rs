use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use orm::schema::witness;
use orm::witness::WitnessDb;

use crate::appstate::AppState;
use crate::utils::sql::abs;

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

        let closest_height: i32 = conn
            .interact(move |conn| {
                witness::table
                    .order(abs(witness::dsl::block_height - block_height).asc())
                    .select(witness::dsl::block_height)
                    .first(conn)
                    .map_err(|e| e.to_string())
            })
            .await
            .map_err(|e| e.to_string())??;

        conn.interact(move |conn| {
            witness::table
                .filter(witness::dsl::block_height.eq(closest_height))
                .select(WitnessDb::as_select())
                .get_results::<WitnessDb>(conn)
                .unwrap_or_default()
        })
        .await
        .map_err(|e| e.to_string())
    }
}
