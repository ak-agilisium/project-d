//! Router assembly. All business endpoints live under `/api/v1`; health probes
//! sit at the root. Cross-cutting concerns (tracing, timeouts, body limits,
//! CORS) are applied here as middleware so handlers stay focused.

use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderValue, StatusCode};
use axum::routing::{delete, get, patch, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::state::AppState;
use crate::{handlers, middleware};

pub fn build_router(state: AppState) -> Router {
    // Allow report uploads up to the configured limit (plus multipart overhead).
    let body_limit = state.config.app.max_upload_bytes + 1024 * 1024;
    let cors = cors_layer(&state);
    // Capture state in a closure so the middleware doesn't depend on router state.
    let audit_state = state.clone();
    let audit = axum::middleware::from_fn(move |req, next| {
        let state = audit_state.clone();
        async move { middleware::audit::record_mutations(state, req, next).await }
    });

    Router::new()
        // ---- Health probes (unversioned, unauthenticated) ----
        .route("/health", get(handlers::health::liveness))
        .route("/health/ready", get(handlers::health::readiness))
        // ---- Interactive API docs (Swagger UI) ----
        .route("/docs", get(handlers::docs::swagger_ui))
        // ---- Versioned API ----
        .nest("/api/v1", api_v1())
        // ---- Middleware (outermost listed first) ----
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(DefaultBodyLimit::max(body_limit))
        .layer(cors)
        .with_state(state)
        // Audit every mutation (outermost: sees the final response status).
        .layer(audit)
}

fn api_v1() -> Router<AppState> {
    Router::new()
        // ---- Auth (public: register / confirm / login) ----
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/confirm", post(handlers::auth::confirm))
        .route("/auth/resend-code", post(handlers::auth::resend_code))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/refresh", post(handlers::auth::refresh))
        .route("/auth/logout", post(handlers::auth::logout))
        .route("/auth/me", get(handlers::auth::me))
        // ---- API docs ----
        .route("/openapi.yaml", get(handlers::docs::openapi))
        // ---- Patients ----
        .route(
            "/patients",
            post(handlers::patient::create).get(handlers::patient::list),
        )
        .route(
            "/patients/{id}",
            get(handlers::patient::get)
                .put(handlers::patient::update)
                .delete(handlers::patient::delete),
        )
        .route(
            "/patients/{patient_id}/reports",
            get(handlers::report::list_by_patient),
        )
        // ---- Doctors ----
        .route(
            "/doctors",
            post(handlers::doctor::create).get(handlers::doctor::list),
        )
        .route(
            "/doctors/{id}",
            get(handlers::doctor::get).delete(handlers::doctor::deactivate),
        )
        // ---- Departments ----
        .route(
            "/departments",
            post(handlers::department::create).get(handlers::department::list),
        )
        .route("/departments/{id}", get(handlers::department::get))
        // ---- OPD visits ----
        .route(
            "/visits",
            post(handlers::opd_visit::create).get(handlers::opd_visit::list),
        )
        .route(
            "/visits/{id}",
            get(handlers::opd_visit::get).patch(handlers::opd_visit::update),
        )
        // ---- Clinical records (RBAC-restricted) ----
        .route(
            "/visits/{id}/clinical",
            get(handlers::clinical::get).put(handlers::clinical::upsert),
        )
        // ---- Prescriptions (per visit) ----
        .route(
            "/visits/{id}/prescriptions",
            post(handlers::prescription::create).get(handlers::prescription::list),
        )
        .route(
            "/prescriptions/{id}",
            delete(handlers::prescription::delete),
        )
        // ---- Lab orders (per visit) ----
        .route(
            "/visits/{id}/lab-orders",
            post(handlers::lab_order::create).get(handlers::lab_order::list),
        )
        .route("/lab-orders/{id}", patch(handlers::lab_order::update))
        // ---- Patient insurance ----
        .route(
            "/patients/{patient_id}/insurance",
            post(handlers::patient_insurance::create).get(handlers::patient_insurance::list),
        )
        .route(
            "/insurance/{id}",
            patch(handlers::patient_insurance::update),
        )
        // ---- Patient documents (S3) ----
        .route("/documents", post(handlers::patient_document::upload))
        .route(
            "/patients/{patient_id}/documents",
            get(handlers::patient_document::list_by_patient),
        )
        .route(
            "/documents/{id}/download",
            get(handlers::patient_document::download),
        )
        .route(
            "/documents/{id}",
            delete(handlers::patient_document::delete),
        )
        // ---- Appointments (OPD booking) ----
        .route(
            "/appointments",
            post(handlers::appointment::create).get(handlers::appointment::list),
        )
        .route(
            "/appointments/{id}",
            get(handlers::appointment::get).patch(handlers::appointment::update),
        )
        .route(
            "/appointments/{id}/cancel",
            post(handlers::appointment::cancel),
        )
        // ---- Billing ----
        .route(
            "/invoices",
            post(handlers::billing::create_invoice).get(handlers::billing::list_invoices),
        )
        .route("/invoices/{id}", get(handlers::billing::get_invoice))
        .route(
            "/invoices/{id}/payments",
            post(handlers::billing::add_payment),
        )
        .route(
            "/invoices/{id}/transactions",
            get(handlers::billing::transactions),
        )
        .route("/invoices/{id}/refund", post(handlers::billing::refund))
        // ---- Audit trail ----
        .route("/audit", get(handlers::audit::list))
        // ---- Reports (stored in S3) ----
        .route("/reports", post(handlers::report::upload))
        .route("/reports/{id}/download", get(handlers::report::download))
        .route("/reports/{id}", delete(handlers::report::delete))
}

fn cors_layer(state: &AppState) -> CorsLayer {
    match &state.config.app.cors_allowed_origins {
        // Development default: allow any origin.
        None => CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
        Some(origins) => {
            let allowed: Vec<HeaderValue> = origins.iter().filter_map(|o| o.parse().ok()).collect();
            CorsLayer::new()
                .allow_origin(allowed)
                .allow_methods(Any)
                .allow_headers(Any)
        }
    }
}
