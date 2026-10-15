use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Allowed invoice statuses (mirrors the DB CHECK constraint).
pub const INVOICE_STATUSES: &[&str] = &["draft", "issued", "partially_paid", "paid", "cancelled"];

/// Allowed payment methods (mirrors the DB CHECK constraint).
pub const PAYMENT_METHODS: &[&str] = &["cash", "card", "upi", "netbanking", "insurance", "other"];

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub invoice_number: String,
    pub patient_id: Uuid,
    pub appointment_id: Option<Uuid>,
    pub visit_id: Option<Uuid>,
    pub status: String,
    pub currency: String,
    pub subtotal: Decimal,
    pub tax: Decimal,
    pub discount: Decimal,
    pub total: Decimal,
    pub amount_paid: Decimal,
    /// Generated column: `total - amount_paid`.
    pub balance_amount: Decimal,
    /// Generated column: `paid` | `partial` | `unpaid`.
    pub payment_status: String,
    pub notes: Option<String>,
    pub issued_at: Option<DateTime<Utc>>,
    pub due_date: Option<NaiveDate>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct InvoiceItem {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub amount: Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub amount: Decimal,
    pub method: String,
    pub reference: Option<String>,
    pub status: String,
    pub refund_amount: Decimal,
    pub received_by: Option<String>,
    pub paid_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// An invoice together with its line items (and, when loaded, payments).
#[derive(Debug, Serialize)]
pub struct InvoiceDetail {
    #[serde(flatten)]
    pub invoice: Invoice,
    pub items: Vec<InvoiceItem>,
    pub payments: Vec<Payment>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateInvoice {
    pub patient_id: Uuid,
    pub appointment_id: Option<Uuid>,
    pub visit_id: Option<Uuid>,
    /// Validated per-item; non-empty is enforced in the handler.
    #[validate(nested)]
    pub items: Vec<CreateInvoiceItem>,
    pub tax: Option<Decimal>,
    pub discount: Option<Decimal>,
    #[validate(length(min = 3, max = 3))]
    pub currency: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub notes: Option<String>,
    /// Set `true` to immediately mark the invoice as issued (default: draft).
    #[serde(default)]
    pub issue_now: bool,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateInvoiceItem {
    #[validate(length(min = 1, max = 255))]
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePayment {
    pub amount: Decimal,
    /// One of [`PAYMENT_METHODS`].
    pub method: String,
    #[validate(length(max = 128))]
    pub reference: Option<String>,
}

/// Ledger entry: a charge, settlement or refund against an invoice.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PaymentTransaction {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub payment_id: Option<Uuid>,
    pub txn_type: String,
    pub amount: Decimal,
    pub method: Option<String>,
    pub transaction_reference: Option<String>,
    pub status: String,
    pub gateway: Option<String>,
    pub processed_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateRefund {
    pub amount: Decimal,
    #[validate(length(max = 32))]
    pub method: Option<String>,
    #[validate(length(max = 128))]
    pub reference: Option<String>,
}
