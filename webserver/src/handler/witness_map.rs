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
    let witnesses = state
        .witness_map_service
        .get_witnesses(BlockHeight(query_params.height))
        .await;

    if let Some((witnesses, block_height)) = witnesses {
        Ok(Json(WitnessMapResponse::new(
            BlockHeight(block_height),
            witnesses,
        )))
    } else {
        Err(WitnessMapError::NotFound)
    }
}
