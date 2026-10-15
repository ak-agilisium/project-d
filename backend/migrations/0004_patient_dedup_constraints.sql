-- ============================================================================
-- Patient duplicate prevention — enforced in the schema (data model) itself.
--
--   Replaces the application-side fuzzy duplicate scoring and the
--   /patients/check-duplicates pre-flight endpoint. A duplicate is now rejected
--   by a UNIQUE constraint at INSERT time (Postgres SQLSTATE 23505), which the
--   API maps to HTTP 409 Conflict (see src/error.rs).
--
--   Design notes:
--   * Indexes are PARTIAL (WHERE status <> 'merged') so a record intentionally
--     retained as a merged duplicate never blocks its surviving record.
--   * Name/email matching is case-insensitive.
--   * Rows with a NULL date_of_birth do NOT collide — Postgres treats NULL key
--     values as distinct, and we cannot assert two records with no DOB are the
--     same person.
--
--   ⚠ If the table already contains rows that violate these rules, the
--     CREATE UNIQUE INDEX statements below will fail and name the offending
--     values. Resolve (merge/delete) those duplicates first, then re-run.
-- ============================================================================

-- Primary identity key: same name + same date of birth = the same patient.
CREATE UNIQUE INDEX IF NOT EXISTS uq_patients_identity
    ON patients (lower(first_name), lower(last_name), date_of_birth)
    WHERE status <> 'merged';

-- One active patient record per email address.
CREATE UNIQUE INDEX IF NOT EXISTS uq_patients_email
    ON patients (lower(email))
    WHERE email IS NOT NULL AND status <> 'merged';

-- Replaced by the unique indexes above (same columns, now with uniqueness).
DROP INDEX IF EXISTS idx_patients_dupe_name_dob;
DROP INDEX IF EXISTS idx_patients_email;

-- Optional — OFF by default. Enforce one patient per phone number only if your
-- intake workflow guarantees a unique phone per patient. Leave disabled when
-- families share a contact number (common in OPD), which the plain
-- idx_patients_phone lookup index (from 0002) already supports.
-- CREATE UNIQUE INDEX IF NOT EXISTS uq_patients_phone
--     ON patients (phone)
--     WHERE phone IS NOT NULL AND status <> 'merged';
