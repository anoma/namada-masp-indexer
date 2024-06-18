use axum::extract::{Query, State};
use axum::Json;
use axum_macros::debug_handler;
use axum_trace_id::TraceId;
use shared::height::BlockHeight;

use crate::dto::witness::WitnessMapQueryParams;
use crate::error::witness_map::WitnessMapError;
use crate::response::witness_map::WitnessMapResponse;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_witness_map(
    _trace_id: TraceId<String>,
    State(state): State<CommonState>,
    Query(query_params): Query<WitnessMapQueryParams>,
) -> Result<Json<WitnessMapResponse>, WitnessMapError> {
    let block_height = BlockHeight(query_params.height);
    let from_index = query_params.from;
    let to_index = from_index + query_params.size;

    let witnesses = state
        .witness_map_service
        .get_witnesses(block_height, from_index, to_index)
        .await;

    if let Some(witnesses) = witnesses {
        Ok(Json(WitnessMapResponse::new(block_height, witnesses)))
    } else {
        Err(WitnessMapError::NotFound)
    }
}
