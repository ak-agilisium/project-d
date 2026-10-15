-- ============================================================================
-- Medical billing & patient OPD booking — initial schema
-- Target: AWS RDS for PostgreSQL (>= 13)
-- ============================================================================

CREATE EXTENSION IF NOT EXISTS "pgcrypto"; -- gen_random_uuid()

-- Reusable trigger to keep updated_at fresh -----------------------------------
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Patients
--   cognito_sub links the patient record to its AWS Cognito identity (the
--   `sub` claim). It is nullable so staff can also pre-register walk-in
--   patients who do not yet have a Cognito login.
-- ============================================================================
CREATE TABLE patients (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cognito_sub   TEXT UNIQUE,
    first_name    TEXT NOT NULL,
    last_name     TEXT NOT NULL,
    date_of_birth DATE,
    gender        TEXT CHECK (gender IN ('male', 'female', 'other', 'unknown')),
    phone         TEXT,
    email         TEXT,
    address       TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_patients_cognito_sub ON patients (cognito_sub);
CREATE INDEX idx_patients_name ON patients (last_name, first_name);

CREATE TRIGGER trg_patients_updated_at
    BEFORE UPDATE ON patients
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ============================================================================
-- Doctors
-- ============================================================================
CREATE TABLE doctors (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name       TEXT NOT NULL,
    last_name        TEXT NOT NULL,
    specialization   TEXT NOT NULL,
    department       TEXT,
    consultation_fee NUMERIC(12, 2) NOT NULL DEFAULT 0,
    email            TEXT,
    phone            TEXT,
    is_active        BOOLEAN NOT NULL DEFAULT TRUE,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_doctors_specialization ON doctors (specialization);

CREATE TRIGGER trg_doctors_updated_at
    BEFORE UPDATE ON doctors
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ============================================================================
-- Appointments (OPD booking)
-- ============================================================================
CREATE TABLE appointments (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id   UUID NOT NULL REFERENCES patients (id) ON DELETE CASCADE,
    doctor_id    UUID NOT NULL REFERENCES doctors (id) ON DELETE RESTRICT,
    scheduled_at TIMESTAMPTZ NOT NULL,
    status       TEXT NOT NULL DEFAULT 'scheduled'
                 CHECK (status IN ('scheduled', 'checked_in', 'completed', 'cancelled', 'no_show')),
    reason       TEXT,
    notes        TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_appointments_patient ON appointments (patient_id);
CREATE INDEX idx_appointments_doctor ON appointments (doctor_id);
CREATE INDEX idx_appointments_scheduled_at ON appointments (scheduled_at);

CREATE TRIGGER trg_appointments_updated_at
    BEFORE UPDATE ON appointments
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ============================================================================
-- Invoices (billing header)
-- ============================================================================
CREATE TABLE invoices (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_number TEXT NOT NULL UNIQUE,
    patient_id     UUID NOT NULL REFERENCES patients (id) ON DELETE RESTRICT,
    appointment_id UUID REFERENCES appointments (id) ON DELETE SET NULL,
    status         TEXT NOT NULL DEFAULT 'draft'
                   CHECK (status IN ('draft', 'issued', 'partially_paid', 'paid', 'cancelled')),
    currency       TEXT NOT NULL DEFAULT 'INR',
    subtotal       NUMERIC(12, 2) NOT NULL DEFAULT 0,
    tax            NUMERIC(12, 2) NOT NULL DEFAULT 0,
    discount       NUMERIC(12, 2) NOT NULL DEFAULT 0,
    total          NUMERIC(12, 2) NOT NULL DEFAULT 0,
    amount_paid    NUMERIC(12, 2) NOT NULL DEFAULT 0,
    notes          TEXT,
    issued_at      TIMESTAMPTZ,
    due_date       DATE,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_invoices_patient ON invoices (patient_id);
CREATE INDEX idx_invoices_status ON invoices (status);

CREATE TRIGGER trg_invoices_updated_at
    BEFORE UPDATE ON invoices
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ============================================================================
-- Invoice line items
-- ============================================================================
CREATE TABLE invoice_items (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id  UUID NOT NULL REFERENCES invoices (id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    quantity    NUMERIC(12, 2) NOT NULL DEFAULT 1,
    unit_price  NUMERIC(12, 2) NOT NULL DEFAULT 0,
    amount      NUMERIC(12, 2) NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_invoice_items_invoice ON invoice_items (invoice_id);

-- ============================================================================
-- Payments
-- ============================================================================
CREATE TABLE payments (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES invoices (id) ON DELETE CASCADE,
    amount     NUMERIC(12, 2) NOT NULL,
    method     TEXT NOT NULL
               CHECK (method IN ('cash', 'card', 'upi', 'netbanking', 'insurance', 'other')),
    reference  TEXT,
    paid_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_payments_invoice ON payments (invoice_id);

-- ============================================================================
-- Reports (files live in S3 ONLY; this table holds metadata/pointers)
-- ============================================================================
CREATE TABLE reports (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id      UUID NOT NULL REFERENCES patients (id) ON DELETE CASCADE,
    appointment_id  UUID REFERENCES appointments (id) ON DELETE SET NULL,
    report_type     TEXT NOT NULL,         -- e.g. lab, radiology, prescription, invoice_pdf
    title           TEXT NOT NULL,
    s3_bucket       TEXT NOT NULL,
    s3_key          TEXT NOT NULL,
    content_type    TEXT NOT NULL DEFAULT 'application/octet-stream',
    file_size_bytes BIGINT NOT NULL DEFAULT 0,
    uploaded_by     TEXT,                  -- Cognito sub of uploader
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_reports_patient ON reports (patient_id);
CREATE UNIQUE INDEX uq_reports_s3_object ON reports (s3_bucket, s3_key);
