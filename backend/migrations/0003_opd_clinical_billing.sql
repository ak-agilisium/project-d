-- ============================================================================
-- OPD, clinical, billing & supporting modules
--   Patient identity (patients) stays lean; visits / clinical / billing /
--   insurance / documents live in their own tables, linked by patient_id or
--   visit_id.
-- ============================================================================

-- ---------------------------------------------------------------- departments
CREATE TABLE departments (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL UNIQUE,
    code        TEXT UNIQUE,
    description TEXT,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TRIGGER trg_departments_updated_at
    BEFORE UPDATE ON departments FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ------------------------------------------------------------ OPD visits
CREATE SEQUENCE IF NOT EXISTS opd_visit_number_seq START 1;

CREATE TABLE opd_visits (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    visit_number    TEXT NOT NULL UNIQUE
                    DEFAULT ('OPD' || lpad(nextval('opd_visit_number_seq')::text, 6, '0')),
    patient_id      UUID NOT NULL REFERENCES patients (id) ON DELETE CASCADE,
    department_id   UUID REFERENCES departments (id) ON DELETE SET NULL,
    doctor_id       UUID REFERENCES doctors (id) ON DELETE SET NULL,
    appointment_id  UUID REFERENCES appointments (id) ON DELETE SET NULL,
    visit_date      TIMESTAMPTZ NOT NULL DEFAULT now(),
    visit_type      TEXT NOT NULL DEFAULT 'new'
                    CHECK (visit_type IN ('new', 'follow_up', 'emergency', 'walk_in')),
    chief_complaint TEXT,
    visit_status    TEXT NOT NULL DEFAULT 'open'
                    CHECK (visit_status IN ('open', 'in_consultation', 'completed', 'cancelled')),
    referred_by     TEXT,
    created_by      TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_opd_visits_patient ON opd_visits (patient_id);
CREATE INDEX idx_opd_visits_doctor ON opd_visits (doctor_id);
CREATE INDEX idx_opd_visits_department ON opd_visits (department_id);
CREATE INDEX idx_opd_visits_date ON opd_visits (visit_date);
CREATE TRIGGER trg_opd_visits_updated_at
    BEFORE UPDATE ON opd_visits FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- -------------------------------------------------- clinical records (RBAC-gated)
-- One clinical record per visit (upserted as the consultation progresses).
CREATE TABLE clinical_records (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    visit_id             UUID NOT NULL UNIQUE REFERENCES opd_visits (id) ON DELETE CASCADE,
    patient_id           UUID NOT NULL REFERENCES patients (id) ON DELETE CASCADE,
    symptoms             TEXT,
    diagnosis_code       TEXT,                 -- e.g. ICD-10
    diagnosis_description TEXT,
    allergies            TEXT,
    current_medications  TEXT,
    -- vital signs (structured)
    systolic_bp          INTEGER,
    diastolic_bp         INTEGER,
    pulse_bpm            INTEGER,
    temperature_c        NUMERIC(4, 1),
    respiratory_rate     INTEGER,
    spo2                 INTEGER,
    height_cm            NUMERIC(5, 1),
    weight_kg            NUMERIC(5, 1),
    clinical_notes       TEXT,
    treatment_plan       TEXT,
    follow_up_date       DATE,
    recorded_by          TEXT,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_clinical_records_patient ON clinical_records (patient_id);
CREATE TRIGGER trg_clinical_records_updated_at
    BEFORE UPDATE ON clinical_records FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ------------------------------------------------------------- prescriptions
CREATE TABLE prescriptions (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    visit_id      UUID NOT NULL REFERENCES opd_visits (id) ON DELETE CASCADE,
    patient_id    UUID NOT NULL REFERENCES patients (id) ON DELETE CASCADE,
    drug_name     TEXT NOT NULL,
    dosage        TEXT,
    frequency     TEXT,
    duration      TEXT,
    route         TEXT,
    instructions  TEXT,
    prescribed_by TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_prescriptions_visit ON prescriptions (visit_id);

-- ---------------------------------------------------------------- lab orders
CREATE TABLE lab_orders (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    visit_id        UUID NOT NULL REFERENCES opd_visits (id) ON DELETE CASCADE,
    patient_id      UUID NOT NULL REFERENCES patients (id) ON DELETE CASCADE,
    test_code       TEXT,
    test_name       TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'ordered'
                    CHECK (status IN ('ordered', 'collected', 'in_progress', 'completed', 'cancelled')),
    result_text     TEXT,
    result_s3_key   TEXT,
    ordered_by      TEXT,
    ordered_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    resulted_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_lab_orders_visit ON lab_orders (visit_id);

-- --------------------------------------------------------- patient insurance
CREATE TABLE patient_insurance (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id             UUID NOT NULL REFERENCES patients (id) ON DELETE CASCADE,
    payer_name             TEXT NOT NULL,
    policy_number          TEXT,
    member_id              TEXT,
    coverage_start_date    DATE,
    coverage_end_date      DATE,
    preauthorization_number TEXT,
    coverage_status        TEXT NOT NULL DEFAULT 'active'
                           CHECK (coverage_status IN ('active', 'inactive', 'expired')),
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_patient_insurance_patient ON patient_insurance (patient_id);
CREATE TRIGGER trg_patient_insurance_updated_at
    BEFORE UPDATE ON patient_insurance FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- --------------------------------------------------------- patient documents
-- Consent forms & supporting docs (files in S3; metadata here).
CREATE TABLE patient_documents (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id      UUID NOT NULL REFERENCES patients (id) ON DELETE CASCADE,
    visit_id        UUID REFERENCES opd_visits (id) ON DELETE SET NULL,
    document_type   TEXT NOT NULL,
    title           TEXT NOT NULL,
    s3_bucket       TEXT NOT NULL,
    s3_key          TEXT NOT NULL,
    content_type    TEXT NOT NULL DEFAULT 'application/octet-stream',
    file_size_bytes BIGINT NOT NULL DEFAULT 0,
    uploaded_by     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_patient_documents_patient ON patient_documents (patient_id);

-- ------------------------------------------------------- payment transactions
-- Ledger of payment attempts, settlements and refunds (payments = cleared rows).
CREATE TABLE payment_transactions (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id            UUID NOT NULL REFERENCES invoices (id) ON DELETE CASCADE,
    payment_id            UUID REFERENCES payments (id) ON DELETE SET NULL,
    txn_type              TEXT NOT NULL CHECK (txn_type IN ('charge', 'settlement', 'refund')),
    amount                NUMERIC(12, 2) NOT NULL,
    method                TEXT,
    transaction_reference TEXT,
    status                TEXT NOT NULL DEFAULT 'success'
                          CHECK (status IN ('pending', 'success', 'failed')),
    gateway               TEXT,
    processed_by          TEXT,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_payment_transactions_invoice ON payment_transactions (invoice_id);

-- --------------------------------------------------------------- audit logs
CREATE TABLE audit_logs (
    id          BIGSERIAL PRIMARY KEY,
    actor_sub   TEXT,
    actor_roles TEXT,
    action      TEXT NOT NULL,
    entity_type TEXT,
    entity_id   TEXT,
    method      TEXT,
    path        TEXT,
    status_code INTEGER,
    ip_address  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_audit_logs_entity ON audit_logs (entity_type, entity_id);
CREATE INDEX idx_audit_logs_actor ON audit_logs (actor_sub);

-- ----------------------------------------------- enrich invoices & payments
ALTER TABLE invoices
    ADD COLUMN IF NOT EXISTS visit_id   UUID REFERENCES opd_visits (id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS created_by TEXT,
    ADD COLUMN IF NOT EXISTS balance_amount NUMERIC(12, 2)
        GENERATED ALWAYS AS (total - amount_paid) STORED,
    ADD COLUMN IF NOT EXISTS payment_status TEXT
        GENERATED ALWAYS AS (
            CASE WHEN amount_paid >= total AND total > 0 THEN 'paid'
                 WHEN amount_paid > 0 THEN 'partial'
                 ELSE 'unpaid' END
        ) STORED;
CREATE INDEX IF NOT EXISTS idx_invoices_visit ON invoices (visit_id);

ALTER TABLE payments
    ADD COLUMN IF NOT EXISTS received_by   TEXT,
    ADD COLUMN IF NOT EXISTS refund_amount NUMERIC(12, 2) NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS status        TEXT NOT NULL DEFAULT 'completed'
        CHECK (status IN ('pending', 'completed', 'failed', 'refunded'));
