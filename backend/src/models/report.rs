use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// Metadata pointer to a report file stored in S3. The bytes live in S3 only.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Report {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub appointment_id: Option<Uuid>,
    pub report_type: String,
    pub title: String,
    pub s3_bucket: String,
    pub s3_key: String,
    pub content_type: String,
    pub file_size_bytes: i64,
    pub uploaded_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Values needed to persist a report record after the file lands in S3.
#[derive(Debug)]
pub struct NewReport {
    pub patient_id: Uuid,
    pub appointment_id: Option<Uuid>,
    pub report_type: String,
    pub title: String,
    pub s3_bucket: String,
    pub s3_key: String,
    pub content_type: String,
    pub file_size_bytes: i64,
    pub uploaded_by: Option<String>,
}

/// Response body for a report download request.
#[derive(Debug, Serialize)]
pub struct ReportDownload {
    pub url: String,
    pub expires_in_secs: u64,
}
