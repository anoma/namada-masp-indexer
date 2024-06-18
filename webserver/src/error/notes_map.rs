use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::response::api::ApiErrorResponse;

#[derive(Error, Debug)]
pub enum NotesMapError {
    #[error("NotesMap not found")]
    NotFound,
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl IntoResponse for NotesMapError {
    fn into_response(self) -> Response {
        let status_code = match self {
            NotesMapError::NotFound => StatusCode::NOT_FOUND,
            NotesMapError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
