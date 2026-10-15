# OPD Billing Backend

Production-grade Rust backend for **medical billing & patient OPD booking**.

- **Web:** [Axum](https://github.com/tokio-rs/axum) (Tokio async)
- **Auth:** AWS **Cognito** (registration + login), JWTs verified against the pool JWKS
- **AuthZ:** Role-based access control (`super_admin`, `admin`, `manager`, `doctor`, `front_office`)
- **Database:** AWS **RDS for PostgreSQL** via [SQLx](https://github.com/launchbadge/sqlx)
- **Files:** Patient **reports are stored in Amazon S3 only** (DB keeps metadata pointers)
- **API:** versioned under `/api/v1`, consistent JSON error envelope, structured `tracing` logs

---

## Architecture

```
HTTP ─▶ handlers/      controllers: authorize (RBAC) → validate → delegate
        services/      AWS clients  (Cognito, S3)
        repositories/  data access  (all SQL lives here)
        models/        domain types (DB rows) + request/response DTOs
        auth/          Cognito JWT verification (JWKS) + RBAC
        config, error, telemetry, state, routes
```

- **Central error handling** — every layer returns `AppError` (`src/error.rs`), which
  implements `IntoResponse` and renders one JSON shape. Server faults are logged in
  full and redacted from the client response.
- **Central logging** — `tracing` + `tower_http::TraceLayer`; JSON logs in production.
- **The database is never touched from handlers** — all SQL is in `repositories/`.

---

## API

Base path: **`/api/v1`**. All endpoints except health and `/auth/*` require a
`Authorization: Bearer <cognito-token>` header.

| Method & path | Permission | Purpose |
|---|---|---|
| `GET /health`, `GET /health/ready` | public | liveness / readiness |
| `POST /api/v1/auth/register` | public | register a user in Cognito |
| `POST /api/v1/auth/confirm` | public | confirm sign-up with emailed code |
| `POST /api/v1/auth/resend-code` | public | resend confirmation code |
| `POST /api/v1/auth/login` | public | get Cognito tokens |
| `POST /api/v1/auth/refresh` | public | exchange a refresh token for new tokens |
| `POST /api/v1/auth/logout` | any authed | global sign-out (revokes all tokens) |
| `GET  /api/v1/auth/me` | any authed | current identity + roles |
| `GET  /api/v1/openapi.yaml` | public | OpenAPI 3.0 spec for the API |
| `POST/GET /api/v1/patients` | PatientWrite / PatientRead | create / list patients |
| `GET/PUT/DELETE /api/v1/patients/{id}` | PatientRead / Write / Delete | patient CRUD |
| `POST/GET /api/v1/doctors` | DoctorWrite / DoctorRead | create / list doctors |
| `GET/DELETE /api/v1/doctors/{id}` | DoctorRead / DoctorWrite | get / deactivate |
| `POST/GET /api/v1/appointments` | AppointmentWrite / Read | book / list OPD appointments |
| `GET/PATCH /api/v1/appointments/{id}` | AppointmentRead / Write | get / update |
| `POST /api/v1/appointments/{id}/cancel` | AppointmentWrite | cancel |
| `POST/GET /api/v1/invoices` | BillingWrite / BillingRead | create / list invoices |
| `GET /api/v1/invoices/{id}` | BillingRead | invoice with items & payments |
| `POST /api/v1/invoices/{id}/payments` | BillingWrite | record a payment |
| `POST /api/v1/reports` | ReportWrite | upload a report file to S3 (multipart) |
| `GET /api/v1/patients/{id}/reports` | ReportRead | list a patient's reports |
| `GET /api/v1/reports/{id}/download` | ReportRead | presigned S3 download URL |
| `DELETE /api/v1/reports/{id}` | ReportDelete | delete record + S3 object |

### Error envelope

```json
{ "error": { "code": "validation_error", "message": "..." } }
```

---

## Role → permission matrix

Roles come from the Cognito `cognito:groups` claim. Create groups named exactly
`super_admin`, `admin`, `manager`, `doctor`, `front_office`.

| Capability | super_admin | admin | manager | doctor | front_office |
|---|:--:|:--:|:--:|:--:|:--:|
| Patient read / write / delete | ✅ all | ✅ all | 👁 read | 👁 read | ✍ read+write |
| Doctor read / write | ✅ | ✅ | 👁 read | 👁 read | 👁 read |
| Appointment read / write / delete | ✅ all | ✅ all | 👁 read | ✍ read+write | ✍ read+write |
| Billing read / write | ✅ | ✅ | ✍ | 👁 read | ✍ |
| Report read / write / delete | ✅ all | ✅ all | ✍ read+write | ✍ read+write | ✍ read+write |
| User management | ✅ | ✅ | — | — | — |

Edit the matrix in [`src/auth/rbac.rs`](src/auth/rbac.rs).

---

## Prerequisites

1. **Rust** (stable, see `rust-toolchain.toml`).
2. **AWS resources:**
   - A **Cognito User Pool** + an **app client** with the `USER_PASSWORD_AUTH`
     flow enabled. If the client has a secret, set `COGNITO_CLIENT_SECRET`.
     Create the five groups listed above.
   - An **S3 bucket** for reports (block public access; the API serves files via
     short-lived presigned URLs).
   - An **RDS PostgreSQL** instance.
3. **AWS credentials** available via the default provider chain (env vars, a
   shared profile, or — in production — an **IAM role** with least-privilege
   access to the pool, the bucket, and nothing else).

### Minimal IAM policy (sketch)

```json
{
  "Version": "2012-10-17",
  "Statement": [
    { "Effect": "Allow",
      "Action": ["cognito-idp:SignUp","cognito-idp:ConfirmSignUp",
                 "cognito-idp:ResendConfirmationCode","cognito-idp:InitiateAuth"],
      "Resource": "arn:aws:cognito-idp:<region>:<acct>:userpool/<pool-id>" },
    { "Effect": "Allow",
      "Action": ["s3:PutObject","s3:GetObject","s3:DeleteObject"],
      "Resource": "arn:aws:s3:::<report-bucket>/*" }
  ]
}
```

---

## Local development

```bash
cp .env.example .env          # then fill in Cognito / S3 / AWS values
docker compose up -d db       # local Postgres (mirrors RDS)
cargo run                     # migrations run on startup (RUN_MIGRATIONS=true)
```

The API listens on `http://localhost:8080`.

### Smoke test

```bash
curl localhost:8080/health

curl -X POST localhost:8080/api/v1/auth/register \
  -H 'content-type: application/json' \
  -d '{"email":"a@b.com","password":"S3cret!pass","given_name":"Ada","family_name":"Lovelace"}'

TOKEN=... # access_token from /auth/login
curl -X POST localhost:8080/api/v1/reports \
  -H "authorization: Bearer $TOKEN" \
  -F patient_id=<uuid> -F report_type=lab -F file=@./result.pdf
```

---

## Developer tooling

- **OpenAPI spec:** [`openapi.yaml`](openapi.yaml), also served live at
  `GET /api/v1/openapi.yaml`. Import the URL into Swagger UI, Postman, Insomnia,
  or Bruno.
- **Request collection:** [`api.http`](api.http) — run with the VS Code
  "REST Client" extension or the JetBrains HTTP client. Run `login` first; the
  bearer token is captured and reused automatically.
- **Seed script:** [`scripts/seed.sh`](scripts/seed.sh) creates a demo doctor,
  patient, OPD appointment, issued invoice, and payment through the live API
  (it provisions a `super_admin` seed user via the AWS CLI). Run with the server
  up: `./scripts/seed.sh`.

## Configuration

All configuration is via environment variables — see [`.env.example`](.env.example)
for the full list (app, database, AWS region, Cognito, S3).

---

## Migrations

SQL migrations live in [`migrations/`](migrations) and are embedded in the binary
(`sqlx::migrate!`). They run automatically on startup unless `RUN_MIGRATIONS=false`.
To run manually with the [sqlx-cli](https://crates.io/crates/sqlx-cli):

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx migrate run
```

> This project uses **runtime-checked** queries (`sqlx::query_as`), so the code
> compiles without a live database. If you switch to the compile-time `query!`
> macros, generate an offline cache with `cargo sqlx prepare` and commit `.sqlx/`.

---

## Build & deploy

```bash
cargo build --release          # binary at target/release/opd-billing-backend
docker build -t opd-billing-backend .
```

The multi-stage `Dockerfile` produces a slim runtime image that runs as a
non-root user and bundles the migrations.

---

## Project layout

```
migrations/0001_init.sql        schema (patients, doctors, appointments,
                                invoices, invoice_items, payments, reports)
src/
  main.rs / lib.rs              entrypoint + server bootstrap
  config.rs                     typed env configuration
  error.rs                      AppError + JSON IntoResponse (central handling)
  telemetry.rs                  tracing setup
  state.rs                      AppState (DB pool + AWS clients + JWKS cache)
  routes.rs                     /api/v1 router + middleware
  auth/                         Cognito JWT verification (jwks) + RBAC (rbac)
  services/                     cognito.rs, s3.rs
  models/                       domain + DTOs
  repositories/                 all SQL
  handlers/                     HTTP controllers
```
