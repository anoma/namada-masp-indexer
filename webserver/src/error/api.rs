use axum::response::{IntoResponse, Response};
use thiserror::Error;

use super::tree::TreeError;
use super::witness_map::WitnessMapError;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    TreeError(#[from] TreeError),
    #[error(transparent)]
    WitnessMapError(#[from] WitnessMapError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::TreeError(error) => error.into_response(),
            ApiError::WitnessMapError(error) => error.into_response(),
        }
    }
}
