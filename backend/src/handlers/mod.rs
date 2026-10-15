//! HTTP handlers (controllers). Each handler: authorizes (RBAC), validates
//! input, delegates to a repository/service, and returns a typed response.
//! Errors propagate as [`crate::error::AppError`], which renders a consistent
//! JSON body — there is no ad-hoc error formatting in handlers.

pub mod appointment;
pub mod audit;
pub mod auth;
pub mod billing;
pub mod clinical;
pub mod department;
pub mod docs;
pub mod doctor;
pub mod health;
pub mod lab_order;
pub mod opd_visit;
pub mod patient;
pub mod patient_document;
pub mod patient_insurance;
pub mod prescription;
pub mod report;

use serde::Deserialize;
use validator::Validate;

use crate::error::{AppError, AppResult};

/// Common pagination query parameters.
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Validate a request DTO, mapping failures to `400 Validation`.
pub fn validate<T: Validate>(input: &T) -> AppResult<()> {
    input.validate().map_err(AppError::from)
}

/// Ensure `value` is one of `allowed`, else `400 Bad Request`.
pub fn ensure_one_of(field: &str, value: &str, allowed: &[&str]) -> AppResult<()> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "invalid {field}: '{value}' (allowed: {})",
            allowed.join(", ")
        )))
    }
}
