use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

use crate::auth::rbac::Permission;
use crate::auth::AuthUser;
use crate::error::AppResult;
use crate::handlers::validate;
use crate::models::clinical::{ClinicalRecord, UpsertClinicalRecord};
use crate::repositories::{audit, clinical as repo, opd_visit as visit_repo};
use crate::state::AppState;

/// GET /api/v1/visits/{visit_id}/clinical — read the visit's clinical record.
/// Restricted to clinical roles (super_admin, doctor).
pub async fn get(
    user: AuthUser,
    State(state): State<AppState>,
    Path(visit_id): Path<Uuid>,
) -> AppResult<Json<ClinicalRecord>> {
    user.require(Permission::ClinicalRead)?;
    let record = repo::get_by_visit(&state.db, visit_id).await?;
    audit::record(
        &state.db,
        &user,
        "clinical.view",
        "visit",
        &visit_id.to_string(),
    )
    .await;
    Ok(Json(record))
}

/// PUT /api/v1/visits/{visit_id}/clinical — create or update the clinical record.
pub async fn upsert(
    user: AuthUser,
    State(state): State<AppState>,
    Path(visit_id): Path<Uuid>,
    Json(body): Json<UpsertClinicalRecord>,
) -> AppResult<Json<ClinicalRecord>> {
    user.require(Permission::ClinicalWrite)?;
    validate(&body)?;
    // Resolve (and validate existence of) the visit to derive patient_id.
    let visit = visit_repo::get(&state.db, visit_id).await?;
    let record = repo::upsert(
        &state.db,
        visit_id,
        visit.patient_id,
        &body,
        Some(user.sub.as_str()),
    )
    .await?;
    tracing::info!(visit_id = %visit_id, "clinical record upserted");
    Ok(Json(record))
}
