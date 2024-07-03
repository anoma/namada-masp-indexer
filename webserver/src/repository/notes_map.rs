use anyhow::Context;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use orm::notes_map::NotesMapDb;
use orm::schema::notes_map;
use shared::error::ContextDbInteractError;

use crate::appstate::AppState;

#[derive(Clone)]
pub struct NotesMapRepository {
    pub(crate) app_state: AppState,
}

pub trait NotesMapRepositoryTrait {
    fn new(app_state: AppState) -> Self;
    async fn get_notes_map(
        &self,
        block_height: i32,
    ) -> anyhow::Result<Vec<NotesMapDb>>;
}

impl NotesMapRepositoryTrait for NotesMapRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn get_notes_map(
        &self,
        block_height: i32,
    ) -> anyhow::Result<Vec<NotesMapDb>> {
        let conn = self.app_state.get_db_connection().await.context(
            "Failed to retrieve connection from the pool of database \
             connections",
        )?;

        conn.interact(move |conn| {
            notes_map::table
                .filter(notes_map::dsl::block_height.le(block_height))
                .select(NotesMapDb::as_select())
                .get_results(conn)
                .with_context(|| {
                    format!(
                        "Failed to retrieve the notes map up to block height \
                         {block_height}"
                    )
                })
        })
        .await
        .context_db_interact_error()?
    }
}
