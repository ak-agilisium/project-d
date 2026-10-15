//! Data-access layer. Each module owns the SQL for one aggregate and returns
//! domain models or [`crate::error::AppError`] — the single place DB errors are
//! produced, keeping SQL out of the handlers.

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
