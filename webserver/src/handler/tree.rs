use axum::extract::{Query, State};
use axum::Json;
use axum_macros::debug_handler;
use axum_trace_id::TraceId;
use shared::error::InspectWrap;

use crate::dto::tree::TreeQueryParams;
use crate::error::tree::TreeError;
use crate::response::tree::TreeResponse;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_commitment_tree(
    _trace_id: TraceId<String>,
    State(state): State<CommonState>,
    Query(query_params): Query<TreeQueryParams>,
) -> Result<Json<TreeResponse>, TreeError> {
    let maybe_commitment_tree = state
        .tree_service
        .get_at_height(query_params.height)
        .await
        .inspect_wrap("get_commitment_tree", |err| {
            TreeError::Database(err.to_string())
        })?;

    if let Some((commitment_tree, block_height)) = maybe_commitment_tree {
        Ok(Json(TreeResponse {
            commitment_tree,
            block_height,
        }))
    } else {
        Err(TreeError::NotFound)
    }
}
