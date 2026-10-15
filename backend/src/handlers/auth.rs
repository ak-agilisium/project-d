use axum::extract::State;
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use validator::Validate;

use crate::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::handlers::validate;
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 256))]
    pub password: String,
    #[validate(length(min = 1, max = 100))]
    pub given_name: String,
    #[validate(length(min = 1, max = 100))]
    pub family_name: String,
    #[validate(length(max = 32))]
    pub phone_number: Option<String>,
}

/// POST /api/v1/auth/register — create a user in Cognito.
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> AppResult<impl IntoResponse> {
    validate(&body)?;
    let result = state
        .cognito
        .sign_up(
            &body.email,
            &body.password,
            &body.given_name,
            &body.family_name,
            body.phone_number.as_deref(),
        )
        .await?;

    tracing::info!(user_sub = %result.user_sub, "registered new Cognito user");

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "user_sub": result.user_sub,
            "confirmed": result.confirmed,
            "confirmation_sent_to": result.code_delivery_destination,
            "message": "registration successful; confirm the account using the code we sent"
        })),
    ))
}

#[derive(Debug, Deserialize, Validate)]
pub struct ConfirmRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 16))]
    pub code: String,
}

/// POST /api/v1/auth/confirm — confirm a sign-up with the emailed code.
pub async fn confirm(
    State(state): State<AppState>,
    Json(body): Json<ConfirmRequest>,
) -> AppResult<Json<Value>> {
    validate(&body)?;
    state
        .cognito
        .confirm_sign_up(&body.email, &body.code)
        .await?;
    Ok(Json(json!({ "message": "account confirmed" })))
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResendRequest {
    #[validate(email)]
    pub email: String,
}

/// POST /api/v1/auth/resend-code — resend the confirmation code.
pub async fn resend_code(
    State(state): State<AppState>,
    Json(body): Json<ResendRequest>,
) -> AppResult<Json<Value>> {
    validate(&body)?;
    let destination = state.cognito.resend_confirmation_code(&body.email).await?;
    Ok(Json(
        json!({ "message": "confirmation code resent", "sent_to": destination }),
    ))
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: Option<String>,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: i32,
}

/// POST /api/v1/auth/login — exchange credentials for Cognito tokens.
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> AppResult<Json<TokenResponse>> {
    validate(&body)?;
    let tokens = state.cognito.login(&body.email, &body.password).await?;
    Ok(Json(TokenResponse {
        access_token: tokens.access_token,
        id_token: tokens.id_token,
        refresh_token: tokens.refresh_token,
        token_type: tokens.token_type.unwrap_or_else(|| "Bearer".to_string()),
        expires_in: tokens.expires_in,
    }))
}

/// GET /api/v1/auth/me — the authenticated caller's identity and roles.
pub async fn me(user: AuthUser) -> Json<Value> {
    let roles: Vec<String> = user.roles().iter().map(|r| r.to_string()).collect();
    Json(json!({
        "sub": user.sub,
        "username": user.username,
        "email": user.email,
        "groups": user.groups,
        "roles": roles,
        "token_use": user.token_use,
    }))
}

#[derive(Debug, Deserialize, Validate)]
pub struct RefreshRequest {
    #[validate(length(min = 1))]
    pub refresh_token: String,
    /// The user's Cognito username (the `sub` from a prior login / `/auth/me`).
    /// Required to compute `SECRET_HASH` for confidential app clients.
    #[validate(length(min = 1))]
    pub username: String,
}

/// POST /api/v1/auth/refresh — exchange a refresh token for new tokens.
pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> AppResult<Json<TokenResponse>> {
    validate(&body)?;
    let tokens = state
        .cognito
        .refresh(&body.username, &body.refresh_token)
        .await?;
    Ok(Json(TokenResponse {
        access_token: tokens.access_token,
        id_token: tokens.id_token,
        refresh_token: tokens.refresh_token,
        token_type: tokens.token_type.unwrap_or_else(|| "Bearer".to_string()),
        expires_in: tokens.expires_in,
    }))
}

/// POST /api/v1/auth/logout — globally sign out, revoking all of the user's
/// tokens. Requires a valid access token in the `Authorization` header.
pub async fn logout(
    user: AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<Value>> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| {
            h.strip_prefix("Bearer ")
                .or_else(|| h.strip_prefix("bearer "))
        })
        .map(str::trim)
        .ok_or_else(|| AppError::Unauthorized("expected 'Bearer <token>'".into()))?;

    state.cognito.logout(token).await?;
    tracing::info!(user = %user.sub, "user logged out (global sign-out)");
    Ok(Json(json!({ "message": "logged out" })))
}
