use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use crate::{models::ApiError, AppState};

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    /// Free-text query (searches name, description, provides)
    #[serde(default)]
    pub q: String,
    /// Filter by language (typescript, python, rust, go)
    pub lang: Option<String>,
    /// Sort: downloads (default) | newest | updated | name
    #[serde(default = "default_sort")]
    pub sort: String,
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: i64,
    /// Results per page (default 20, max 50)
    pub size: Option<i64>,
}

fn default_sort() -> String { "downloads".to_string() }
fn default_page() -> i64 { 1 }

/// GET /v1/search?q=&lang=&sort=&page=&size=
///
/// Returns `SearchResult { packages: Package[], total, page, per_page }`.
/// Matches TypeScript `SearchResult` interface exactly.
pub async fn search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> (StatusCode, Json<Value>) {
    let query = params.q.trim();
    let per_page = params.size.unwrap_or(20).min(50).max(1);
    let page = params.page.max(1);

    match crate::db::search(
        &state.pool,
        query,
        params.lang.as_deref(),
        &params.sort,
        page,
        per_page,
    )
    .await
    {
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ApiError::new(e.to_string())).unwrap()),
        ),
        Ok((rows, total)) => {
            let packages: Vec<crate::models::Package> = rows
                .into_iter()
                .map(|(pkg, latest)| pkg.into_package(latest))
                .collect();

            let result = crate::models::SearchResult {
                total,
                page,
                per_page,
                packages,
            };
            (StatusCode::OK, Json(serde_json::to_value(result).unwrap()))
        }
    }
}
