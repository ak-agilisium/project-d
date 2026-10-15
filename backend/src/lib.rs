//! Medical billing & patient OPD booking backend.
//!
//! Layers: `handlers` (HTTP) → `services`/`repositories` (AWS & DB) → `models`.
//! Cross-cutting: `config`, `error`, `auth` (Cognito JWT + RBAC), `telemetry`.

pub mod auth;
pub mod config;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod state;
pub mod telemetry;

use anyhow::Context;

use crate::config::Config;
use crate::state::AppState;

/// Load configuration, initialise state, and run the HTTP server until a
/// shutdown signal is received.
pub async fn run() -> anyhow::Result<()> {
    // Populate the environment from a local `.env` during development.
    let _ = dotenvy::dotenv();

    let config = Config::from_env().context("failed to load configuration")?;
    telemetry::init(config.app.env);

    tracing::info!(env = ?config.app.env, "starting opd-billing-backend");

    let bind_addr = format!("{}:{}", config.app.host, config.app.port);
    let run_migrations = config.database.run_migrations;

    let state = AppState::init(config).await?;
    if run_migrations {
        state.run_migrations().await?;
    }

    let app = routes::build_router(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind {bind_addr}"))?;
    tracing::info!(address = %bind_addr, "server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    tracing::info!("server shut down cleanly");
    Ok(())
}

/// Resolve when the process receives Ctrl+C or (on Unix) SIGTERM.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
