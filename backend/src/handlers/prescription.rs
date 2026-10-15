use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use crate::auth::rbac::Permission;
use crate::auth::AuthUser;
use crate::error::AppResult;
use crate::handlers::validate;
use crate::models::prescription::{CreatePrescription, Prescription};
use crate::repositories::{opd_visit as visit_repo, prescription as repo};
use crate::state::AppState;

/// POST /api/v1/visits/{visit_id}/prescriptions
pub async fn create(
    user: AuthUser,
    State(state): State<AppState>,
    Path(visit_id): Path<Uuid>,
    Json(body): Json<CreatePrescription>,
) -> AppResult<impl IntoResponse> {
    user.require(Permission::PrescriptionWrite)?;
    validate(&body)?;
    let visit = visit_repo::get(&state.db, visit_id).await?;
    let rx = repo::create(
        &state.db,
        visit_id,
        visit.patient_id,
        &body,
        Some(user.sub.as_str()),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(rx)))
}

/// GET /api/v1/visits/{visit_id}/prescriptions
pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
    Path(visit_id): Path<Uuid>,
) -> AppResult<Json<Vec<Prescription>>> {
    user.require(Permission::PrescriptionRead)?;
    Ok(Json(repo::list_by_visit(&state.db, visit_id).await?))
}

/// DELETE /api/v1/prescriptions/{id}
pub async fn delete(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    user.require(Permission::PrescriptionWrite)?;
    repo::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
