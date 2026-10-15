-- ============================================================================
-- Patient master expansion
--   * Human-readable MRN (patient_number)
--   * Structured name & address, emergency contact, demographics
--   * Lifecycle status + merge pointer (for de-duplication)
--   * Audit columns (created_by / updated_by)
-- ============================================================================

-- Sequence backing the medical record number (MRN).
CREATE SEQUENCE IF NOT EXISTS patient_number_seq START 1000;

ALTER TABLE patients
    ADD COLUMN IF NOT EXISTS patient_number                 TEXT,
    ADD COLUMN IF NOT EXISTS middle_name                    TEXT,
    ADD COLUMN IF NOT EXISTS alternate_phone                TEXT,
    ADD COLUMN IF NOT EXISTS address_line1                  TEXT,
    ADD COLUMN IF NOT EXISTS address_line2                  TEXT,
    ADD COLUMN IF NOT EXISTS city                           TEXT,
    ADD COLUMN IF NOT EXISTS state                          TEXT,
    ADD COLUMN IF NOT EXISTS postal_code                    TEXT,
    ADD COLUMN IF NOT EXISTS country                        TEXT,
    ADD COLUMN IF NOT EXISTS blood_group                    TEXT,
    ADD COLUMN IF NOT EXISTS marital_status                 TEXT,
    ADD COLUMN IF NOT EXISTS preferred_language             TEXT,
    ADD COLUMN IF NOT EXISTS emergency_contact_name         TEXT,
    ADD COLUMN IF NOT EXISTS emergency_contact_phone        TEXT,
    ADD COLUMN IF NOT EXISTS emergency_contact_relationship TEXT,
    ADD COLUMN IF NOT EXISTS patient_photo_url              TEXT,
    ADD COLUMN IF NOT EXISTS status                         TEXT NOT NULL DEFAULT 'active',
    ADD COLUMN IF NOT EXISTS merged_into                    UUID REFERENCES patients (id),
    ADD COLUMN IF NOT EXISTS created_by                     TEXT,
    ADD COLUMN IF NOT EXISTS updated_by                     TEXT;

-- Migrate the old single-line address into address_line1, then drop it.
UPDATE patients SET address_line1 = address
    WHERE address IS NOT NULL AND address_line1 IS NULL;
ALTER TABLE patients DROP COLUMN IF EXISTS address;

-- Backfill MRNs for any existing rows, then make it defaulted, required & unique.
UPDATE patients
    SET patient_number = 'MRN' || lpad(nextval('patient_number_seq')::text, 6, '0')
    WHERE patient_number IS NULL;
ALTER TABLE patients
    ALTER COLUMN patient_number SET DEFAULT ('MRN' || lpad(nextval('patient_number_seq')::text, 6, '0'));
ALTER TABLE patients ALTER COLUMN patient_number SET NOT NULL;
ALTER TABLE patients ADD CONSTRAINT uq_patients_patient_number UNIQUE (patient_number);

-- Value constraints (mirrored by the API layer).
ALTER TABLE patients
    ADD CONSTRAINT chk_patients_status CHECK (status IN ('active', 'inactive', 'merged'));
ALTER TABLE patients
    ADD CONSTRAINT chk_patients_blood_group
    CHECK (blood_group IS NULL OR blood_group IN
        ('A+', 'A-', 'B+', 'B-', 'AB+', 'AB-', 'O+', 'O-', 'unknown'));

-- Indexes to support duplicate detection.
CREATE INDEX IF NOT EXISTS idx_patients_dupe_name_dob
    ON patients (lower(last_name), lower(first_name), date_of_birth);
CREATE INDEX IF NOT EXISTS idx_patients_phone ON patients (phone);
CREATE INDEX IF NOT EXISTS idx_patients_email ON patients (lower(email));
