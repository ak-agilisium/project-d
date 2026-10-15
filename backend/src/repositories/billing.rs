use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::billing::{
    CreateInvoice, CreatePayment, CreateRefund, Invoice, InvoiceDetail, InvoiceItem, Payment,
    PaymentTransaction,
};

/// Create an invoice and its line items atomically, computing totals server-side.
pub async fn create_invoice(
    pool: &PgPool,
    input: &CreateInvoice,
    actor: Option<&str>,
) -> AppResult<InvoiceDetail> {
    // Compute line amounts and the subtotal (never trust client-sent totals).
    let mut subtotal = Decimal::ZERO;
    let mut lines: Vec<(String, Decimal, Decimal, Decimal)> = Vec::with_capacity(input.items.len());
    for item in &input.items {
        let amount = item.quantity * item.unit_price;
        subtotal += amount;
        lines.push((
            item.description.clone(),
            item.quantity,
            item.unit_price,
            amount,
        ));
    }

    let tax = input.tax.unwrap_or(Decimal::ZERO);
    let discount = input.discount.unwrap_or(Decimal::ZERO);
    let total = subtotal + tax - discount;
    let currency = input.currency.clone().unwrap_or_else(|| "INR".to_string());
    let (status, issued_at) = if input.issue_now {
        ("issued", Some(Utc::now()))
    } else {
        ("draft", None)
    };
    let invoice_number = generate_invoice_number();

    let mut tx = pool.begin().await?;

    let invoice = sqlx::query_as::<_, Invoice>(
        r#"
        INSERT INTO invoices
            (invoice_number, patient_id, appointment_id, visit_id, status, currency,
             subtotal, tax, discount, total, amount_paid, notes, issued_at, due_date, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 0, $11, $12, $13, $14)
        RETURNING *
        "#,
    )
    .bind(&invoice_number)
    .bind(input.patient_id)
    .bind(input.appointment_id)
    .bind(input.visit_id)
    .bind(status)
    .bind(&currency)
    .bind(subtotal)
    .bind(tax)
    .bind(discount)
    .bind(total)
    .bind(input.notes.as_deref())
    .bind(issued_at)
    .bind(input.due_date)
    .bind(actor)
    .fetch_one(&mut *tx)
    .await?;

    let mut items = Vec::with_capacity(lines.len());
    for (description, quantity, unit_price, amount) in lines {
        let item = sqlx::query_as::<_, InvoiceItem>(
            r#"
            INSERT INTO invoice_items (invoice_id, description, quantity, unit_price, amount)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(invoice.id)
        .bind(&description)
        .bind(quantity)
        .bind(unit_price)
        .bind(amount)
        .fetch_one(&mut *tx)
        .await?;
        items.push(item);
    }

    tx.commit().await?;

    Ok(InvoiceDetail {
        invoice,
        items,
        payments: Vec::new(),
    })
}

pub async fn get_detail(pool: &PgPool, id: Uuid) -> AppResult<InvoiceDetail> {
    let invoice = sqlx::query_as::<_, Invoice>("SELECT * FROM invoices WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let items = sqlx::query_as::<_, InvoiceItem>(
        "SELECT * FROM invoice_items WHERE invoice_id = $1 ORDER BY created_at",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let payments = sqlx::query_as::<_, Payment>(
        "SELECT * FROM payments WHERE invoice_id = $1 ORDER BY paid_at",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    Ok(InvoiceDetail {
        invoice,
        items,
        payments,
    })
}

pub async fn list(
    pool: &PgPool,
    patient_id: Option<Uuid>,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Invoice>> {
    let rows = sqlx::query_as::<_, Invoice>(
        r#"
        SELECT * FROM invoices
        WHERE ($1::uuid IS NULL OR patient_id = $1)
          AND ($2::text IS NULL OR status = $2)
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(patient_id)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Record a payment and recompute the invoice's paid amount and status.
pub async fn add_payment(
    pool: &PgPool,
    invoice_id: Uuid,
    input: &CreatePayment,
    actor: Option<&str>,
) -> AppResult<(Invoice, Payment)> {
    let mut tx = pool.begin().await?;

    // Lock the invoice row so concurrent payments can't race the recompute.
    let invoice = sqlx::query_as::<_, Invoice>("SELECT * FROM invoices WHERE id = $1 FOR UPDATE")
        .bind(invoice_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound)?;

    let payment = sqlx::query_as::<_, Payment>(
        r#"
        INSERT INTO payments (invoice_id, amount, method, reference, received_by)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(invoice_id)
    .bind(input.amount)
    .bind(&input.method)
    .bind(input.reference.as_deref())
    .bind(actor)
    .fetch_one(&mut *tx)
    .await?;

    // Ledger entry for this settlement.
    sqlx::query(
        r#"
        INSERT INTO payment_transactions
            (invoice_id, payment_id, txn_type, amount, method, transaction_reference, processed_by)
        VALUES ($1, $2, 'settlement', $3, $4, $5, $6)
        "#,
    )
    .bind(invoice_id)
    .bind(payment.id)
    .bind(input.amount)
    .bind(&input.method)
    .bind(input.reference.as_deref())
    .bind(actor)
    .execute(&mut *tx)
    .await?;

    let amount_paid: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount), 0)::numeric FROM payments WHERE invoice_id = $1",
    )
    .bind(invoice_id)
    .fetch_one(&mut *tx)
    .await?;

    let status = if amount_paid >= invoice.total {
        "paid"
    } else if amount_paid > Decimal::ZERO {
        "partially_paid"
    } else {
        invoice.status.as_str()
    };

    let updated = sqlx::query_as::<_, Invoice>(
        "UPDATE invoices SET amount_paid = $2, status = $3 WHERE id = $1 RETURNING *",
    )
    .bind(invoice_id)
    .bind(amount_paid)
    .bind(status)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((updated, payment))
}

pub async fn list_transactions(
    pool: &PgPool,
    invoice_id: Uuid,
) -> AppResult<Vec<PaymentTransaction>> {
    let rows = sqlx::query_as::<_, PaymentTransaction>(
        "SELECT * FROM payment_transactions WHERE invoice_id = $1 ORDER BY created_at",
    )
    .bind(invoice_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Record a refund: ledger entry + reduce the invoice's paid amount.
pub async fn refund(
    pool: &PgPool,
    invoice_id: Uuid,
    input: &CreateRefund,
    actor: Option<&str>,
) -> AppResult<(Invoice, PaymentTransaction)> {
    let mut tx = pool.begin().await?;

    let invoice = sqlx::query_as::<_, Invoice>("SELECT * FROM invoices WHERE id = $1 FOR UPDATE")
        .bind(invoice_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound)?;

    if input.amount <= Decimal::ZERO {
        return Err(AppError::BadRequest(
            "refund amount must be positive".into(),
        ));
    }
    if input.amount > invoice.amount_paid {
        return Err(AppError::BadRequest(
            "refund amount exceeds the amount paid".into(),
        ));
    }

    let txn = sqlx::query_as::<_, PaymentTransaction>(
        r#"
        INSERT INTO payment_transactions
            (invoice_id, txn_type, amount, method, transaction_reference, processed_by)
        VALUES ($1, 'refund', $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(invoice_id)
    .bind(input.amount)
    .bind(input.method.as_deref())
    .bind(input.reference.as_deref())
    .bind(actor)
    .fetch_one(&mut *tx)
    .await?;

    let updated = sqlx::query_as::<_, Invoice>(
        "UPDATE invoices SET amount_paid = amount_paid - $2 WHERE id = $1 RETURNING *",
    )
    .bind(invoice_id)
    .bind(input.amount)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((updated, txn))
}

/// Generate a human-readable, collision-resistant invoice number.
fn generate_invoice_number() -> String {
    let date = Utc::now().format("%Y%m%d");
    let id = Uuid::new_v4().simple().to_string();
    format!("INV-{date}-{}", id[..8].to_uppercase())
}
