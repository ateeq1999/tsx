use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::Value;
use std::sync::Arc;

use crate::{models::ApiError, AppState};

/// GET /v1/stats
///
/// Returns flat RegistryStats — matches TypeScript `RegistryStats` interface directly.
pub async fn get_stats(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    match crate::db::get_stats(&state.pool).await {
        Ok(stats) => (StatusCode::OK, Json(serde_json::to_value(stats).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ApiError::new(e.to_string())).unwrap()),
        ),
    }
}
