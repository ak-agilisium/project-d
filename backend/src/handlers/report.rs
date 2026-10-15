use axum::extract::multipart::MultipartError;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use crate::auth::rbac::Permission;
use crate::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::handlers::ListQuery;
use crate::models::report::{NewReport, Report, ReportDownload};
use crate::models::{paginate, Paginated};
use crate::repositories::report as repo;
use crate::state::AppState;

/// POST /api/v1/reports — upload a report file to S3 and persist its metadata.
///
/// Expects a `multipart/form-data` body with:
///   - `file`           (required) the report file
///   - `patient_id`     (required) UUID of the patient
///   - `appointment_id` (optional) UUID of a related appointment
///   - `report_type`    (optional) e.g. `lab`, `radiology`, `prescription`
///   - `title`          (optional) human-readable title
pub async fn upload(
    user: AuthUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<impl IntoResponse> {
    user.require(Permission::ReportWrite)?;

    let mut patient_id: Option<Uuid> = None;
    let mut appointment_id: Option<Uuid> = None;
    let mut report_type: Option<String> = None;
    let mut title: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut content_type = "application/octet-stream".to_string();

    while let Some(field) = multipart.next_field().await.map_err(multipart_err)? {
        match field.name().unwrap_or_default() {
            "patient_id" => {
                let v = field.text().await.map_err(multipart_err)?;
                patient_id = Some(parse_uuid("patient_id", &v)?);
            }
            "appointment_id" => {
                let v = field.text().await.map_err(multipart_err)?;
                if !v.trim().is_empty() {
                    appointment_id = Some(parse_uuid("appointment_id", &v)?);
                }
            }
            "report_type" => report_type = Some(field.text().await.map_err(multipart_err)?),
            "title" => title = Some(field.text().await.map_err(multipart_err)?),
            "file" => {
                file_name = field.file_name().map(str::to_string);
                if let Some(ct) = field.content_type() {
                    content_type = ct.to_string();
                }
                let data = field.bytes().await.map_err(multipart_err)?;
                file_bytes = Some(data.to_vec());
            }
            _ => {
                // Drain and ignore unknown parts.
                let _ = field.bytes().await;
            }
        }
    }

    let patient_id =
        patient_id.ok_or_else(|| AppError::BadRequest("'patient_id' is required".into()))?;
    let bytes =
        file_bytes.ok_or_else(|| AppError::BadRequest("a 'file' part is required".into()))?;

    if bytes.is_empty() {
        return Err(AppError::BadRequest("the uploaded file is empty".into()));
    }
    if bytes.len() > state.config.app.max_upload_bytes {
        return Err(AppError::BadRequest(
            "the uploaded file exceeds the maximum allowed size".into(),
        ));
    }

    let report_type = report_type.unwrap_or_else(|| "general".to_string());
    let title = title
        .or_else(|| file_name.clone())
        .unwrap_or_else(|| "report".to_string());

    let file_id = Uuid::new_v4();
    let safe_name = file_name
        .as_deref()
        .map(sanitize)
        .unwrap_or_else(|| "file".to_string());
    let s3_key = format!("reports/{patient_id}/{file_id}-{safe_name}");
    let size = bytes.len() as i64;

    // Upload to S3 first; only record metadata if the object landed.
    state.s3.put_object(&s3_key, bytes, &content_type).await?;

    let new = NewReport {
        patient_id,
        appointment_id,
        report_type,
        title,
        s3_bucket: state.s3.bucket().to_string(),
        s3_key,
        content_type,
        file_size_bytes: size,
        uploaded_by: Some(user.sub.clone()),
    };
    let report = repo::create(&state.db, &new).await?;
    tracing::info!(report_id = %report.id, patient_id = %patient_id, key = %report.s3_key, "uploaded report to S3");

    Ok((StatusCode::CREATED, Json(report)))
}

/// GET /api/v1/patients/{patient_id}/reports — list a patient's reports.
pub async fn list_by_patient(
    user: AuthUser,
    State(state): State<AppState>,
    Path(patient_id): Path<Uuid>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Paginated<Report>>> {
    user.require(Permission::ReportRead)?;
    let (limit, offset) = paginate(q.limit, q.offset);
    let data = repo::list_by_patient(&state.db, patient_id, limit, offset).await?;
    Ok(Json(Paginated {
        data,
        limit,
        offset,
    }))
}

/// GET /api/v1/reports/{id}/download — presigned S3 URL for the report file.
pub async fn download(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ReportDownload>> {
    user.require(Permission::ReportRead)?;
    let report = repo::get(&state.db, id).await?;
    let presigned = state.s3.presign_get(&report.s3_key).await?;
    Ok(Json(ReportDownload {
        url: presigned.url,
        expires_in_secs: presigned.expires_in_secs,
    }))
}

/// DELETE /api/v1/reports/{id} — remove the record and the S3 object.
pub async fn delete(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    user.require(Permission::ReportDelete)?;
    let report = repo::delete(&state.db, id).await?;
    // Best-effort cleanup; the metadata row is already gone.
    if let Err(e) = state.s3.delete_object(&report.s3_key).await {
        tracing::warn!(error = %e, key = %report.s3_key, "failed to delete S3 object for removed report");
    }
    Ok(StatusCode::NO_CONTENT)
}

fn multipart_err(e: MultipartError) -> AppError {
    AppError::BadRequest(format!("invalid multipart form: {e}"))
}

fn parse_uuid(field: &str, value: &str) -> AppResult<Uuid> {
    Uuid::parse_str(value.trim())
        .map_err(|_| AppError::BadRequest(format!("'{field}' must be a valid UUID")))
}

/// Keep S3 keys tidy: allow alphanumerics plus `. - _`, replace the rest.
fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect()
}
