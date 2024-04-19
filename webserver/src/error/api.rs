use axum::response::{IntoResponse, Response};
use thiserror::Error;

use super::tree::TreeError;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    TreeError(#[from] TreeError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::TreeError(error) => error.into_response(),
        }
    }
}
