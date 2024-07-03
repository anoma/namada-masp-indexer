use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::response::api::ApiErrorResponse;

#[derive(Error, Debug)]
pub enum NamadaStateError {
    #[error("Database error: {0}")]
    Database(String),
}

impl IntoResponse for NamadaStateError {
    fn into_response(self) -> Response {
        let status_code = match self {
            NamadaStateError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
