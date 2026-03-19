use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// Application error type with HTTP status mapping.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // ─── 400 Bad Request ───
    #[error("validation error: {0}")]
    Validation(String),

    #[error("invalid request: {0}")]
    BadRequest(String),

    // ─── 401 Unauthorized ───
    #[error("invalid login or password")]
    InvalidLoginOrPassword,

    #[error("access token expired")]
    AccessTokenExpired,

    #[error("refresh token expired")]
    RefreshTokenExpired,

    #[error("session not found")]
    SessionNotFound,

    #[error("csrf token invalid or expired")]
    CsrfTokenInvalid,

    #[error("authentication required")]
    Unauthorized,

    // ─── 403 Forbidden ───
    #[error("two-factor authentication required")]
    TwoFactorRequired,

    #[error("insufficient permissions")]
    Forbidden,

    // ─── 404 Not Found ───
    #[error("user not found")]
    UserNotFound,

    #[error("vault not found")]
    VaultNotFound,

    #[error("folder not found")]
    FolderNotFound,

    #[error("item not found")]
    ItemNotFound,

    #[error("snapshot not found")]
    SnapshotNotFound,

    #[error("attachment not found")]
    AttachmentNotFound,

    #[error("send not found")]
    SendNotFound,

    #[error("resource not found")]
    NotFound(String),

    // ─── 409 Conflict ───
    #[error("login already taken")]
    LoginAlreadyTaken,

    #[error("email already taken")]
    EmailAlreadyTaken,

    #[error("vault name already exists")]
    VaultNameTaken,

    #[error("folder name already exists in this location")]
    FolderNameTaken,

    // ─── 422 Unprocessable Entity ───
    #[error("password does not meet complexity requirements: {0}")]
    PasswordComplexityFailed(String),

    // ─── 413 Payload Too Large ───
    #[error("file too large: {0}")]
    PayloadTooLarge(String),

    // ─── 429 Too Many Requests ───
    #[error("too many requests")]
    TooManyRequests,

    // ─── 500 Internal Server Error ───
    #[error("internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

/// Error response body sent to clients.
#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            AppError::InvalidLoginOrPassword => (StatusCode::UNAUTHORIZED, "INVALID_LOGIN_OR_PASSWORD"),
            AppError::AccessTokenExpired => (StatusCode::UNAUTHORIZED, "ACCESS_TOKEN_EXPIRED"),
            AppError::RefreshTokenExpired => (StatusCode::UNAUTHORIZED, "REFRESH_TOKEN_EXPIRED"),
            AppError::SessionNotFound => (StatusCode::UNAUTHORIZED, "SESSION_NOT_FOUND"),
            AppError::CsrfTokenInvalid => (StatusCode::UNAUTHORIZED, "CSRF_TOKEN_INVALID"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            AppError::TwoFactorRequired => (StatusCode::FORBIDDEN, "TWO_FACTOR_REQUIRED"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            AppError::UserNotFound => (StatusCode::NOT_FOUND, "USER_NOT_FOUND"),
            AppError::VaultNotFound => (StatusCode::NOT_FOUND, "VAULT_NOT_FOUND"),
            AppError::FolderNotFound => (StatusCode::NOT_FOUND, "FOLDER_NOT_FOUND"),
            AppError::ItemNotFound => (StatusCode::NOT_FOUND, "ITEM_NOT_FOUND"),
            AppError::SnapshotNotFound => (StatusCode::NOT_FOUND, "SNAPSHOT_NOT_FOUND"),
            AppError::AttachmentNotFound => (StatusCode::NOT_FOUND, "ATTACHMENT_NOT_FOUND"),
            AppError::SendNotFound => (StatusCode::NOT_FOUND, "SEND_NOT_FOUND"),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            AppError::LoginAlreadyTaken => (StatusCode::CONFLICT, "LOGIN_ALREADY_TAKEN"),
            AppError::EmailAlreadyTaken => (StatusCode::CONFLICT, "EMAIL_ALREADY_TAKEN"),
            AppError::VaultNameTaken => (StatusCode::CONFLICT, "VAULT_NAME_TAKEN"),
            AppError::FolderNameTaken => (StatusCode::CONFLICT, "FOLDER_NAME_TAKEN"),
            AppError::PasswordComplexityFailed(_) => (StatusCode::UNPROCESSABLE_ENTITY, "PASSWORD_COMPLEXITY_FAILED"),
            AppError::PayloadTooLarge(_) => (StatusCode::PAYLOAD_TOO_LARGE, "PAYLOAD_TOO_LARGE"),
            AppError::TooManyRequests => (StatusCode::TOO_MANY_REQUESTS, "TOO_MANY_REQUESTS"),
            AppError::Internal(_) | AppError::Anyhow(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };

        // Log 5xx errors with full details server-side
        if status.is_server_error() {
            tracing::error!(error = %self, "Internal server error");
        }

        // Never leak internal error details to clients
        let message = match &self {
            AppError::Internal(_) | AppError::Anyhow(_) => {
                "An internal error occurred".to_string()
            }
            other => other.to_string(),
        };

        let body = ErrorBody { code, message };

        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!(error = %e, "Database error");
        AppError::Internal("database error".to_string())
    }
}

impl From<redis::RedisError> for AppError {
    fn from(e: redis::RedisError) -> Self {
        tracing::error!(error = %e, "Redis error");
        AppError::Internal("cache error".to_string())
    }
}
