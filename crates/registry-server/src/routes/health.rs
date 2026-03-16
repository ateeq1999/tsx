use axum::{http::StatusCode, response::Json};
use serde_json::{json, Value};

/// GET /health
pub async fn health() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "service": "registry.tsx.dev",
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
}
