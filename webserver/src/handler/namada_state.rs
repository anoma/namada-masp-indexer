use axum::extract::State;
use axum::Json;
use axum_macros::debug_handler;
use axum_trace_id::TraceId;

use crate::error::namada_state::NamadaStateError;
use crate::response::namada_state::LatestHeightResponse;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_latest_height(
    _trace_id: TraceId<String>,
    State(state): State<CommonState>,
) -> Result<Json<LatestHeightResponse>, NamadaStateError> {
    let maybe_height = state.namada_state_service.get_latest_height().await;

    Ok(Json(LatestHeightResponse {
        block_height: maybe_height.map(|h| h.0).unwrap_or_default(),
    }))
}
