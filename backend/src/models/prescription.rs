use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Prescription {
    pub id: Uuid,
    pub visit_id: Uuid,
    pub patient_id: Uuid,
    pub drug_name: String,
    pub dosage: Option<String>,
    pub frequency: Option<String>,
    pub duration: Option<String>,
    pub route: Option<String>,
    pub instructions: Option<String>,
    pub prescribed_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePrescription {
    #[validate(length(min = 1, max = 200))]
    pub drug_name: String,
    pub dosage: Option<String>,
    pub frequency: Option<String>,
    pub duration: Option<String>,
    pub route: Option<String>,
    pub instructions: Option<String>,
}
