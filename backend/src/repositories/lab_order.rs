use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::lab_order::{CreateLabOrder, LabOrder, UpdateLabOrder};

pub async fn create(
    pool: &PgPool,
    visit_id: Uuid,
    patient_id: Uuid,
    input: &CreateLabOrder,
    actor: Option<&str>,
) -> AppResult<LabOrder> {
    let order = sqlx::query_as::<_, LabOrder>(
        r#"
        INSERT INTO lab_orders (visit_id, patient_id, test_code, test_name, ordered_by)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(visit_id)
    .bind(patient_id)
    .bind(input.test_code.as_deref())
    .bind(&input.test_name)
    .bind(actor)
    .fetch_one(pool)
    .await?;
    Ok(order)
}

pub async fn list_by_visit(pool: &PgPool, visit_id: Uuid) -> AppResult<Vec<LabOrder>> {
    let rows = sqlx::query_as::<_, LabOrder>(
        "SELECT * FROM lab_orders WHERE visit_id = $1 ORDER BY ordered_at",
    )
    .bind(visit_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Update status/result. `resulted_at` is stamped when status becomes completed.
pub async fn update(pool: &PgPool, id: Uuid, input: &UpdateLabOrder) -> AppResult<LabOrder> {
    sqlx::query_as::<_, LabOrder>(
        r#"
        UPDATE lab_orders SET
            status        = COALESCE($2, status),
            result_text   = COALESCE($3, result_text),
            result_s3_key = COALESCE($4, result_s3_key),
            resulted_at   = CASE WHEN $2 = 'completed' AND resulted_at IS NULL
                                 THEN now() ELSE resulted_at END
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(input.status.as_deref())
    .bind(input.result_text.as_deref())
    .bind(input.result_s3_key.as_deref())
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}
