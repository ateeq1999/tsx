use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use crate::{db, models::ApiError, AppState};

#[derive(Deserialize)]
pub struct UserPackagesQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
}
fn default_limit() -> i64 { 50 }

/// GET /v1/users/{name}/packages
///
/// Returns packages published by the given author name (case-insensitive).
/// Used by the public author profile pages in the registry web app.
#[utoipa::path(
    get, path = "/v1/users/{name}/packages",
    params(
        ("name"  = String, Path,  description = "Author display name"),
        ("limit" = Option<i64>, Query, description = "Max results (default 50)"),
    ),
    responses(
        (status = 200, description = "Packages by this author", body = Vec<crate::models::Package>),
        (status = 404, description = "No packages found for this author", body = crate::models::ApiError),
    ),
    tag = "packages"
)]
pub async fn get_user_packages(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<UserPackagesQuery>,
) -> (StatusCode, Json<Value>) {
    let limit = params.limit.min(100).max(1);

    let rows = match db::get_packages_by_author(&state.pool, &name, limit).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::to_value(ApiError::new(e.to_string())).unwrap()),
            )
        }
    };

    if rows.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::to_value(ApiError::new(format!("No packages found for author '{name}'"))).unwrap()),
        );
    }

    let mut packages = Vec::with_capacity(rows.len());
    for pkg in rows {
        let latest = db::get_latest_version(&state.pool, pkg.id)
            .await
            .unwrap_or_default()
            .unwrap_or_default();
        packages.push(pkg.into_package(latest));
    }

    (
        StatusCode::OK,
        Json(serde_json::to_value(packages).unwrap()),
    )
}
