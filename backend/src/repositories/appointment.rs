use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::appointment::{Appointment, CreateAppointment, UpdateAppointment};

pub async fn create(pool: &PgPool, input: &CreateAppointment) -> AppResult<Appointment> {
    let appointment = sqlx::query_as::<_, Appointment>(
        r#"
        INSERT INTO appointments (patient_id, doctor_id, scheduled_at, reason, notes)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(input.patient_id)
    .bind(input.doctor_id)
    .bind(input.scheduled_at)
    .bind(input.reason.as_deref())
    .bind(input.notes.as_deref())
    .fetch_one(pool)
    .await?;
    Ok(appointment)
}

pub async fn get(pool: &PgPool, id: Uuid) -> AppResult<Appointment> {
    sqlx::query_as::<_, Appointment>("SELECT * FROM appointments WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn list(
    pool: &PgPool,
    patient_id: Option<Uuid>,
    doctor_id: Option<Uuid>,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Appointment>> {
    let rows = sqlx::query_as::<_, Appointment>(
        r#"
        SELECT * FROM appointments
        WHERE ($1::uuid IS NULL OR patient_id = $1)
          AND ($2::uuid IS NULL OR doctor_id = $2)
          AND ($3::text IS NULL OR status = $3)
        ORDER BY scheduled_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(patient_id)
    .bind(doctor_id)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn update(pool: &PgPool, id: Uuid, input: &UpdateAppointment) -> AppResult<Appointment> {
    sqlx::query_as::<_, Appointment>(
        r#"
        UPDATE appointments SET
            doctor_id    = COALESCE($2, doctor_id),
            scheduled_at = COALESCE($3, scheduled_at),
            status       = COALESCE($4, status),
            reason       = COALESCE($5, reason),
            notes        = COALESCE($6, notes)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(input.doctor_id)
    .bind(input.scheduled_at)
    .bind(input.status.as_deref())
    .bind(input.reason.as_deref())
    .bind(input.notes.as_deref())
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

/// Convenience setter used by the "cancel" endpoint.
pub async fn set_status(pool: &PgPool, id: Uuid, status: &str) -> AppResult<Appointment> {
    sqlx::query_as::<_, Appointment>(
        "UPDATE appointments SET status = $2 WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(status)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}
