use axum::extract::{Query, State};
use axum::Json;

use crate::auth::rbac::Permission;
use crate::auth::AuthUser;
use crate::error::AppResult;
use crate::models::audit::{AuditFilter, AuditLog};
use crate::models::{paginate, Paginated};
use crate::repositories::audit as repo;
use crate::state::AppState;

/// GET /api/v1/audit — query the audit trail (admins only).
pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
    Query(filter): Query<AuditFilter>,
) -> AppResult<Json<Paginated<AuditLog>>> {
    user.require(Permission::AuditRead)?;
    let (limit, offset) = paginate(filter.limit, filter.offset);
    let data = repo::list(&state.db, &filter, limit, offset).await?;
    Ok(Json(Paginated {
        data,
        limit,
        offset,
    }))
}
