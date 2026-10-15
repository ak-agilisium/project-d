//! Shared application state, cloned (cheaply, via `Arc`) into every handler.

use std::sync::Arc;

use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::auth::jwks::JwksCache;
use crate::config::Config;
use crate::services::{CognitoService, S3Service};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: PgPool,
    pub cognito: Arc<CognitoService>,
    pub s3: Arc<S3Service>,
    pub jwks: Arc<JwksCache>,
}

impl AppState {
    /// Build the application state: connect to Postgres and construct the AWS
    /// service clients from the default credential provider chain.
    pub async fn init(config: Config) -> anyhow::Result<Self> {
        let db = PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .connect(&config.database.url)
            .await
            .context("failed to connect to the Postgres database")?;
        tracing::info!("connected to Postgres");

        let aws_shared = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(config.aws.region.clone()))
            .load()
            .await;

        let cognito = CognitoService::new(
            aws_sdk_cognitoidentityprovider::Client::new(&aws_shared),
            config.aws.cognito_client_id.clone(),
            config.aws.cognito_client_secret.clone(),
        );

        let s3 = S3Service::new(
            aws_sdk_s3::Client::new(&aws_shared),
            config.aws.s3_report_bucket.clone(),
            config.aws.s3_presign_expiry,
        );

        let jwks = JwksCache::new(config.cognito_jwks_url());

        Ok(Self {
            config: Arc::new(config),
            db,
            cognito: Arc::new(cognito),
            s3: Arc::new(s3),
            jwks: Arc::new(jwks),
        })
    }

    /// Apply embedded SQL migrations to the database.
    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.db)
            .await
            .context("database migration failed")?;
        tracing::info!("database migrations applied");
        Ok(())
    }
}
