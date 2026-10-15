use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// Consent form / supporting document. File in S3, metadata here.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PatientDocument {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub visit_id: Option<Uuid>,
    pub document_type: String,
    pub title: String,
    pub s3_bucket: String,
    pub s3_key: String,
    pub content_type: String,
    pub file_size_bytes: i64,
    pub uploaded_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct NewPatientDocument {
    pub patient_id: Uuid,
    pub visit_id: Option<Uuid>,
    pub document_type: String,
    pub title: String,
    pub s3_bucket: String,
    pub s3_key: String,
    pub content_type: String,
    pub file_size_bytes: i64,
    pub uploaded_by: Option<String>,
}
