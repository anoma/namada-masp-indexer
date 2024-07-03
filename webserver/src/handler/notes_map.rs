use axum::extract::{Query, State};
use axum::Json;
use axum_macros::debug_handler;
use axum_trace_id::TraceId;
use shared::error::InspectWrap;

use crate::dto::notes_map::NotesMapQueryParams;
use crate::error::notes_map::NotesMapError;
use crate::response::notes_map::NotesMapResponse;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_notes_map(
    _trace_id: TraceId<String>,
    State(state): State<CommonState>,
    Query(query_params): Query<NotesMapQueryParams>,
) -> Result<Json<NotesMapResponse>, NotesMapError> {
    let from_block_height = query_params.height;

    let notes_map = state
        .notes_map_service
        .get_notes_map(from_block_height)
        .await
        .inspect_wrap("get_notes_map", |err| {
            NotesMapError::Database(err.to_string())
        })?;

    Ok(Json(NotesMapResponse::new(notes_map)))
}
