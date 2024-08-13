use anyhow::Context;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use orm::notes_index::NotesIndexDb;
use orm::schema::notes_index;
use shared::error::ContextDbInteractError;

use crate::appstate::AppState;

#[derive(Clone)]
pub struct NotesIndexRepository {
    pub(crate) app_state: AppState,
}

pub trait NotesIndexRepositoryTrait {
    fn new(app_state: AppState) -> Self;
    async fn get_notes_index(
        &self,
        block_height: i32,
    ) -> anyhow::Result<Vec<NotesIndexDb>>;
}

impl NotesIndexRepositoryTrait for NotesIndexRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn get_notes_index(
        &self,
        block_height: i32,
    ) -> anyhow::Result<Vec<NotesIndexDb>> {
        let conn = self.app_state.get_db_connection().await.context(
            "Failed to retrieve connection from the pool of database \
             connections",
        )?;

        conn.interact(move |conn| {
            notes_index::table
                .filter(notes_index::dsl::block_height.le(block_height))
                .select(NotesIndexDb::as_select())
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
