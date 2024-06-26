use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::response::api::ApiErrorResponse;

#[derive(Error, Debug)]
pub enum NamadaStateError {
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl IntoResponse for NamadaStateError {
    fn into_response(self) -> Response {
        let status_code = match self {
            NamadaStateError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}