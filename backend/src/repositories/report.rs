use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::report::{NewReport, Report};

pub async fn create(pool: &PgPool, input: &NewReport) -> AppResult<Report> {
    let report = sqlx::query_as::<_, Report>(
        r#"
        INSERT INTO reports
            (patient_id, appointment_id, report_type, title,
             s3_bucket, s3_key, content_type, file_size_bytes, uploaded_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(input.patient_id)
    .bind(input.appointment_id)
    .bind(&input.report_type)
    .bind(&input.title)
    .bind(&input.s3_bucket)
    .bind(&input.s3_key)
    .bind(&input.content_type)
    .bind(input.file_size_bytes)
    .bind(input.uploaded_by.as_deref())
    .fetch_one(pool)
    .await?;
    Ok(report)
}

pub async fn get(pool: &PgPool, id: Uuid) -> AppResult<Report> {
    sqlx::query_as::<_, Report>("SELECT * FROM reports WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn list_by_patient(
    pool: &PgPool,
    patient_id: Uuid,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Report>> {
    let rows = sqlx::query_as::<_, Report>(
        "SELECT * FROM reports WHERE patient_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(patient_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<Report> {
    sqlx::query_as::<_, Report>("DELETE FROM reports WHERE id = $1 RETURNING *")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}
