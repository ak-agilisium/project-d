//! Domain models: database row types (`FromRow`) and request/response DTOs.

pub mod appointment;
pub mod audit;
pub mod billing;
pub mod clinical;
pub mod department;
pub mod doctor;
pub mod lab_order;
pub mod opd_visit;
pub mod patient;
pub mod patient_document;
pub mod patient_insurance;
pub mod prescription;
pub mod report;

use serde::Serialize;

/// Generic paginated list envelope.
#[derive(Debug, Serialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub limit: i64,
    pub offset: i64,
}

/// Clamp caller-supplied pagination into sane bounds.
pub fn paginate(limit: Option<i64>, offset: Option<i64>) -> (i64, i64) {
    let limit = limit.unwrap_or(50).clamp(1, 200);
    let offset = offset.unwrap_or(0).max(0);
    (limit, offset)
}
