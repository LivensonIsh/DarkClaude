use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub msg: String,
}

impl ApiError {
    pub fn new(status: u16, code: &'static str, msg: impl Into<String>) -> Self {
        ApiError { status: StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), code, msg: msg.into() }
    }
    pub fn bad(msg: impl Into<String>) -> Self { Self::new(400, "BAD_REQUEST", msg) }
    pub fn internal() -> Self { Self::new(500, "INTERNAL", "Erreur serveur") }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "ok": false, "error": self.msg, "details": self.msg, "code": self.code }))).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self { tracing::error!("db: {e}"); ApiError::internal() }
}
impl From<redis::RedisError> for ApiError {
    fn from(e: redis::RedisError) -> Self { tracing::error!("redis: {e}"); ApiError::internal() }
}
pub type ApiResult<T> = Result<T, ApiError>;
