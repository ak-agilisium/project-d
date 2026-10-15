use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::auth::rbac::Permission;
use crate::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::handlers::{ensure_one_of, validate};
use crate::models::billing::{
    CreateInvoice, CreatePayment, CreateRefund, Invoice, InvoiceDetail, PaymentTransaction,
    INVOICE_STATUSES, PAYMENT_METHODS,
};
use crate::models::{paginate, Paginated};
use crate::repositories::billing as repo;
use crate::state::AppState;

pub async fn create_invoice(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateInvoice>,
) -> AppResult<impl IntoResponse> {
    user.require(Permission::BillingWrite)?;
    validate(&body)?;
    if body.items.is_empty() {
        return Err(AppError::BadRequest(
            "an invoice needs at least one line item".into(),
        ));
    }
    let detail = repo::create_invoice(&state.db, &body, Some(user.sub.as_str())).await?;
    tracing::info!(
        invoice_id = %detail.invoice.id,
        invoice_number = %detail.invoice.invoice_number,
        "created invoice"
    );
    Ok((StatusCode::CREATED, Json(detail)))
}

#[derive(Debug, Deserialize)]
pub struct InvoiceListQuery {
    pub patient_id: Option<Uuid>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_invoices(
    user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<InvoiceListQuery>,
) -> AppResult<Json<Paginated<Invoice>>> {
    user.require(Permission::BillingRead)?;
    if let Some(status) = &q.status {
        ensure_one_of("status", status, INVOICE_STATUSES)?;
    }
    let (limit, offset) = paginate(q.limit, q.offset);
    let data = repo::list(&state.db, q.patient_id, q.status.as_deref(), limit, offset).await?;
    Ok(Json(Paginated {
        data,
        limit,
        offset,
    }))
}

pub async fn get_invoice(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<InvoiceDetail>> {
    user.require(Permission::BillingRead)?;
    Ok(Json(repo::get_detail(&state.db, id).await?))
}

pub async fn add_payment(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<CreatePayment>,
) -> AppResult<Json<Value>> {
    user.require(Permission::BillingWrite)?;
    validate(&body)?;
    ensure_one_of("method", &body.method, PAYMENT_METHODS)?;
    if body.amount <= Decimal::ZERO {
        return Err(AppError::BadRequest(
            "payment amount must be positive".into(),
        ));
    }
    let (invoice, payment) =
        repo::add_payment(&state.db, id, &body, Some(user.sub.as_str())).await?;
    tracing::info!(invoice_id = %invoice.id, payment_id = %payment.id, "recorded payment");
    Ok(Json(json!({ "invoice": invoice, "payment": payment })))
}

/// GET /api/v1/invoices/{id}/transactions — the invoice's payment ledger.
pub async fn transactions(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<PaymentTransaction>>> {
    user.require(Permission::BillingRead)?;
    Ok(Json(repo::list_transactions(&state.db, id).await?))
}

/// POST /api/v1/invoices/{id}/refund — record a refund against the invoice.
pub async fn refund(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateRefund>,
) -> AppResult<Json<Value>> {
    user.require(Permission::BillingWrite)?;
    validate(&body)?;
    let (invoice, transaction) =
        repo::refund(&state.db, id, &body, Some(user.sub.as_str())).await?;
    tracing::info!(invoice_id = %invoice.id, amount = %body.amount, "recorded refund");
    Ok(Json(
        json!({ "invoice": invoice, "transaction": transaction }),
    ))
}
