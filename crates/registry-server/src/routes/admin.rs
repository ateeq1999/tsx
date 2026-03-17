use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use crate::{models::ApiError, AppState};

#[derive(Debug, Deserialize)]
pub struct LimitQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
}
fn default_limit() -> i64 { 100 }

/// GET /v1/admin/audit-log?limit=N
///
/// Requires admin API key (`Authorization: Bearer <TSX_REGISTRY_API_KEY>`).
pub async fn get_audit_log(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<LimitQuery>,
) -> (StatusCode, Json<Value>) {
    if let Err(e) = require_admin_key(&state, &headers) {
        return e;
    }
    match crate::db::get_audit_log(&state.pool, params.limit).await {
        Ok(entries) => (StatusCode::OK, Json(serde_json::to_value(entries).expect("BUG: serialization of known types cannot fail"))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ApiError::new(e.to_string())).expect("BUG: serialization of known types cannot fail")),
        ),
    }
}

/// GET /v1/admin/rate-limits
///
/// Returns live rate-limit state: current window request counts per IP.
/// Requires admin API key.
pub async fn get_rate_limits(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    if let Err(e) = require_admin_key(&state, &headers) {
        return e;
    }

    let now = std::time::Instant::now();
    let window = std::time::Duration::from_secs(crate::routes::packages::RATE_LIMIT_WINDOW_SECS);
    let limit = crate::routes::packages::RATE_LIMIT_MAX_REQUESTS;

    let entries: Vec<crate::models::RateLimitEntry> = {
        let limiter = state.rate_limiter.lock().unwrap();
        limiter
            .iter()
            .map(|(ip, (count, start))| {
                let elapsed = now.duration_since(*start);
                let remaining = if elapsed < window {
                    window.saturating_sub(elapsed).as_secs()
                } else {
                    0
                };
                crate::models::RateLimitEntry {
                    ip: ip.to_string(),
                    requests: *count,
                    limit,
                    blocked: *count >= limit && elapsed < window,
                    window_secs_remaining: remaining,
                }
            })
            .collect()
    };

    (StatusCode::OK, Json(serde_json::to_value(entries).expect("BUG: serialization of known types cannot fail")))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn require_admin_key(
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<Value>)> {
    let expected = state.api_key.as_ref().ok_or_else(|| (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::to_value(ApiError::new("Admin access is not configured on this server")).expect("BUG: serialization of known types cannot fail")),
    ))?;

    let provided = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    if provided.map(|k| k != expected.as_str()).unwrap_or(true) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::to_value(ApiError::new("Admin API key required")).expect("BUG: serialization of known types cannot fail")),
        ));
    }
    Ok(())
}
