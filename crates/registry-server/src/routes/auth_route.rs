use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde_json::Value;
use std::sync::Arc;

use crate::{db, models::ApiError, AppState};

/// GET /v1/auth/whoami
///
/// Validates the provided bearer token (session token or API key) and
/// returns the associated user.  Used by `tsx login` to confirm credentials.
#[utoipa::path(
    get, path = "/v1/auth/whoami",
    responses(
        (status = 200, description = "Authenticated user info"),
        (status = 401, description = "Invalid or missing token", body = crate::models::ApiError),
    ),
    security(("bearer_auth" = [])),
    tag = "meta"
)]
pub async fn whoami(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    let token = match headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        Some(t) => t.to_string(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::to_value(ApiError::new("Authorization header required")).unwrap()),
            )
        }
    };

    // Admin key
    if let Some(ref key) = state.api_key {
        if token == *key {
            return (
                StatusCode::OK,
                Json(serde_json::json!({ "admin": true, "registry": "tsx-registry" })),
            );
        }
    }

    // Session token (browser login)
    if let Ok(Some(user)) = db::validate_session_token(&state.pool, &token).await {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "user_id":  user.user_id,
                "username": user.name,
                "email":    user.email,
            })),
        );
    }

    // API key (from /account/api-keys)
    match db::validate_api_key(&state.pool, &token).await {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "user_id":  user.user_id,
                "username": user.name,
                "email":    user.email,
            })),
        ),
        _ => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::to_value(ApiError::new("Invalid or expired token")).unwrap()),
        ),
    }
}
