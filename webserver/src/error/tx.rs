use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::response::api::ApiErrorResponse;

#[derive(Error, Debug)]
pub enum TxError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Error parsing API query: {0}")]
    RawQuery(String),
    #[error("Failed to validate API query: {0}")]
    Validation(String),
}

impl IntoResponse for TxError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            TxError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TxError::RawQuery(_) => StatusCode::BAD_REQUEST,
            TxError::Validation(_) => StatusCode::BAD_REQUEST,
        };
        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
