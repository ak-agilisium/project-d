//! Unified application error type and its HTTP representation.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// The error type returned by handlers, services and repositories.
///
/// Implements [`IntoResponse`] so any handler can return `Result<T, AppError>`
/// and get a consistent JSON error body.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("resource not found")]
    NotFound,

    #[error("{0}")]
    BadRequest(String),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("authentication required")]
    Unauthorized(String),

    #[error("access denied")]
    Forbidden,

    #[error("{0}")]
    Conflict(String),

    #[error("database error")]
    Database(#[from] sqlx::Error),

    #[error("upstream AWS error: {0}")]
    Aws(String),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) | AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Database(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND,
            AppError::Database(e) if is_unique_violation(e) => StatusCode::CONFLICT,
            AppError::Database(_) | AppError::Aws(_) | AppError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    /// A stable, machine-readable error code for clients.
    fn code(&self) -> &'static str {
        match self {
            AppError::NotFound => "not_found",
            AppError::BadRequest(_) => "bad_request",
            AppError::Validation(_) => "validation_error",
            AppError::Unauthorized(_) => "unauthorized",
            AppError::Forbidden => "forbidden",
            AppError::Conflict(_) => "conflict",
            AppError::Database(sqlx::Error::RowNotFound) => "not_found",
            AppError::Database(e) if is_unique_violation(e) => "conflict",
            AppError::Database(_) => "database_error",
            AppError::Aws(_) => "aws_error",
            AppError::Internal(_) => "internal_error",
        }
    }

    /// Client-safe message. Internal errors are redacted to avoid leaking
    /// implementation details; the full error is logged instead.
    fn public_message(&self) -> String {
        match self {
            AppError::Database(sqlx::Error::RowNotFound) => "resource not found".to_string(),
            AppError::Database(e) if is_unique_violation(e) => unique_violation_message(e),
            AppError::Database(_) => "an internal database error occurred".to_string(),
            AppError::Aws(_) => "an upstream service error occurred".to_string(),
            AppError::Internal(_) => "an internal error occurred".to_string(),
            other => other.to_string(),
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();

        // Log server-side faults with full detail; client faults at debug.
        if status.is_server_error() {
            tracing::error!(error = ?self, "request failed with server error");
        } else {
            tracing::debug!(error = %self, "request failed with client error");
        }

        let body = ErrorBody {
            error: ErrorDetail {
                code: self.code(),
                message: self.public_message(),
            },
        };

        (status, Json(body)).into_response()
    }
}

/// Convenience alias used throughout the crate.
pub type AppResult<T> = Result<T, AppError>;

/// True when a SQLx error is a Postgres unique-constraint violation (SQLSTATE
/// 23505) — i.e. a duplicate rejected by a schema-level UNIQUE index.
fn is_unique_violation(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .is_some_and(|db| db.is_unique_violation())
}

/// A human-friendly message for a unique-violation, tailored by the constraint
/// that was hit so clients learn *what* was duplicated.
fn unique_violation_message(e: &sqlx::Error) -> String {
    match e.as_database_error().and_then(|db| db.constraint()) {
        Some("uq_patients_identity") => {
            "a patient with the same name and date of birth already exists".to_string()
        }
        Some("uq_patients_email") => "a patient with this email already exists".to_string(),
        Some("uq_patients_phone") => {
            "a patient with this phone number already exists".to_string()
        }
        _ => "a record with the same unique value already exists".to_string(),
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        AppError::Validation(errors.to_string())
    }
}
