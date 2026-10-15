# Architecture & Flow Review — OPD Billing Backend

> Reviewed: 2026-10-15 · Scope: full `src/`, `migrations/`, config & deploy files.
> Rust (Axum/Tokio) · AWS Cognito (auth) · RDS PostgreSQL via SQLx · S3 (files).
> This is a reference document for future work — findings are ranked by severity
> with `file:line` anchors and concrete fixes.

---

## 1. End-to-end request flow

```
                         ┌─────────────────────────────────────────────┐
HTTP request  ─────────▶ │ routes.rs — layered middleware (outer→inner) │
                         │   TraceLayer → TimeoutLayer(30s)             │
                         │   → DefaultBodyLimit(max_upload+1MiB)        │
                         │   → CorsLayer → audit middleware             │
                         └─────────────────────────────────────────────┘
                                          │
                                          ▼
          ┌──────────────────────────────────────────────────────┐
          │ AuthUser extractor (auth/mod.rs)                       │
          │   Bearer token → decode header → JwksCache.decoding_key│
          │   → verify RS256 sig + issuer + exp + token_use +      │
          │     client_id/aud → AuthUser{sub, groups, …}          │
          └──────────────────────────────────────────────────────┘
                                          │
                                          ▼
          ┌──────────────────────────────────────────────────────┐
          │ handler (handlers/*.rs)                                │
          │   user.require(Permission::…)  ← RBAC                  │
          │   validate(&body)              ← validator crate       │
          │   ensure_one_of(enum fields)                           │
          │   delegate ↓                                           │
          └──────────────────────────────────────────────────────┘
                     │                                   │
                     ▼                                   ▼
        repositories/*.rs (ALL SQL)          services/{cognito,s3}.rs (AWS)
                     │                                   │
                     ▼                                   ▼
              RDS PostgreSQL                      Cognito / S3
                     │
                     ▼
          Result<T, AppError> ──▶ error.rs IntoResponse ──▶ single JSON envelope
                                   { "error": { "code", "message" } }
```

**Layering is clean and consistently applied.** Handlers authorize → validate →
delegate; **all SQL lives in `repositories/`**; AWS calls live in `services/`;
every layer returns `AppError`, which renders one JSON shape and redacts
server-fault details while logging them in full (`error.rs:75-83, 97-116`).

### Middleware ordering note
In `routes.rs:29-47` the `audit` layer is attached **after** `.with_state(state)`
as the outermost layer, so it observes the final response status — correct. The
30s timeout, body limit and CORS wrap the versioned router.

---

## 2. Subsystem flows

### Auth (public → Cognito)
`handlers/auth.rs` → `services/cognito.rs`:
- `register` → `sign_up` (email = username; `SECRET_HASH` computed when the app
  client has a secret, `cognito.rs:55-67`), `confirm`, `resend-code`, `login`
  (`USER_PASSWORD_AUTH`), `refresh` (`REFRESH_TOKEN_AUTH`; **caller must pass the
  `sub` as username** so the secret hash matches — `cognito.rs:217`), `logout`
  (global sign-out). Login maps unknown-user and bad-password to the **same**
  error to prevent user enumeration (`cognito.rs:185-189`) — good.

### Token verification (every authed request)
`auth/jwks.rs` caches JWKS keyed by `kid`, lazily refreshing on a miss to handle
rotation. Access tokens have no `aud`, so audience validation is disabled and
`client_id`/`aud` is checked manually (`auth/mod.rs:140-158`). Both `access` and
`id` tokens are accepted.

### RBAC
`auth/rbac.rs` maps Cognito `cognito:groups` → `Role` → `Permission`. Effective
permissions are the **union** across a user's roles. Five roles: `super_admin`,
`admin`, `manager`, `doctor`, `front_office`. Clinical records
(`ClinicalRead/Write`) are withheld from everyone except `super_admin` and
`doctor` — including `admin` (`rbac.rs:110`).

### Patient → OPD visit → clinical → prescription / lab
Patient master (MRN auto-assigned via `patient_number_seq`; duplicate prevention
enforced by schema-level UNIQUE indexes — migration `0004`),
then `opd_visits` (auto `visit_number`), then per-visit `clinical_records`
(1:1, upserted), `prescriptions`, `lab_orders`. Clinical reads are **explicitly
audited** inside the handler (`clinical.rs:22-29`), as the design intends for
sensitive reads.

### Billing (money path)
All money is `NUMERIC(12,2)` ↔ `rust_decimal::Decimal` (no floats).
- **Invoice create** (`repositories/billing.rs:13-97`): totals computed
  **server-side** (client-sent totals ignored), header + line items inserted in
  **one transaction**. 
- **Payment** (`billing.rs:153-226`): `SELECT … FOR UPDATE` row lock → insert
  payment + ledger row → recompute `amount_paid` → set status — all in a tx.
- **Refund** (`billing.rs:242-293`): row lock, validates amount > 0 and
  ≤ `amount_paid`, writes a `refund` ledger row, decrements `amount_paid`.

### Files (S3-only, DB holds the pointer)
`reports` and `patient_documents` store bytes in S3; Postgres keeps metadata and
a `UNIQUE (s3_bucket, s3_key)` index (`0001_init.sql:163`). Downloads return a
short-lived **presigned GET URL** (default 900s) so bytes never transit the API.

---

## 3. Findings

Severity reflects impact in **this** system (an internal, staff-facing tool — all
authenticated principals are staff, not patients).

### 🔴 High

**H1 — Orphaned S3 objects when the DB write fails (upload path).**
`handlers/report.rs:97-110` and `handlers/patient_document.rs:91-104` upload to
S3 **first**, then insert the metadata row. If the insert fails (constraint, FK,
pool exhaustion), the object is already in S3 with no pointer and no cleanup.
*Fix:* on `repo::create(...)` error, issue a compensating `delete_object`, or
reverse the order (reserve the row, then upload, then mark ready). A background
reconciler over the `UNIQUE(s3_bucket,s3_key)` index can also sweep orphans.

**H2 — Invoice `status` goes stale after a refund; two competing status fields.**
`repositories/billing.rs:283-289` decrements `amount_paid` on refund but **does
not recompute `status`** — a fully refunded invoice keeps `status = 'paid'`.
Separately, migration `0003_opd_clinical_billing.sql:187-197` adds **generated**
columns `balance_amount` and `payment_status` (`paid|partial|unpaid`) that live
*alongside* the hand-maintained `status` (`draft|issued|partially_paid|paid|
cancelled`). The two can disagree. *Fix:* recompute `status` inside `refund`
(as `add_payment` does), and pick **one** source of truth for payment state —
prefer the generated `payment_status`/`balance_amount` and drop the manual
recompute, or drop the generated columns. Having both invites drift.

### 🟠 Medium

**M1 — JWT verified twice on every mutation.**
`middleware/audit.rs:30-33` calls `verify_token` to capture the actor, then the
`AuthUser` extractor verifies the **same** token again inside the handler. Two
signature validations (and a possible JWKS fetch) per POST/PUT/PATCH/DELETE.
*Fix:* verify once in the middleware, stash the `AuthUser` in
`request.extensions`, and have the extractor read it back (falling back to
verification if absent).

**M2 — No per-record / per-patient scoping on reads & downloads.**
`report.rs:134-146` and `patient_document.rs:127-139` check only the coarse
`ReportRead`/`DocumentRead` permission — any staff member holding it can presign
**any** patient's file by ID; same for list/get across billing, clinical, etc.
For a staff-only internal tool this may be **intended** (reception needs broad
access). Flag to **confirm intent**: if row-level restrictions (e.g. a doctor
only sees their own patients) are ever required, there is currently no hook for
them. Clinical reads are at least audited (`clinical.rs:22`).

**M3 — RBAC vs. stated policy: manager can read prescriptions & lab orders.**
`rbac.rs:16-17` states clinical data is withheld from non-clinical roles, yet
`Manager` is granted `PrescriptionRead` and `LabRead` (`rbac.rs:126-127`), which
are arguably clinical. Likely deliberate (finance/oversight needs visibility of
billable orders) but **the policy comment and the README role matrix don't say
so**. Confirm intent and document it.

**M4 — Overpayment is accepted silently.**
`add_payment` (`billing.rs:200-213`) sums payments and marks `paid` once
`amount_paid >= total`, but never checks the new payment against the outstanding
balance. `amount_paid` can exceed `total` (and the generated `balance_amount`
goes negative). *Fix:* reject or explicitly allow-and-flag overpayment per
business rule.

**M5 — Docs drift: README/OpenAPI lag the router.**
The README API table documents ~8 resource groups, but `routes.rs:50-182`
exposes departments, OPD visits, clinical, prescriptions, lab orders, insurance,
patient documents, audit, invoice transactions and refunds in addition. Update
`README.md` and `openapi.yaml` (the latter is served live at
`/api/v1/openapi.yaml`, so drift is user-visible).

### 🟡 Low / polish

- **L1 — Client-supplied `content_type` stored unchecked** (`report.rs:56-60`,
  `patient_document.rs:55-57`). No MIME allowlist and no server-side sniffing;
  the value is persisted and echoed. Low risk (files are served via presigned
  URL, not inline by this API) but worth an allowlist for medical docs.
- **L2 — Malformed multipart parts silently ignored.**
  `field.name().unwrap_or_default()` (`report.rs:41`, `patient_document.rs:38`)
  maps an unnamed part to `""`, which drains into the catch-all arm — a
  mis-sent `file` part becomes "file is required" rather than a clear error.
- **L3 — Orphaned S3 object on delete.** Delete removes the DB row first, then
  deletes S3 **best-effort** (`report.rs:155-159`, `patient_document.rs:148-151`).
  A failed S3 delete leaves a pointer-less object. Acceptable, but pair with the
  H1 reconciler.
- **L4 — Offset pagination.** `models/mod.rs:28-32` clamps limit to [1,200]
  (good) but deep `OFFSET` still scans; consider keyset pagination for
  high-volume lists (invoices, audit) later.
- **L5 — CORS defaults to `Any` when unset** (`routes.rs:186-190`,
  `config.rs:65-74`). Fine for dev; ensure `CORS_ALLOWED_ORIGINS` is set in
  staging/production. (Bearer-token auth, not cookies, so `Any` is low-risk, but
  be explicit.)
- **L6 — Audit rows for failed mutations.** The middleware logs every mutation
  including 4xx/401 with `actor = None`; intended, but note audit volume and
  that unauthenticated attempts appear with a null actor.

---

## 4. Strengths (keep doing this)

- Strict layering; **zero SQL outside `repositories/`**; no `unwrap`/`expect`
  panics on request paths; all queries parameterized (the one `format!` in
  `patient.rs` interpolates a compile-time column constant, not user input).
- Central, redacting error handling with stable machine-readable `code`s.
- Money as `Decimal`/`NUMERIC`; totals computed server-side; row-locked,
  transactional payment/refund flows.
- Sensible JWKS caching with rotation handling; user-enumeration-safe login.
- Reproducible, non-root, multi-stage Docker build; migrations embedded and
  auto-applied; graceful shutdown on SIGTERM/Ctrl-C.

---

## 5. Suggested priority order

1. **H2** — fix refund status + collapse the dual payment-status fields (data
   correctness; affects financial reporting).
2. **H1 + L3** — make file upload/delete atomic (compensating delete or
   reorder) and add an orphan reconciler.
3. **M1** — stop double-verifying JWTs (latency on every write).
4. **M4** — decide and enforce the overpayment rule.
5. **M3 + M5** — reconcile RBAC policy wording and refresh README/OpenAPI.
6. **M2** — confirm whether row-level access scoping is required.
7. **L1/L2/L4/L5/L6** — hardening polish.

---

*Line references are from the source as of the review date; re-anchor if the code
has since moved.*
