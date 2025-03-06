use axum::Json;
use axum::extract::{Query, State};
use axum_macros::debug_handler;
use axum_trace_id::TraceId;
use shared::error::InspectWrap;

use crate::dto::notes_index::NotesIndexQueryParams;
use crate::error::notes_index::NotesIndexError;
use crate::response::notes_index::NotesIndexResponse;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_notes_index(
    _trace_id: TraceId<String>,
    State(state): State<CommonState>,
    Query(query_params): Query<NotesIndexQueryParams>,
) -> Result<Json<NotesIndexResponse>, NotesIndexError> {
    let from_block_height = query_params.height;

    let notes_index = state
        .notes_index_service
        .get_notes_index(from_block_height)
        .await
        .inspect_wrap("get_notes_index", |err| {
            NotesIndexError::Database(err.to_string())
        })?;

    Ok(Json(NotesIndexResponse::new(notes_index)))
}
