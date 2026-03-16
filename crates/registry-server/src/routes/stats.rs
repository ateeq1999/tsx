use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::Value;
use std::sync::Arc;

use crate::{
    models::{ApiError, ApiResponse},
    AppState,
};

/// GET /v1/stats
///
/// Returns aggregate registry statistics: total packages, versions, downloads,
/// and the number of packages published in the last 7 days.
pub async fn get_stats(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    let db = state.db.lock().unwrap();
    match db.get_stats() {
        Ok(stats) => (
            StatusCode::OK,
            Json(serde_json::to_value(ApiResponse::ok(stats)).unwrap()),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ApiError::new(e.to_string())).unwrap()),
        ),
    }
}
