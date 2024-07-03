use axum::extract::{Query, State};
use axum::Json;
use axum_macros::debug_handler;
use axum_trace_id::TraceId;
use shared::error::InspectWrap;

use crate::dto::txs::TxQueryParams;
use crate::error::tx::TxError;
use crate::response::tx::TxResponse;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_tx(
    _trace_id: TraceId<String>,
    State(state): State<CommonState>,
    Query(query_params): Query<TxQueryParams>,
) -> Result<Json<TxResponse>, TxError> {
    let from_block_height = query_params.height;
    let to_block_height = from_block_height + query_params.height_offset;

    let txs = state
        .tx_service
        .get_txs(from_block_height, to_block_height)
        .await
        .inspect_wrap("get_tx", |err| TxError::Database(err.to_string()))?;

    Ok(Json(TxResponse::new(txs)))
}
