use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Visit-scoped clinical record (one per visit). Access is RBAC-restricted to
/// clinical roles — see [`crate::auth::rbac`].
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ClinicalRecord {
    pub id: Uuid,
    pub visit_id: Uuid,
    pub patient_id: Uuid,
    pub symptoms: Option<String>,
    pub diagnosis_code: Option<String>,
    pub diagnosis_description: Option<String>,
    pub allergies: Option<String>,
    pub current_medications: Option<String>,
    pub systolic_bp: Option<i32>,
    pub diastolic_bp: Option<i32>,
    pub pulse_bpm: Option<i32>,
    pub temperature_c: Option<Decimal>,
    pub respiratory_rate: Option<i32>,
    pub spo2: Option<i32>,
    pub height_cm: Option<Decimal>,
    pub weight_kg: Option<Decimal>,
    pub clinical_notes: Option<String>,
    pub treatment_plan: Option<String>,
    pub follow_up_date: Option<NaiveDate>,
    pub recorded_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Upsert payload for a visit's clinical record.
#[derive(Debug, Deserialize, Validate)]
pub struct UpsertClinicalRecord {
    pub symptoms: Option<String>,
    #[validate(length(max = 20))]
    pub diagnosis_code: Option<String>,
    pub diagnosis_description: Option<String>,
    pub allergies: Option<String>,
    pub current_medications: Option<String>,
    pub systolic_bp: Option<i32>,
    pub diastolic_bp: Option<i32>,
    pub pulse_bpm: Option<i32>,
    pub temperature_c: Option<Decimal>,
    pub respiratory_rate: Option<i32>,
    pub spo2: Option<i32>,
    pub height_cm: Option<Decimal>,
    pub weight_kg: Option<Decimal>,
    pub clinical_notes: Option<String>,
    pub treatment_plan: Option<String>,
    pub follow_up_date: Option<NaiveDate>,
}
