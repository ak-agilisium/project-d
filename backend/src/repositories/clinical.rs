use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::clinical::{ClinicalRecord, UpsertClinicalRecord};

pub async fn get_by_visit(pool: &PgPool, visit_id: Uuid) -> AppResult<ClinicalRecord> {
    sqlx::query_as::<_, ClinicalRecord>("SELECT * FROM clinical_records WHERE visit_id = $1")
        .bind(visit_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

/// Insert or update the clinical record for a visit. On update, `NULL` fields in
/// the payload keep their existing value (COALESCE), supporting progressive
/// documentation across a consultation.
pub async fn upsert(
    pool: &PgPool,
    visit_id: Uuid,
    patient_id: Uuid,
    input: &UpsertClinicalRecord,
    actor: Option<&str>,
) -> AppResult<ClinicalRecord> {
    let record = sqlx::query_as::<_, ClinicalRecord>(
        r#"
        INSERT INTO clinical_records (
            visit_id, patient_id, symptoms, diagnosis_code, diagnosis_description,
            allergies, current_medications, systolic_bp, diastolic_bp, pulse_bpm,
            temperature_c, respiratory_rate, spo2, height_cm, weight_kg,
            clinical_notes, treatment_plan, follow_up_date, recorded_by
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)
        ON CONFLICT (visit_id) DO UPDATE SET
            symptoms              = COALESCE(EXCLUDED.symptoms, clinical_records.symptoms),
            diagnosis_code        = COALESCE(EXCLUDED.diagnosis_code, clinical_records.diagnosis_code),
            diagnosis_description = COALESCE(EXCLUDED.diagnosis_description, clinical_records.diagnosis_description),
            allergies             = COALESCE(EXCLUDED.allergies, clinical_records.allergies),
            current_medications   = COALESCE(EXCLUDED.current_medications, clinical_records.current_medications),
            systolic_bp           = COALESCE(EXCLUDED.systolic_bp, clinical_records.systolic_bp),
            diastolic_bp          = COALESCE(EXCLUDED.diastolic_bp, clinical_records.diastolic_bp),
            pulse_bpm             = COALESCE(EXCLUDED.pulse_bpm, clinical_records.pulse_bpm),
            temperature_c         = COALESCE(EXCLUDED.temperature_c, clinical_records.temperature_c),
            respiratory_rate      = COALESCE(EXCLUDED.respiratory_rate, clinical_records.respiratory_rate),
            spo2                  = COALESCE(EXCLUDED.spo2, clinical_records.spo2),
            height_cm             = COALESCE(EXCLUDED.height_cm, clinical_records.height_cm),
            weight_kg             = COALESCE(EXCLUDED.weight_kg, clinical_records.weight_kg),
            clinical_notes        = COALESCE(EXCLUDED.clinical_notes, clinical_records.clinical_notes),
            treatment_plan        = COALESCE(EXCLUDED.treatment_plan, clinical_records.treatment_plan),
            follow_up_date        = COALESCE(EXCLUDED.follow_up_date, clinical_records.follow_up_date),
            recorded_by           = EXCLUDED.recorded_by
        RETURNING *
        "#,
    )
    .bind(visit_id)
    .bind(patient_id)
    .bind(input.symptoms.as_deref())
    .bind(input.diagnosis_code.as_deref())
    .bind(input.diagnosis_description.as_deref())
    .bind(input.allergies.as_deref())
    .bind(input.current_medications.as_deref())
    .bind(input.systolic_bp)
    .bind(input.diastolic_bp)
    .bind(input.pulse_bpm)
    .bind(input.temperature_c)
    .bind(input.respiratory_rate)
    .bind(input.spo2)
    .bind(input.height_cm)
    .bind(input.weight_kg)
    .bind(input.clinical_notes.as_deref())
    .bind(input.treatment_plan.as_deref())
    .bind(input.follow_up_date)
    .bind(actor)
    .fetch_one(pool)
    .await?;
    Ok(record)
}
