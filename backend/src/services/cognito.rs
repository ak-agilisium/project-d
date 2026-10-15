//! Thin wrapper over the AWS Cognito Identity Provider SDK for user
//! registration and authentication.

use aws_sdk_cognitoidentityprovider::operation::confirm_sign_up::ConfirmSignUpError;
use aws_sdk_cognitoidentityprovider::operation::initiate_auth::InitiateAuthError;
use aws_sdk_cognitoidentityprovider::operation::resend_confirmation_code::ResendConfirmationCodeError;
use aws_sdk_cognitoidentityprovider::operation::sign_up::SignUpError;
use aws_sdk_cognitoidentityprovider::types::{AttributeType, AuthFlowType};
use aws_sdk_cognitoidentityprovider::Client;
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::error::{AppError, AppResult};

type HmacSha256 = Hmac<Sha256>;

/// Wrapper around the Cognito client bound to a single app client.
pub struct CognitoService {
    client: Client,
    client_id: String,
    client_secret: Option<String>,
}

/// Result of a successful sign-up call.
#[derive(Debug)]
pub struct SignUpResult {
    pub user_sub: String,
    pub confirmed: bool,
    /// Masked destination the confirmation code was sent to (e.g. `a***@b.com`).
    pub code_delivery_destination: Option<String>,
}

/// Tokens returned by a successful authentication.
#[derive(Debug, Default)]
pub struct AuthTokens {
    pub access_token: Option<String>,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub expires_in: i32,
}

impl CognitoService {
    pub fn new(client: Client, client_id: String, client_secret: Option<String>) -> Self {
        Self {
            client,
            client_id,
            client_secret,
        }
    }

    /// Compute the `SECRET_HASH` required when the app client has a secret.
    /// Returns `None` when no client secret is configured.
    fn secret_hash(&self, username: &str) -> AppResult<Option<String>> {
        let Some(secret) = self.client_secret.as_deref() else {
            return Ok(None);
        };
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid client secret: {e}")))?;
        mac.update(username.as_bytes());
        mac.update(self.client_id.as_bytes());
        let digest = mac.finalize().into_bytes();
        Ok(Some(
            base64::engine::general_purpose::STANDARD.encode(digest),
        ))
    }

    /// Register a new user. The email doubles as the username.
    pub async fn sign_up(
        &self,
        email: &str,
        password: &str,
        given_name: &str,
        family_name: &str,
        phone_number: Option<&str>,
    ) -> AppResult<SignUpResult> {
        let mut attributes = vec![
            attribute("email", email)?,
            attribute("given_name", given_name)?,
            attribute("family_name", family_name)?,
        ];
        if let Some(phone) = phone_number {
            attributes.push(attribute("phone_number", phone)?);
        }

        let output = self
            .client
            .sign_up()
            .client_id(&self.client_id)
            .set_secret_hash(self.secret_hash(email)?)
            .username(email)
            .password(password)
            .set_user_attributes(Some(attributes))
            .send()
            .await
            .map_err(|e| match e.as_service_error() {
                Some(SignUpError::UsernameExistsException(_)) => {
                    AppError::Conflict("an account with this email already exists".into())
                }
                Some(SignUpError::InvalidPasswordException(_)) => {
                    AppError::BadRequest("password does not meet the pool's policy".into())
                }
                Some(SignUpError::InvalidParameterException(_)) => {
                    AppError::BadRequest("one or more registration fields are invalid".into())
                }
                _ => AppError::Aws(format!("cognito sign_up failed: {e}")),
            })?;

        Ok(SignUpResult {
            user_sub: output.user_sub().to_string(),
            confirmed: output.user_confirmed(),
            code_delivery_destination: output
                .code_delivery_details()
                .and_then(|d| d.destination())
                .map(String::from),
        })
    }

    /// Confirm a sign-up using the code emailed/SMSed to the user.
    pub async fn confirm_sign_up(&self, username: &str, code: &str) -> AppResult<()> {
        self.client
            .confirm_sign_up()
            .client_id(&self.client_id)
            .set_secret_hash(self.secret_hash(username)?)
            .username(username)
            .confirmation_code(code)
            .send()
            .await
            .map_err(|e| match e.as_service_error() {
                Some(ConfirmSignUpError::CodeMismatchException(_)) => {
                    AppError::BadRequest("the confirmation code is incorrect".into())
                }
                Some(ConfirmSignUpError::ExpiredCodeException(_)) => {
                    AppError::BadRequest("the confirmation code has expired".into())
                }
                Some(ConfirmSignUpError::UserNotFoundException(_)) => AppError::NotFound,
                Some(ConfirmSignUpError::NotAuthorizedException(_)) => {
                    AppError::Conflict("the account is already confirmed".into())
                }
                _ => AppError::Aws(format!("cognito confirm_sign_up failed: {e}")),
            })?;
        Ok(())
    }

    /// Resend the confirmation code to a not-yet-confirmed user.
    pub async fn resend_confirmation_code(&self, username: &str) -> AppResult<Option<String>> {
        let output = self
            .client
            .resend_confirmation_code()
            .client_id(&self.client_id)
            .set_secret_hash(self.secret_hash(username)?)
            .username(username)
            .send()
            .await
            .map_err(|e| match e.as_service_error() {
                Some(ResendConfirmationCodeError::UserNotFoundException(_)) => AppError::NotFound,
                _ => AppError::Aws(format!("cognito resend_confirmation_code failed: {e}")),
            })?;

        Ok(output
            .code_delivery_details()
            .and_then(|d| d.destination())
            .map(String::from))
    }

    /// Authenticate with username + password (USER_PASSWORD_AUTH flow).
    pub async fn login(&self, username: &str, password: &str) -> AppResult<AuthTokens> {
        let mut params = std::collections::HashMap::new();
        params.insert("USERNAME".to_string(), username.to_string());
        params.insert("PASSWORD".to_string(), password.to_string());
        if let Some(hash) = self.secret_hash(username)? {
            params.insert("SECRET_HASH".to_string(), hash);
        }

        let output = self
            .client
            .initiate_auth()
            .auth_flow(AuthFlowType::UserPasswordAuth)
            .client_id(&self.client_id)
            .set_auth_parameters(Some(params))
            .send()
            .await
            .map_err(|e| match e.as_service_error() {
                // Treat unknown-user and bad-password alike to avoid user enumeration.
                Some(InitiateAuthError::NotAuthorizedException(_))
                | Some(InitiateAuthError::UserNotFoundException(_)) => {
                    AppError::Unauthorized("incorrect email or password".into())
                }
                Some(InitiateAuthError::UserNotConfirmedException(_)) => AppError::Forbidden,
                _ => AppError::Aws(format!("cognito initiate_auth failed: {e}")),
            })?;

        let result = output.authentication_result().ok_or_else(|| {
            let challenge = output
                .challenge_name()
                .map(|c| c.as_str().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            AppError::BadRequest(format!("authentication requires challenge: {challenge}"))
        })?;

        Ok(AuthTokens {
            access_token: result.access_token().map(String::from),
            id_token: result.id_token().map(String::from),
            refresh_token: result.refresh_token().map(String::from),
            token_type: result.token_type().map(String::from),
            expires_in: result.expires_in(),
        })
    }

    /// Exchange a refresh token for fresh access/ID tokens (REFRESH_TOKEN_AUTH).
    ///
    /// When the app client has a secret, Cognito requires a `SECRET_HASH`
    /// computed from the user's `username` (their Cognito `sub`), so the caller
    /// must supply it alongside the refresh token. No new refresh token is
    /// issued — the original keeps working until it expires or is revoked.
    pub async fn refresh(&self, username: &str, refresh_token: &str) -> AppResult<AuthTokens> {
        let mut params = std::collections::HashMap::new();
        params.insert("REFRESH_TOKEN".to_string(), refresh_token.to_string());
        if let Some(hash) = self.secret_hash(username)? {
            params.insert("SECRET_HASH".to_string(), hash);
        }

        let output = self
            .client
            .initiate_auth()
            .auth_flow(AuthFlowType::RefreshTokenAuth)
            .client_id(&self.client_id)
            .set_auth_parameters(Some(params))
            .send()
            .await
            .map_err(|e| match e.as_service_error() {
                Some(InitiateAuthError::NotAuthorizedException(_)) => {
                    AppError::Unauthorized("refresh token is invalid or revoked".into())
                }
                _ => AppError::Aws(format!("cognito refresh failed: {e}")),
            })?;

        let result = output
            .authentication_result()
            .ok_or_else(|| AppError::Unauthorized("refresh did not return tokens".into()))?;

        Ok(AuthTokens {
            access_token: result.access_token().map(String::from),
            id_token: result.id_token().map(String::from),
            // Refresh flow reuses the caller's existing refresh token.
            refresh_token: None,
            token_type: result.token_type().map(String::from),
            expires_in: result.expires_in(),
        })
    }

    /// Globally sign the user out, revoking all of their issued tokens.
    pub async fn logout(&self, access_token: &str) -> AppResult<()> {
        self.client
            .global_sign_out()
            .access_token(access_token)
            .send()
            .await
            .map_err(|e| AppError::Aws(format!("cognito global_sign_out failed: {e}")))?;
        Ok(())
    }
}

fn attribute(name: &str, value: &str) -> AppResult<AttributeType> {
    AttributeType::builder()
        .name(name)
        .value(value)
        .build()
        .map_err(|e| AppError::Internal(e.into()))
}
