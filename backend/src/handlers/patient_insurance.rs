use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use crate::auth::rbac::Permission;
use crate::auth::AuthUser;
use crate::error::AppResult;
use crate::handlers::{ensure_one_of, validate};
use crate::models::patient_insurance::{
    CreatePatientInsurance, PatientInsurance, UpdatePatientInsurance, COVERAGE_STATUSES,
};
use crate::repositories::patient_insurance as repo;
use crate::state::AppState;

/// POST /api/v1/patients/{patient_id}/insurance
pub async fn create(
    user: AuthUser,
    State(state): State<AppState>,
    Path(patient_id): Path<Uuid>,
    Json(body): Json<CreatePatientInsurance>,
) -> AppResult<impl IntoResponse> {
    user.require(Permission::InsuranceWrite)?;
    validate(&body)?;
    let ins = repo::create(&state.db, patient_id, &body).await?;
    Ok((StatusCode::CREATED, Json(ins)))
}

/// GET /api/v1/patients/{patient_id}/insurance
pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
    Path(patient_id): Path<Uuid>,
) -> AppResult<Json<Vec<PatientInsurance>>> {
    user.require(Permission::InsuranceRead)?;
    Ok(Json(repo::list_by_patient(&state.db, patient_id).await?))
}

/// PATCH /api/v1/insurance/{id}
pub async fn update(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdatePatientInsurance>,
) -> AppResult<Json<PatientInsurance>> {
    user.require(Permission::InsuranceWrite)?;
    validate(&body)?;
    if let Some(s) = &body.coverage_status {
        ensure_one_of("coverage_status", s, COVERAGE_STATUSES)?;
    }
    Ok(Json(repo::update(&state.db, id, &body).await?))
}
