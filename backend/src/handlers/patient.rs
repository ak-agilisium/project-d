use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::auth::rbac::Permission;
use crate::auth::AuthUser;
use crate::error::AppResult;
use crate::handlers::{ensure_one_of, validate, ListQuery};
use crate::models::patient::{
    CreatePatient, Patient, UpdatePatient, BLOOD_GROUPS, GENDERS, PATIENT_STATUSES,
};
use crate::models::{paginate, Paginated};
use crate::repositories::patient as repo;
use crate::state::AppState;

pub async fn create(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreatePatient>,
) -> AppResult<(StatusCode, Json<Patient>)> {
    user.require(Permission::PatientWrite)?;
    validate(&body)?;
    validate_patient_enums(body.gender.as_deref(), body.blood_group.as_deref(), None)?;

    // Duplicate prevention is enforced declaratively by UNIQUE indexes in the
    // schema (migration 0004). A collision surfaces as a Postgres unique
    // violation, which `AppError` maps to `409 Conflict` with a message naming
    // the duplicated field — no application-side pre-flight check needed.
    let patient = repo::create(&state.db, &body, Some(user.sub.as_str())).await?;
    tracing::info!(patient_id = %patient.id, mrn = %patient.patient_number, "created patient");
    Ok((StatusCode::CREATED, Json(patient)))
}

pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Paginated<Patient>>> {
    user.require(Permission::PatientRead)?;
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
) -> AppResult<Json<Patient>> {
    user.require(Permission::PatientRead)?;
    Ok(Json(repo::get(&state.db, id).await?))
}

pub async fn update(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdatePatient>,
) -> AppResult<Json<Patient>> {
    user.require(Permission::PatientWrite)?;
    validate(&body)?;
    validate_patient_enums(
        body.gender.as_deref(),
        body.blood_group.as_deref(),
        body.status.as_deref(),
    )?;
    Ok(Json(
        repo::update(&state.db, id, &body, Some(user.sub.as_str())).await?,
    ))
}

pub async fn delete(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    user.require(Permission::PatientDelete)?;
    repo::delete(&state.db, id).await?;
    tracing::info!(patient_id = %id, "deleted patient");
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------

fn validate_patient_enums(
    gender: Option<&str>,
    blood_group: Option<&str>,
    status: Option<&str>,
) -> AppResult<()> {
    if let Some(g) = gender {
        ensure_one_of("gender", g, GENDERS)?;
    }
    if let Some(b) = blood_group {
        ensure_one_of("blood_group", b, BLOOD_GROUPS)?;
    }
    if let Some(s) = status {
        ensure_one_of("status", s, PATIENT_STATUSES)?;
    }
    Ok(())
}
