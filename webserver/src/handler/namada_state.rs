use axum::extract::State;
use axum::Json;
use axum_macros::debug_handler;
use axum_trace_id::TraceId;
use shared::error::InspectWrap;

use crate::error::namada_state::NamadaStateError;
use crate::response::namada_state::{BlockIndexResponse, LatestHeightResponse};
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_latest_height(
    _trace_id: TraceId<String>,
    State(state): State<CommonState>,
) -> Result<Json<LatestHeightResponse>, NamadaStateError> {
    let maybe_height = state
        .namada_state_service
        .get_latest_height()
        .await
        .inspect_wrap("get_latest_height", |err| {
            NamadaStateError::Database(err.to_string())
        })?;

    Ok(Json(LatestHeightResponse {
        block_height: maybe_height.map(|h| h.0).unwrap_or_default(),
    }))
}

#[debug_handler]
pub async fn get_block_index(
    _trace_id: TraceId<String>,
    State(state): State<CommonState>,
) -> Result<Json<BlockIndexResponse>, NamadaStateError> {
    let maybe_block_index = state
        .namada_state_service
        .get_block_index()
        .await
        .inspect_wrap("get_block_index", |err| {
            NamadaStateError::Database(err.to_string())
        })?;

    if let Some((height, index)) = maybe_block_index {
        Ok(Json(BlockIndexResponse {
            block_height: height.0,
            index,
        }))
    } else {
        Err(NamadaStateError::BlockIndexNotFound)
    }
}
