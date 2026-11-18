use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::state::AppState;

/// Healthcheck do rustploy: 200 só se o banco responde.
pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "ok" }))),
        Err(error) => {
            tracing::error!(%error, "healthcheck: banco indisponível");
            (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "status": "database_unavailable" })))
        }
    }
}
