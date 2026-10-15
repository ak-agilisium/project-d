use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::patient_insurance::{
    CreatePatientInsurance, PatientInsurance, UpdatePatientInsurance,
};

pub async fn create(
    pool: &PgPool,
    patient_id: Uuid,
    input: &CreatePatientInsurance,
) -> AppResult<PatientInsurance> {
    let ins = sqlx::query_as::<_, PatientInsurance>(
        r#"
        INSERT INTO patient_insurance
            (patient_id, payer_name, policy_number, member_id,
             coverage_start_date, coverage_end_date, preauthorization_number)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(patient_id)
    .bind(&input.payer_name)
    .bind(input.policy_number.as_deref())
    .bind(input.member_id.as_deref())
    .bind(input.coverage_start_date)
    .bind(input.coverage_end_date)
    .bind(input.preauthorization_number.as_deref())
    .fetch_one(pool)
    .await?;
    Ok(ins)
}

pub async fn list_by_patient(pool: &PgPool, patient_id: Uuid) -> AppResult<Vec<PatientInsurance>> {
    let rows = sqlx::query_as::<_, PatientInsurance>(
        "SELECT * FROM patient_insurance WHERE patient_id = $1 ORDER BY created_at DESC",
    )
    .bind(patient_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    input: &UpdatePatientInsurance,
) -> AppResult<PatientInsurance> {
    sqlx::query_as::<_, PatientInsurance>(
        r#"
        UPDATE patient_insurance SET
            payer_name              = COALESCE($2, payer_name),
            policy_number           = COALESCE($3, policy_number),
            member_id               = COALESCE($4, member_id),
            coverage_start_date     = COALESCE($5, coverage_start_date),
            coverage_end_date       = COALESCE($6, coverage_end_date),
            preauthorization_number = COALESCE($7, preauthorization_number),
            coverage_status         = COALESCE($8, coverage_status)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(input.payer_name.as_deref())
    .bind(input.policy_number.as_deref())
    .bind(input.member_id.as_deref())
    .bind(input.coverage_start_date)
    .bind(input.coverage_end_date)
    .bind(input.preauthorization_number.as_deref())
    .bind(input.coverage_status.as_deref())
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}
