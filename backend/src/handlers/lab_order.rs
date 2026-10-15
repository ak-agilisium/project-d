use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use crate::auth::rbac::Permission;
use crate::auth::AuthUser;
use crate::error::AppResult;
use crate::handlers::{ensure_one_of, validate};
use crate::models::lab_order::{CreateLabOrder, LabOrder, UpdateLabOrder, LAB_STATUSES};
use crate::repositories::{lab_order as repo, opd_visit as visit_repo};
use crate::state::AppState;

/// POST /api/v1/visits/{visit_id}/lab-orders
pub async fn create(
    user: AuthUser,
    State(state): State<AppState>,
    Path(visit_id): Path<Uuid>,
    Json(body): Json<CreateLabOrder>,
) -> AppResult<impl IntoResponse> {
    user.require(Permission::LabWrite)?;
    validate(&body)?;
    let visit = visit_repo::get(&state.db, visit_id).await?;
    let order = repo::create(
        &state.db,
        visit_id,
        visit.patient_id,
        &body,
        Some(user.sub.as_str()),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(order)))
}

/// GET /api/v1/visits/{visit_id}/lab-orders
pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
    Path(visit_id): Path<Uuid>,
) -> AppResult<Json<Vec<LabOrder>>> {
    user.require(Permission::LabRead)?;
    Ok(Json(repo::list_by_visit(&state.db, visit_id).await?))
}

/// PATCH /api/v1/lab-orders/{id} — update status / attach results.
pub async fn update(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateLabOrder>,
) -> AppResult<Json<LabOrder>> {
    user.require(Permission::LabWrite)?;
    validate(&body)?;
    if let Some(s) = &body.status {
        ensure_one_of("status", s, LAB_STATUSES)?;
    }
    Ok(Json(repo::update(&state.db, id, &body).await?))
}
