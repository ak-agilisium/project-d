use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::error::AppResult;
use crate::models::audit::{AuditFilter, AuditLog};

/// Best-effort audit write for a sensitive action. Failures are logged, not
/// propagated — auditing must never break the primary request.
pub async fn record(
    pool: &PgPool,
    user: &AuthUser,
    action: &str,
    entity_type: &str,
    entity_id: &str,
) {
    let roles = user
        .roles()
        .iter()
        .map(|r| r.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let result = sqlx::query(
        r#"
        INSERT INTO audit_logs (actor_sub, actor_roles, action, entity_type, entity_id)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(&user.sub)
    .bind(roles)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, action, entity_type, entity_id, "failed to write audit log");
    }
}

/// Best-effort HTTP-level audit write (used by the audit middleware).
pub async fn record_http(
    pool: &PgPool,
    actor: Option<&AuthUser>,
    method: &str,
    path: &str,
    status_code: i32,
    ip: Option<&str>,
) {
    let (sub, roles) = match actor {
        Some(u) => (
            Some(u.sub.clone()),
            Some(
                u.roles()
                    .iter()
                    .map(|r| r.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        ),
        None => (None, None),
    };

    let result = sqlx::query(
        r#"
        INSERT INTO audit_logs
            (actor_sub, actor_roles, action, method, path, status_code, ip_address)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(sub)
    .bind(roles)
    .bind(format!("{method} {path}"))
    .bind(method)
    .bind(path)
    .bind(status_code)
    .bind(ip)
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, method, path, "failed to write HTTP audit log");
    }
}

pub async fn list(
    pool: &PgPool,
    filter: &AuditFilter,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<AuditLog>> {
    let rows = sqlx::query_as::<_, AuditLog>(
        r#"
        SELECT * FROM audit_logs
        WHERE ($1::text IS NULL OR actor_sub = $1)
          AND ($2::text IS NULL OR entity_type = $2)
          AND ($3::text IS NULL OR entity_id = $3)
        ORDER BY created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(filter.actor_sub.as_deref())
    .bind(filter.entity_type.as_deref())
    .bind(filter.entity_id.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
