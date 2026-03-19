use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::AppState;

/// GET /health
#[utoipa::path(
    get, path = "/health",
    responses(
        (status = 200, description = "Service is healthy"),
    ),
    tag = "meta"
)]
pub async fn health(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    let db_ok = sqlx::query("SELECT 1").execute(&state.pool).await.is_ok();
    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "service": "registry.tsx.dev",
            "version": env!("CARGO_PKG_VERSION"),
            "db": if db_ok { "ok" } else { "error" },
        })),
    )
}
