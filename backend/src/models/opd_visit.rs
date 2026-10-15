use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

pub const VISIT_TYPES: &[&str] = &["new", "follow_up", "emergency", "walk_in"];
pub const VISIT_STATUSES: &[&str] = &["open", "in_consultation", "completed", "cancelled"];

/// An OPD visit: one consultation episode for a patient, linked by `patient_id`.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct OpdVisit {
    pub id: Uuid,
    pub visit_number: String,
    pub patient_id: Uuid,
    pub department_id: Option<Uuid>,
    pub doctor_id: Option<Uuid>,
    pub appointment_id: Option<Uuid>,
    pub visit_date: DateTime<Utc>,
    pub visit_type: String,
    pub chief_complaint: Option<String>,
    pub visit_status: String,
    pub referred_by: Option<String>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateOpdVisit {
    pub patient_id: Uuid,
    pub department_id: Option<Uuid>,
    pub doctor_id: Option<Uuid>,
    pub appointment_id: Option<Uuid>,
    /// One of [`VISIT_TYPES`]; defaults to `new`.
    pub visit_type: Option<String>,
    #[validate(length(max = 1000))]
    pub chief_complaint: Option<String>,
    pub referred_by: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateOpdVisit {
    pub department_id: Option<Uuid>,
    pub doctor_id: Option<Uuid>,
    pub visit_type: Option<String>,
    /// One of [`VISIT_STATUSES`].
    pub visit_status: Option<String>,
    #[validate(length(max = 1000))]
    pub chief_complaint: Option<String>,
    pub referred_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VisitFilter {
    pub patient_id: Option<Uuid>,
    pub doctor_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub visit_status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
