use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use crate::auth::rbac::Permission;
use crate::auth::AuthUser;
use crate::error::AppResult;
use crate::handlers::{validate, ListQuery};
use crate::models::doctor::{CreateDoctor, Doctor};
use crate::models::{paginate, Paginated};
use crate::repositories::doctor as repo;
use crate::state::AppState;

pub async fn create(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateDoctor>,
) -> AppResult<impl IntoResponse> {
    user.require(Permission::DoctorWrite)?;
    validate(&body)?;
    let doctor = repo::create(&state.db, &body).await?;
    tracing::info!(doctor_id = %doctor.id, "created doctor");
    Ok((StatusCode::CREATED, Json(doctor)))
}

pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Paginated<Doctor>>> {
    user.require(Permission::DoctorRead)?;
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
) -> AppResult<Json<Doctor>> {
    user.require(Permission::DoctorRead)?;
    Ok(Json(repo::get(&state.db, id).await?))
}

pub async fn deactivate(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    user.require(Permission::DoctorWrite)?;
    repo::deactivate(&state.db, id).await?;
    tracing::info!(doctor_id = %id, "deactivated doctor");
    Ok(StatusCode::NO_CONTENT)
}
