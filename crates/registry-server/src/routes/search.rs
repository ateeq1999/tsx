use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use crate::{
    models::{ApiError, ApiResponse, SearchResult},
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    /// Free-text query (name, description, provides)
    #[serde(default)]
    pub q: String,
    /// Filter by language (typescript, python, rust, go)
    pub lang: Option<String>,
    /// Max results (default 20, max 50)
    pub size: Option<usize>,
}

/// GET /v1/search?q=drizzle&lang=typescript
pub async fn search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> (StatusCode, Json<Value>) {
    let query = params.q.trim();
    let size = params.size.unwrap_or(20).min(50);

    let db = state.db.lock().unwrap();
    match db.search_with_latest(query, params.lang.as_deref()) {
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ApiError::new(e.to_string())).unwrap()),
        ),
        Ok(rows) => {
            let results: Vec<SearchResult> = rows
                .into_iter()
                .take(size)
                .map(|(r, latest_version)| SearchResult {
                    install: format!("tsx registry install {}", r.name),
                    latest_version,
                    provides: r.provides_vec(),
                    downloads: r.downloads,
                    lang: r.lang_vec(),
                    description: r.description,
                    name: r.name,
                })
                .collect();

            let count = results.len();
            (
                StatusCode::OK,
                Json(
                    serde_json::to_value(ApiResponse::ok(serde_json::json!({
                        "results": results,
                        "total": count,
                        "query": query,
                    })))
                    .unwrap(),
                ),
            )
        }
    }
}
