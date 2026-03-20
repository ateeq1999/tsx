use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Json, Redirect},
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
    /// Filter by tag (exact match against the tags array)
    pub tag: Option<String>,
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

#[derive(Debug, Deserialize)]
pub struct SuggestParams {
    #[serde(default)]
    pub q: String,
}

/// GET /v1/search?q=&lang=&sort=&page=&size=
///
/// Returns `SearchResult { packages: Package[], total, page, per_page }`.
/// Matches TypeScript `SearchResult` interface exactly.
#[utoipa::path(
    get, path = "/v1/search",
    params(
        ("q"    = Option<String>, Query, description = "Free-text search query"),
        ("lang" = Option<String>, Query, description = "Filter by language: typescript | python | rust | go"),
        ("tag"  = Option<String>, Query, description = "Filter by tag (exact match)"),
        ("sort" = Option<String>, Query, description = "Sort order: downloads (default) | newest | updated | name"),
        ("page" = Option<i64>,   Query, description = "Page number (1-based, default 1)"),
        ("size" = Option<i64>,   Query, description = "Results per page (default 20, max 50)"),
    ),
    responses(
        (status = 200, description = "Paginated package search results", body = crate::models::SearchResult),
        (status = 500, description = "Internal server error", body = crate::models::ApiError),
    ),
    tag = "packages"
)]
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
        params.tag.as_deref(),
        &params.sort,
        page,
        per_page,
    )
    .await
    {
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ApiError::new(e.to_string())).expect("BUG: serialization of known types cannot fail")),
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
            (StatusCode::OK, Json(serde_json::to_value(result).expect("BUG: serialization of known types cannot fail")))
        }
    }
}

// ── GET /v1/search/suggest?q= ─────────────────────────────────────────────────

#[utoipa::path(
    get, path = "/v1/search/suggest",
    params(("q" = String, Query, description = "Partial package name (minimum 1 character)")),
    responses(
        (status = 200, description = "Top 5 package name completions as a JSON string array"),
        (status = 500, description = "Internal server error", body = crate::models::ApiError),
    ),
    tag = "packages"
)]
pub async fn suggest(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SuggestParams>,
) -> (StatusCode, Json<Value>) {
    let q = params.q.trim();
    if q.is_empty() {
        return (StatusCode::OK, Json(serde_json::json!([])));
    }
    match crate::db::suggest_packages(&state.pool, q, 5).await {
        Ok(names) => (StatusCode::OK, Json(serde_json::to_value(names).expect("BUG"))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ApiError::new(e.to_string())).expect("BUG")),
        ),
    }
}
