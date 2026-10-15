//! Audit middleware: records every mutating request to `audit_logs`.
//!
//! Captures the verified caller (Cognito `sub` + roles), method, path, response
//! status and client IP. Read requests are not logged here (sensitive reads,
//! e.g. clinical records, are audited explicitly in their handlers).

use axum::extract::Request;
use axum::http::header::AUTHORIZATION;
use axum::http::Method;
use axum::middleware::Next;
use axum::response::Response;

use crate::auth::verify_token;
use crate::repositories::audit;
use crate::state::AppState;

pub async fn record_mutations(state: AppState, req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let is_mutation = matches!(
        method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    );

    // Snapshot everything we need from the request *synchronously* — never hold
    // a `&Request` across an await (the body is Send but not Sync).
    let path = req.uri().path().to_string();
    let ip = client_ip(&req);
    let token = if is_mutation { bearer_token(&req) } else { None };

    let actor = match token {
        Some(t) => verify_token(&state, &t).await.ok(),
        None => None,
    };

    let response = next.run(req).await;

    if is_mutation {
        let status = response.status().as_u16() as i32;
        audit::record_http(
            &state.db,
            actor.as_ref(),
            method.as_str(),
            &path,
            status,
            ip.as_deref(),
        )
        .await;
    }

    response
}

/// Synchronously extract the owned bearer token from the request headers.
fn bearer_token(req: &Request) -> Option<String> {
    let header = req.headers().get(AUTHORIZATION)?.to_str().ok()?;
    header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .map(|t| t.trim().to_string())
}

fn client_ip(req: &Request) -> Option<String> {
    let headers = req.headers();
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        })
}
