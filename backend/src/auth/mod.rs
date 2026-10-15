//! Authentication: verifying AWS Cognito JWTs and exposing the caller identity.

pub mod jwks;
pub mod rbac;

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use jsonwebtoken::{decode, decode_header, Algorithm, Validation};
use serde::Deserialize;

use crate::auth::rbac::{Permission, Role};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// Claims we care about from a Cognito access or ID token.
#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    token_use: String,
    #[serde(default)]
    client_id: Option<String>,
    #[serde(default)]
    aud: Option<String>,
    #[serde(default)]
    username: Option<String>,
    #[serde(rename = "cognito:username", default)]
    cognito_username: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(rename = "cognito:groups", default)]
    groups: Vec<String>,
}

/// The authenticated caller, extracted from a verified bearer token.
///
/// Add it as a handler argument to require authentication:
/// ```ignore
/// async fn handler(user: AuthUser) -> impl IntoResponse { /* ... */ }
/// ```
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// Cognito `sub` — the stable unique user identifier.
    pub sub: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub groups: Vec<String>,
    pub token_use: String,
}

impl AuthUser {
    /// True if the user belongs to the given Cognito group.
    pub fn in_group(&self, group: &str) -> bool {
        self.groups.iter().any(|g| g == group)
    }

    /// The RBAC roles the user holds, derived from their Cognito groups.
    pub fn roles(&self) -> Vec<Role> {
        self.groups
            .iter()
            .filter_map(|g| Role::from_group(g))
            .collect()
    }

    /// True if the user holds the given role.
    pub fn has_role(&self, role: Role) -> bool {
        self.roles().contains(&role)
    }

    /// True if any of the user's roles grants the permission.
    pub fn has_permission(&self, perm: Permission) -> bool {
        self.roles().iter().any(|r| r.grants(perm))
    }

    /// Error with `403 Forbidden` unless the user has the permission.
    pub fn require(&self, perm: Permission) -> AppResult<()> {
        if self.has_permission(perm) {
            Ok(())
        } else {
            tracing::warn!(
                user = %self.sub,
                roles = ?self.roles(),
                permission = ?perm,
                "authorization denied"
            );
            Err(AppError::Forbidden)
        }
    }

    /// Error with `403 Forbidden` unless the user holds the given role.
    pub fn require_role(&self, role: Role) -> AppResult<()> {
        if self.has_role(role) {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("missing Authorization header".into()))?;

        let token = header
            .strip_prefix("Bearer ")
            .or_else(|| header.strip_prefix("bearer "))
            .ok_or_else(|| AppError::Unauthorized("expected 'Bearer <token>'".into()))?
            .trim();

        verify_token(state, token).await
    }
}

/// Verify a Cognito JWT and return the caller identity.
///
/// Checks: RS256 signature (against the pool JWKS), issuer, expiry, a supported
/// `token_use`, and that the token was issued for this app client.
pub async fn verify_token(state: &AppState, token: &str) -> AppResult<AuthUser> {
    let header = decode_header(token)
        .map_err(|e| AppError::Unauthorized(format!("malformed token: {e}")))?;
    let kid = header
        .kid
        .ok_or_else(|| AppError::Unauthorized("token header missing 'kid'".into()))?;

    let key = state.jwks.decoding_key(&kid).await?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[state.config.cognito_issuer()]);
    // Cognito access tokens have no `aud` claim (they use `client_id`), so we
    // disable automatic audience validation and check the client id manually.
    validation.validate_aud = false;
    validation.set_required_spec_claims(&["exp", "iss", "sub"]);

    let data = decode::<Claims>(token, &key, &validation)
        .map_err(|e| AppError::Unauthorized(format!("invalid token: {e}")))?;
    let claims = data.claims;

    if claims.token_use != "access" && claims.token_use != "id" {
        return Err(AppError::Unauthorized("unsupported token_use".into()));
    }

    let expected = state.config.aws.cognito_client_id.as_str();
    let client_ok =
        claims.client_id.as_deref() == Some(expected) || claims.aud.as_deref() == Some(expected);
    if !client_ok {
        return Err(AppError::Unauthorized(
            "token was not issued for this application".into(),
        ));
    }

    Ok(AuthUser {
        sub: claims.sub,
        username: claims.username.or(claims.cognito_username),
        email: claims.email,
        groups: claims.groups,
        token_use: claims.token_use,
    })
}
