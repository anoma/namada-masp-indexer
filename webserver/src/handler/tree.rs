use axum::extract::{Query, State};
use axum::Json;
use axum_macros::debug_handler;
use axum_trace_id::TraceId;

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
    let commitment_tree = if let Some(height) = query_params.height {
        state.tree_service.get_at_height(height).await
    } else {
        state.tree_service.get_latest().await
    };

    if let Some((commitment_tree, block_height)) = commitment_tree {
        Ok(Json(TreeResponse {
            commitment_tree,
            block_height,
        }))
    } else {
        Err(TreeError::NotFound)
    }
}
