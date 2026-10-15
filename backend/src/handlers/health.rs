use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use crate::error::AppResult;
use crate::state::AppState;

/// Liveness probe — the process is up. Never touches dependencies.
pub async fn liveness() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

/// Readiness probe — verifies the database is reachable.
pub async fn readiness(State(state): State<AppState>) -> AppResult<Json<Value>> {
    sqlx::query("SELECT 1").execute(&state.db).await?;
    Ok(Json(json!({ "status": "ready", "database": "ok" })))
}
