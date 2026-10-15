use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::department::{CreateDepartment, Department};

pub async fn create(pool: &PgPool, input: &CreateDepartment) -> AppResult<Department> {
    let dept = sqlx::query_as::<_, Department>(
        r#"
        INSERT INTO departments (name, code, description)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(&input.name)
    .bind(input.code.as_deref())
    .bind(input.description.as_deref())
    .fetch_one(pool)
    .await?;
    Ok(dept)
}

pub async fn get(pool: &PgPool, id: Uuid) -> AppResult<Department> {
    sqlx::query_as::<_, Department>("SELECT * FROM departments WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn list(pool: &PgPool, limit: i64, offset: i64) -> AppResult<Vec<Department>> {
    let rows = sqlx::query_as::<_, Department>(
        "SELECT * FROM departments WHERE is_active = TRUE ORDER BY name LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
