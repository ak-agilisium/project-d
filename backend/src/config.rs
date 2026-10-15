//! Strongly-typed application configuration, loaded from environment variables.

use std::time::Duration;

/// Top-level configuration assembled at startup. Cheap to clone-share via `Arc`.
#[derive(Debug, Clone)]
pub struct Config {
    pub app: AppConfig,
    pub database: DatabaseConfig,
    pub aws: AwsConfig,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub env: Environment,
    pub host: String,
    pub port: u16,
    /// Allowed CORS origins. `None` means "allow any" (development convenience).
    pub cors_allowed_origins: Option<Vec<String>>,
    pub max_upload_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub run_migrations: bool,
}

#[derive(Debug, Clone)]
pub struct AwsConfig {
    pub region: String,
    pub cognito_user_pool_id: String,
    pub cognito_client_id: String,
    pub cognito_client_secret: Option<String>,
    pub s3_report_bucket: String,
    pub s3_presign_expiry: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    pub fn is_production(self) -> bool {
        matches!(self, Environment::Production)
    }
}

impl Config {
    /// Load configuration from the process environment.
    ///
    /// Call [`dotenvy::dotenv`] beforehand to populate the environment from a
    /// local `.env` file during development.
    pub fn from_env() -> Result<Self, ConfigError> {
        let env = match optional("APP_ENV").as_deref() {
            Some("production") | Some("prod") => Environment::Production,
            Some("staging") | Some("stage") => Environment::Staging,
            _ => Environment::Development,
        };

        let cors_allowed_origins = match optional("CORS_ALLOWED_ORIGINS") {
            None => None,
            Some(v) if v.trim() == "*" || v.trim().is_empty() => None,
            Some(v) => Some(
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
            ),
        };

        let app = AppConfig {
            env,
            host: optional("APP_HOST").unwrap_or_else(|| "0.0.0.0".to_string()),
            port: parse_or("APP_PORT", 8080)?,
            cors_allowed_origins,
            max_upload_bytes: parse_or("MAX_UPLOAD_BYTES", 25 * 1024 * 1024)?,
        };

        let database = DatabaseConfig {
            url: required("DATABASE_URL")?,
            max_connections: parse_or("DATABASE_MAX_CONNECTIONS", 10)?,
            run_migrations: parse_or("RUN_MIGRATIONS", true)?,
        };

        let aws = AwsConfig {
            region: optional("AWS_REGION").unwrap_or_else(|| "us-east-1".to_string()),
            cognito_user_pool_id: required("COGNITO_USER_POOL_ID")?,
            cognito_client_id: required("COGNITO_CLIENT_ID")?,
            cognito_client_secret: optional("COGNITO_CLIENT_SECRET"),
            s3_report_bucket: required("S3_REPORT_BUCKET")?,
            s3_presign_expiry: Duration::from_secs(parse_or("S3_PRESIGN_EXPIRY_SECS", 900)?),
        };

        Ok(Self { app, database, aws })
    }

    /// The Cognito token issuer URL for this user pool.
    pub fn cognito_issuer(&self) -> String {
        format!(
            "https://cognito-idp.{}.amazonaws.com/{}",
            self.aws.region, self.aws.cognito_user_pool_id
        )
    }

    /// The JWKS endpoint used to verify Cognito JWT signatures.
    pub fn cognito_jwks_url(&self) -> String {
        format!("{}/.well-known/jwks.json", self.cognito_issuer())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    Missing(&'static str),
    #[error("invalid value for environment variable {name}: {source}")]
    Invalid {
        name: &'static str,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

fn optional(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

fn required(name: &'static str) -> Result<String, ConfigError> {
    optional(name).ok_or(ConfigError::Missing(name))
}

fn parse_or<T>(name: &'static str, default: T) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match optional(name) {
        None => Ok(default),
        Some(v) => v.parse::<T>().map_err(|e| ConfigError::Invalid {
            name,
            source: Box::new(e),
        }),
    }
}
