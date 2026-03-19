use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::Value;
use std::sync::Arc;

use crate::{models::ApiError, AppState};

/// GET /v1/stats
///
/// Returns flat RegistryStats — matches TypeScript `RegistryStats` interface directly.
#[utoipa::path(
    get, path = "/v1/stats",
    responses(
        (status = 200, description = "Registry-wide aggregate statistics", body = crate::models::RegistryStats),
        (status = 500, description = "Internal server error", body = crate::models::ApiError),
    ),
    tag = "meta"
)]
pub async fn get_stats(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    match crate::db::get_stats(&state.pool).await {
        Ok(stats) => (StatusCode::OK, Json(serde_json::to_value(stats).expect("BUG: serialization of known types cannot fail"))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ApiError::new(e.to_string())).expect("BUG: serialization of known types cannot fail")),
        ),
    }
}
