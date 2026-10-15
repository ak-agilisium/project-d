//! Tracing / logging setup.

use crate::config::Environment;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialise the global tracing subscriber.
///
/// Reads the `RUST_LOG` env var for the filter, falling back to a sane default.
/// In production, logs are emitted as JSON for ingestion by CloudWatch / ELK.
pub fn init(env: Environment) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,opd_billing_backend=debug,tower_http=info"));

    let registry = tracing_subscriber::registry().with(filter);

    if env.is_production() {
        registry
            .with(fmt::layer().json().with_current_span(true))
            .init();
    } else {
        registry
            .with(fmt::layer().pretty().with_target(true))
            .init();
    }
}
