use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use crate::auth::rbac::Permission;
use crate::auth::AuthUser;
use crate::error::AppResult;
use crate::handlers::{validate, ListQuery};
use crate::models::department::{CreateDepartment, Department};
use crate::models::{paginate, Paginated};
use crate::repositories::department as repo;
use crate::state::AppState;

pub async fn create(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateDepartment>,
) -> AppResult<impl IntoResponse> {
    user.require(Permission::DepartmentWrite)?;
    validate(&body)?;
    let dept = repo::create(&state.db, &body).await?;
    Ok((StatusCode::CREATED, Json(dept)))
}

pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Paginated<Department>>> {
    user.require(Permission::DepartmentRead)?;
    let (limit, offset) = paginate(q.limit, q.offset);
    let data = repo::list(&state.db, limit, offset).await?;
    Ok(Json(Paginated {
        data,
        limit,
        offset,
    }))
}

pub async fn get(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Department>> {
    user.require(Permission::DepartmentRead)?;
    Ok(Json(repo::get(&state.db, id).await?))
}
