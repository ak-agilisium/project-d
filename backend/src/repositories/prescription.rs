use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::prescription::{CreatePrescription, Prescription};

pub async fn create(
    pool: &PgPool,
    visit_id: Uuid,
    patient_id: Uuid,
    input: &CreatePrescription,
    actor: Option<&str>,
) -> AppResult<Prescription> {
    let rx = sqlx::query_as::<_, Prescription>(
        r#"
        INSERT INTO prescriptions
            (visit_id, patient_id, drug_name, dosage, frequency, duration, route, instructions, prescribed_by)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        RETURNING *
        "#,
    )
    .bind(visit_id)
    .bind(patient_id)
    .bind(&input.drug_name)
    .bind(input.dosage.as_deref())
    .bind(input.frequency.as_deref())
    .bind(input.duration.as_deref())
    .bind(input.route.as_deref())
    .bind(input.instructions.as_deref())
    .bind(actor)
    .fetch_one(pool)
    .await?;
    Ok(rx)
}

pub async fn list_by_visit(pool: &PgPool, visit_id: Uuid) -> AppResult<Vec<Prescription>> {
    let rows = sqlx::query_as::<_, Prescription>(
        "SELECT * FROM prescriptions WHERE visit_id = $1 ORDER BY created_at",
    )
    .bind(visit_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let r = sqlx::query("DELETE FROM prescriptions WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    if r.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}
