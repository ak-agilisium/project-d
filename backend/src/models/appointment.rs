use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Allowed appointment statuses (mirrors the DB CHECK constraint).
pub const APPOINTMENT_STATUSES: &[&str] = &[
    "scheduled",
    "checked_in",
    "completed",
    "cancelled",
    "no_show",
];

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Appointment {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub doctor_id: Uuid,
    pub scheduled_at: DateTime<Utc>,
    pub status: String,
    pub reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAppointment {
    pub patient_id: Uuid,
    pub doctor_id: Uuid,
    pub scheduled_at: DateTime<Utc>,
    #[validate(length(max = 500))]
    pub reason: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAppointment {
    pub doctor_id: Option<Uuid>,
    pub scheduled_at: Option<DateTime<Utc>>,
    /// One of [`APPOINTMENT_STATUSES`].
    pub status: Option<String>,
    #[validate(length(max = 500))]
    pub reason: Option<String>,
    pub notes: Option<String>,
}

/// Query filters for listing appointments.
#[derive(Debug, Deserialize)]
pub struct AppointmentFilter {
    pub patient_id: Option<Uuid>,
    pub doctor_id: Option<Uuid>,
    pub status: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}
