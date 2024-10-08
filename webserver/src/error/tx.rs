use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::response::api::ApiErrorResponse;

#[derive(Error, Debug)]
pub enum TxError {
    #[error("Database error: {0}")]
    Database(String),
}

impl IntoResponse for TxError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            TxError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
