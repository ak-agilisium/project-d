use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Allowed values (mirrored by DB CHECK constraints where present).
pub const GENDERS: &[&str] = &["male", "female", "other", "unknown"];
pub const PATIENT_STATUSES: &[&str] = &["active", "inactive", "merged"];
pub const BLOOD_GROUPS: &[&str] = &["A+", "A-", "B+", "B-", "AB+", "AB-", "O+", "O-", "unknown"];

/// The patient master record: demographic & contact identity only. Visits,
/// clinical encounters and billing live in their own tables, linked by id.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Patient {
    pub id: Uuid,
    /// Human-readable medical record number (MRN), e.g. `MRN001000`.
    pub patient_number: String,
    pub cognito_sub: Option<String>,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub phone: Option<String>,
    pub alternate_phone: Option<String>,
    pub email: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub blood_group: Option<String>,
    pub marital_status: Option<String>,
    pub preferred_language: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub emergency_contact_relationship: Option<String>,
    pub patient_photo_url: Option<String>,
    /// Lifecycle status: `active`, `inactive`, or `merged`.
    pub status: String,
    /// When `status = merged`, the surviving patient this record was merged into.
    pub merged_into: Option<Uuid>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePatient {
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    #[validate(length(max = 100))]
    pub middle_name: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    #[validate(length(max = 32))]
    pub phone: Option<String>,
    #[validate(length(max = 32))]
    pub alternate_phone: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    #[validate(length(max = 20))]
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub blood_group: Option<String>,
    pub marital_status: Option<String>,
    pub preferred_language: Option<String>,
    pub emergency_contact_name: Option<String>,
    #[validate(length(max = 32))]
    pub emergency_contact_phone: Option<String>,
    pub emergency_contact_relationship: Option<String>,
    #[validate(length(max = 1024))]
    pub patient_photo_url: Option<String>,
    /// Optionally link this record to a Cognito identity (its `sub`).
    pub cognito_sub: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePatient {
    #[validate(length(min = 1, max = 100))]
    pub first_name: Option<String>,
    #[validate(length(max = 100))]
    pub middle_name: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub last_name: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    #[validate(length(max = 32))]
    pub phone: Option<String>,
    #[validate(length(max = 32))]
    pub alternate_phone: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    #[validate(length(max = 20))]
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub blood_group: Option<String>,
    pub marital_status: Option<String>,
    pub preferred_language: Option<String>,
    pub emergency_contact_name: Option<String>,
    #[validate(length(max = 32))]
    pub emergency_contact_phone: Option<String>,
    pub emergency_contact_relationship: Option<String>,
    #[validate(length(max = 1024))]
    pub patient_photo_url: Option<String>,
    /// `active` | `inactive` | `merged`.
    pub status: Option<String>,
}

