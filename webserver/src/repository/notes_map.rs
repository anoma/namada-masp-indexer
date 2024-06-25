use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use orm::notes_map::NotesMapDb;
use orm::schema::notes_map;

use crate::appstate::AppState;

#[derive(Clone)]
pub struct NotesMapRepository {
    pub(crate) app_state: AppState,
}

pub trait NotesMapRepositoryTrait {
    fn new(app_state: AppState) -> Self;
    async fn get_notes_map(
        &self,
        from_block_height: i32,
    ) -> Result<Vec<NotesMapDb>, String>;
}

impl NotesMapRepositoryTrait for NotesMapRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn get_notes_map(
        &self,
        from_block_height: i32,
    ) -> Result<Vec<NotesMapDb>, String> {
        let conn = self.app_state.get_db_connection().await.unwrap();

        conn.interact(move |conn| {
            notes_map::table
                .filter(notes_map::dsl::block_height.le(from_block_height))
                .select(NotesMapDb::as_select())
                .get_results(conn)
                .unwrap_or_default()
        })
        .await
        .map_err(|e| e.to_string())
    }
}
