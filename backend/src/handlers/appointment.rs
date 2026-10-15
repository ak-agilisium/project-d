use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use crate::auth::rbac::Permission;
use crate::auth::AuthUser;
use crate::error::AppResult;
use crate::handlers::{ensure_one_of, validate};
use crate::models::appointment::{
    Appointment, AppointmentFilter, CreateAppointment, UpdateAppointment, APPOINTMENT_STATUSES,
};
use crate::models::{paginate, Paginated};
use crate::repositories::appointment as repo;
use crate::state::AppState;

pub async fn create(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateAppointment>,
) -> AppResult<impl IntoResponse> {
    user.require(Permission::AppointmentWrite)?;
    validate(&body)?;
    let appointment = repo::create(&state.db, &body).await?;
    tracing::info!(appointment_id = %appointment.id, "booked appointment");
    Ok((StatusCode::CREATED, Json(appointment)))
}

pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
    Query(filter): Query<AppointmentFilter>,
) -> AppResult<Json<Paginated<Appointment>>> {
    user.require(Permission::AppointmentRead)?;
    if let Some(status) = &filter.status {
        ensure_one_of("status", status, APPOINTMENT_STATUSES)?;
    }
    let (limit, offset) = paginate(filter.limit, filter.offset);
    let data = repo::list(
        &state.db,
        filter.patient_id,
        filter.doctor_id,
        filter.status.as_deref(),
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
) -> AppResult<Json<Appointment>> {
    user.require(Permission::AppointmentRead)?;
    Ok(Json(repo::get(&state.db, id).await?))
}

pub async fn update(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateAppointment>,
) -> AppResult<Json<Appointment>> {
    user.require(Permission::AppointmentWrite)?;
    validate(&body)?;
    if let Some(status) = &body.status {
        ensure_one_of("status", status, APPOINTMENT_STATUSES)?;
    }
    Ok(Json(repo::update(&state.db, id, &body).await?))
}

pub async fn cancel(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Appointment>> {
    user.require(Permission::AppointmentWrite)?;
    let appointment = repo::set_status(&state.db, id, "cancelled").await?;
    tracing::info!(appointment_id = %id, "cancelled appointment");
    Ok(Json(appointment))
}
