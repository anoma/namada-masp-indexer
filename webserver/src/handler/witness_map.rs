use axum::Json;
use axum::extract::{Query, State};
use axum_macros::debug_handler;
use axum_trace_id::TraceId;
use shared::error::InspectWrap;
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
    let witnesses_and_height = state
        .witness_map_service
        .get_witnesses(BlockHeight(query_params.height))
        .await
        .inspect_wrap("get_witness_map", |err| {
            WitnessMapError::Database(err.to_string())
        })?;

    let (witnesses, block_height) =
        witnesses_and_height.unwrap_or((Vec::new(), query_params.height));

    Ok(Json(WitnessMapResponse::new(
        BlockHeight(block_height),
        witnesses,
    )))
}
