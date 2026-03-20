use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde_json::Value;
use std::sync::Arc;

use crate::{db, models::ApiError, AppState};

// ── POST /v1/packages/:name/star ──────────────────────────────────────────────

#[utoipa::path(
    post, path = "/v1/packages/{name}/star",
    params(("name" = String, Path, description = "Package name")),
    responses(
        (status = 200, description = "Package starred"),
        (status = 401, description = "Unauthorized", body = crate::models::ApiError),
        (status = 404, description = "Package not found", body = crate::models::ApiError),
    ),
    security(("bearer_auth" = [])),
    tag = "packages"
)]
pub async fn star_package(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    let user = match require_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => return e,
    };

    let decoded = crate::routes::packages::url_decode_pub(&name);
    if db::get_package(&state.pool, &decoded).await.ok().flatten().is_none() {
        return err404(format!("Package '{}' not found", decoded));
    }

    match db::star_package(&state.pool, &user.user_id, &decoded).await {
        Ok(_) => {
            let count = db::get_star_count(&state.pool, &decoded).await.unwrap_or(0);
            (StatusCode::OK, Json(serde_json::json!({ "ok": true, "star_count": count })))
        }
        Err(e) => err500(e.to_string()),
    }
}

// ── DELETE /v1/packages/:name/star ────────────────────────────────────────────

#[utoipa::path(
    delete, path = "/v1/packages/{name}/star",
    params(("name" = String, Path, description = "Package name")),
    responses(
        (status = 200, description = "Package unstarred"),
        (status = 401, description = "Unauthorized", body = crate::models::ApiError),
    ),
    security(("bearer_auth" = [])),
    tag = "packages"
)]
pub async fn unstar_package(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    let user = match require_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => return e,
    };

    let decoded = crate::routes::packages::url_decode_pub(&name);
    match db::unstar_package(&state.pool, &user.user_id, &decoded).await {
        Ok(_) => {
            let count = db::get_star_count(&state.pool, &decoded).await.unwrap_or(0);
            (StatusCode::OK, Json(serde_json::json!({ "ok": true, "star_count": count })))
        }
        Err(e) => err500(e.to_string()),
    }
}

// ── GET /v1/packages/:name/star ───────────────────────────────────────────────
// Returns star count + whether the authenticated user has starred it.

#[utoipa::path(
    get, path = "/v1/packages/{name}/star",
    params(("name" = String, Path, description = "Package name")),
    responses(
        (status = 200, description = "Star status and count"),
        (status = 404, description = "Package not found", body = crate::models::ApiError),
    ),
    security(("bearer_auth" = [])),
    tag = "packages"
)]
pub async fn get_star_status(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    let decoded = crate::routes::packages::url_decode_pub(&name);
    if db::get_package(&state.pool, &decoded).await.ok().flatten().is_none() {
        return err404(format!("Package '{}' not found", decoded));
    }

    let count = db::get_star_count(&state.pool, &decoded).await.unwrap_or(0);

    let starred_val = match require_user(&state, &headers).await {
        Ok(user) => db::is_starred(&state.pool, &user.user_id, &decoded).await.ok(),
        Err(_) => None,
    };

    (StatusCode::OK, Json(serde_json::json!({
        "star_count": count,
        "starred": starred_val,
    })))
}

// ── GET /v1/account/starred ───────────────────────────────────────────────────

#[utoipa::path(
    get, path = "/v1/account/starred",
    responses(
        (status = 200, description = "Packages starred by the authenticated user", body = Vec<crate::models::Package>),
        (status = 401, description = "Unauthorized", body = crate::models::ApiError),
    ),
    security(("bearer_auth" = [])),
    tag = "account"
)]
pub async fn list_starred(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    let user = match require_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => return e,
    };

    match db::get_starred_package_rows(&state.pool, &user.user_id).await {
        Ok(rows) => {
            let packages: Vec<crate::models::Package> = rows
                .into_iter()
                .map(|(pkg, latest)| pkg.into_package(latest))
                .collect();
            (StatusCode::OK, Json(serde_json::to_value(packages).expect("BUG")))
        }
        Err(e) => err500(e.to_string()),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn require_user(
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<db::AuthUser, (StatusCode, Json<Value>)> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(String::from)
        .ok_or_else(|| (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::to_value(ApiError::new("Authorization header required")).unwrap()),
        ))?;

    if let Ok(Some(user)) = db::validate_session_token(&state.pool, &token).await {
        return Ok(user);
    }
    if let Ok(Some(user)) = db::validate_api_key(&state.pool, &token).await {
        return Ok(user);
    }
    Err((
        StatusCode::UNAUTHORIZED,
        Json(serde_json::to_value(ApiError::new("Invalid or expired token")).unwrap()),
    ))
}

fn err404(msg: impl Into<String>) -> (StatusCode, Json<Value>) {
    (StatusCode::NOT_FOUND, Json(serde_json::to_value(ApiError::new(msg.into())).unwrap()))
}

fn err500(msg: impl Into<String>) -> (StatusCode, Json<Value>) {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::to_value(ApiError::new(msg.into())).unwrap()))
}
