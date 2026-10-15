use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use crate::auth::rbac::Permission;
use crate::auth::AuthUser;
use crate::error::AppResult;
use crate::handlers::{ensure_one_of, validate};
use crate::models::opd_visit::{
    CreateOpdVisit, OpdVisit, UpdateOpdVisit, VisitFilter, VISIT_STATUSES, VISIT_TYPES,
};
use crate::models::{paginate, Paginated};
use crate::repositories::opd_visit as repo;
use crate::state::AppState;

pub async fn create(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateOpdVisit>,
) -> AppResult<impl IntoResponse> {
    user.require(Permission::VisitWrite)?;
    validate(&body)?;
    if let Some(vt) = &body.visit_type {
        ensure_one_of("visit_type", vt, VISIT_TYPES)?;
    }
    let visit = repo::create(&state.db, &body, Some(user.sub.as_str())).await?;
    tracing::info!(visit_id = %visit.id, visit_number = %visit.visit_number, "opened OPD visit");
    Ok((StatusCode::CREATED, Json(visit)))
}

pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
    Query(filter): Query<VisitFilter>,
) -> AppResult<Json<Paginated<OpdVisit>>> {
    user.require(Permission::VisitRead)?;
    if let Some(s) = &filter.visit_status {
        ensure_one_of("visit_status", s, VISIT_STATUSES)?;
    }
    let (limit, offset) = paginate(filter.limit, filter.offset);
    let data = repo::list(
        &state.db,
        filter.patient_id,
        filter.doctor_id,
        filter.department_id,
        filter.visit_status.as_deref(),
        limit,
        offset,
    )
    .await?;
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
) -> AppResult<Json<OpdVisit>> {
    user.require(Permission::VisitRead)?;
    Ok(Json(repo::get(&state.db, id).await?))
}

pub async fn update(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateOpdVisit>,
) -> AppResult<Json<OpdVisit>> {
    user.require(Permission::VisitWrite)?;
    validate(&body)?;
    if let Some(vt) = &body.visit_type {
        ensure_one_of("visit_type", vt, VISIT_TYPES)?;
    }
    if let Some(s) = &body.visit_status {
        ensure_one_of("visit_status", s, VISIT_STATUSES)?;
    }
    Ok(Json(repo::update(&state.db, id, &body).await?))
}
