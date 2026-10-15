use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::doctor::{CreateDoctor, Doctor};

pub async fn create(pool: &PgPool, input: &CreateDoctor) -> AppResult<Doctor> {
    let doctor = sqlx::query_as::<_, Doctor>(
        r#"
        INSERT INTO doctors
            (first_name, last_name, specialization, department, consultation_fee, email, phone)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(&input.first_name)
    .bind(&input.last_name)
    .bind(&input.specialization)
    .bind(input.department.as_deref())
    .bind(input.consultation_fee.unwrap_or(Decimal::ZERO))
    .bind(input.email.as_deref())
    .bind(input.phone.as_deref())
    .fetch_one(pool)
    .await?;
    Ok(doctor)
}

pub async fn get(pool: &PgPool, id: Uuid) -> AppResult<Doctor> {
    sqlx::query_as::<_, Doctor>("SELECT * FROM doctors WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn list(pool: &PgPool, limit: i64, offset: i64) -> AppResult<Vec<Doctor>> {
    let rows = sqlx::query_as::<_, Doctor>(
        "SELECT * FROM doctors WHERE is_active = TRUE ORDER BY last_name, first_name LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn deactivate(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query("UPDATE doctors SET is_active = FALSE WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}
