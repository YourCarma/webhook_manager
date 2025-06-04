use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

use crate::storage::error::StorageError;

pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("cache component error: {0}")]
    RedisError(String),
    #[error("failed to (de)serialize object: {0}")]
    SerdeError(String),
    #[error("continues executing: {0}")]
    Unavailable(String),
    #[error("unexpected runtime error: {0}")]
    RuntimeError(String),
}

impl ServerError {
    pub fn status_code(&self) -> (String, StatusCode) {
        match self {
            ServerError::RedisError(msg) => (msg.to_owned(), StatusCode::INTERNAL_SERVER_ERROR),
            ServerError::SerdeError(msg) => (msg.to_owned(), StatusCode::INTERNAL_SERVER_ERROR),
            ServerError::Unavailable(msg) => (msg.to_owned(), StatusCode::INTERNAL_SERVER_ERROR),
            ServerError::RuntimeError(msg) => (msg.to_owned(), StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}

impl ErrorResponse {
    pub fn new(code: u16, err: &str, msg: &str) -> Self {
        ErrorResponse {
            code,
            error: err.to_string(),
            message: msg.to_string(),
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (message, status) = self.status_code();
        let mut resp = Json(ErrorResponse { message }).into_response();

        *resp.status_mut() = status;
        resp
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(err: serde_json::Error) -> Self {
        tracing::error!("serde error: {err:#?}");
        ServerError::SerdeError(err.to_string())
    }
}

impl From<reqwest::Error> for ServerError {
    fn from(err: reqwest::Error) -> Self {
        tracing::error!("request error: {err:#?}");
        ServerError::RuntimeError(err.to_string())
    }
}

impl From<StorageError> for ServerError {
    fn from(err: StorageError) -> Self {
        ServerError::RedisError(err.to_string())
    }
}

#[derive(Debug, Deserialize, Serialize, Getters, CopyGetters, ToSchema)]
pub struct Successful {
    #[getset(get_copy = "pub")]
    code: u16,
    #[getset(get = "pub")]
    message: String,
}

impl Default for Successful {
    fn default() -> Self {
        Successful::new(200, "ok")
    }
}

impl Successful {
    pub fn new(code: u16, msg: &str) -> Self {
        let message = msg.to_string();
        Successful { code, message }
    }
}
