use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

pub const COVERAGE_STATUSES: &[&str] = &["active", "inactive", "expired"];

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PatientInsurance {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub payer_name: String,
    pub policy_number: Option<String>,
    pub member_id: Option<String>,
    pub coverage_start_date: Option<NaiveDate>,
    pub coverage_end_date: Option<NaiveDate>,
    pub preauthorization_number: Option<String>,
    pub coverage_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePatientInsurance {
    #[validate(length(min = 1, max = 160))]
    pub payer_name: String,
    pub policy_number: Option<String>,
    pub member_id: Option<String>,
    pub coverage_start_date: Option<NaiveDate>,
    pub coverage_end_date: Option<NaiveDate>,
    pub preauthorization_number: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePatientInsurance {
    #[validate(length(min = 1, max = 160))]
    pub payer_name: Option<String>,
    pub policy_number: Option<String>,
    pub member_id: Option<String>,
    pub coverage_start_date: Option<NaiveDate>,
    pub coverage_end_date: Option<NaiveDate>,
    pub preauthorization_number: Option<String>,
    /// One of [`COVERAGE_STATUSES`].
    pub coverage_status: Option<String>,
}
