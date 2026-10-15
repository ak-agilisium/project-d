use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

pub const LAB_STATUSES: &[&str] = &[
    "ordered",
    "collected",
    "in_progress",
    "completed",
    "cancelled",
];

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct LabOrder {
    pub id: Uuid,
    pub visit_id: Uuid,
    pub patient_id: Uuid,
    pub test_code: Option<String>,
    pub test_name: String,
    pub status: String,
    pub result_text: Option<String>,
    pub result_s3_key: Option<String>,
    pub ordered_by: Option<String>,
    pub ordered_at: DateTime<Utc>,
    pub resulted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateLabOrder {
    #[validate(length(max = 40))]
    pub test_code: Option<String>,
    #[validate(length(min = 1, max = 200))]
    pub test_name: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateLabOrder {
    /// One of [`LAB_STATUSES`].
    pub status: Option<String>,
    pub result_text: Option<String>,
    pub result_s3_key: Option<String>,
}
