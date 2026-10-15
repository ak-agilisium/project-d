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
use crate::models::patient_document::{NewPatientDocument, PatientDocument};
use crate::models::report::ReportDownload;
use crate::models::{paginate, Paginated};
use crate::repositories::patient_document as repo;
use crate::state::AppState;

/// POST /api/v1/documents — upload a consent form / supporting document to S3.
///
/// multipart/form-data: `patient_id` (required), `file` (required),
/// `visit_id`, `document_type`, `title` (optional).
pub async fn upload(
    user: AuthUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<impl IntoResponse> {
    user.require(Permission::DocumentWrite)?;

    let mut patient_id: Option<Uuid> = None;
    let mut visit_id: Option<Uuid> = None;
    let mut document_type: Option<String> = None;
    let mut title: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut content_type = "application/octet-stream".to_string();

    while let Some(field) = multipart.next_field().await.map_err(mp_err)? {
        match field.name().unwrap_or_default() {
            "patient_id" => {
                patient_id = Some(parse_uuid(
                    "patient_id",
                    &field.text().await.map_err(mp_err)?,
                )?)
            }
            "visit_id" => {
                let v = field.text().await.map_err(mp_err)?;
                if !v.trim().is_empty() {
                    visit_id = Some(parse_uuid("visit_id", &v)?);
                }
            }
            "document_type" => document_type = Some(field.text().await.map_err(mp_err)?),
            "title" => title = Some(field.text().await.map_err(mp_err)?),
            "file" => {
                file_name = field.file_name().map(str::to_string);
                if let Some(ct) = field.content_type() {
                    content_type = ct.to_string();
                }
                file_bytes = Some(field.bytes().await.map_err(mp_err)?.to_vec());
            }
            _ => {
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
            "the uploaded file is too large".into(),
        ));
    }

    let document_type = document_type.unwrap_or_else(|| "general".to_string());
    let title = title
        .or_else(|| file_name.clone())
        .unwrap_or_else(|| "document".to_string());
    let file_id = Uuid::new_v4();
    let safe = file_name
        .as_deref()
        .map(sanitize)
        .unwrap_or_else(|| "file".into());
    let s3_key = format!("documents/{patient_id}/{file_id}-{safe}");
    let size = bytes.len() as i64;

    state.s3.put_object(&s3_key, bytes, &content_type).await?;

    let new = NewPatientDocument {
        patient_id,
        visit_id,
        document_type,
        title,
        s3_bucket: state.s3.bucket().to_string(),
        s3_key,
        content_type,
        file_size_bytes: size,
        uploaded_by: Some(user.sub.clone()),
    };
    let doc = repo::create(&state.db, &new).await?;
    tracing::info!(document_id = %doc.id, patient_id = %patient_id, "uploaded patient document");
    Ok((StatusCode::CREATED, Json(doc)))
}

/// GET /api/v1/patients/{patient_id}/documents
pub async fn list_by_patient(
    user: AuthUser,
    State(state): State<AppState>,
    Path(patient_id): Path<Uuid>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Paginated<PatientDocument>>> {
    user.require(Permission::DocumentRead)?;
    let (limit, offset) = paginate(q.limit, q.offset);
    let data = repo::list_by_patient(&state.db, patient_id, limit, offset).await?;
    Ok(Json(Paginated {
        data,
        limit,
        offset,
    }))
}

/// GET /api/v1/documents/{id}/download — presigned S3 URL.
pub async fn download(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ReportDownload>> {
    user.require(Permission::DocumentRead)?;
    let doc = repo::get(&state.db, id).await?;
    let presigned = state.s3.presign_get(&doc.s3_key).await?;
    Ok(Json(ReportDownload {
        url: presigned.url,
        expires_in_secs: presigned.expires_in_secs,
    }))
}

/// DELETE /api/v1/documents/{id}
pub async fn delete(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    user.require(Permission::DocumentWrite)?;
    let doc = repo::delete(&state.db, id).await?;
    if let Err(e) = state.s3.delete_object(&doc.s3_key).await {
        tracing::warn!(error = %e, key = %doc.s3_key, "failed to delete S3 object for document");
    }
    Ok(StatusCode::NO_CONTENT)
}

fn mp_err(e: MultipartError) -> AppError {
    AppError::BadRequest(format!("invalid multipart form: {e}"))
}

fn parse_uuid(field: &str, value: &str) -> AppResult<Uuid> {
    Uuid::parse_str(value.trim())
        .map_err(|_| AppError::BadRequest(format!("'{field}' must be a valid UUID")))
}

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
