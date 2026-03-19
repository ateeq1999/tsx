use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::Deserialize;
use serde_json::Value;
use sqlx::Row;
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
/// Returns live rate-limit state from the PostgreSQL `rate_limits` table.
/// Requires admin API key.
pub async fn get_rate_limits(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    if let Err(e) = require_admin_key(&state, &headers) {
        return e;
    }

    let limit = crate::routes::packages::RATE_LIMIT_MAX_REQUESTS as i64;
    let window_secs = crate::routes::packages::RATE_LIMIT_WINDOW_SECS as i64;

    let rows = match sqlx::query(
        r#"SELECT ip, request_count,
                  GREATEST(0, $1 - CEIL(EXTRACT(EPOCH FROM (NOW() - window_start)))::BIGINT) AS secs_remaining
           FROM rate_limits
           WHERE window_start >= to_timestamp(floor(extract(epoch from now()) / $1) * $1)
           ORDER BY request_count DESC
           LIMIT 200"#,
    )
    .bind(window_secs)
    .fetch_all(&state.pool)
    .await
    {
        Ok(r) => r,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ApiError::new(e.to_string())).expect("BUG: serialization of known types cannot fail")),
        ),
    };

    let entries: Vec<crate::models::RateLimitEntry> = rows
        .iter()
        .map(|r| {
            let count = r.get::<i32, _>("request_count") as u32;
            let remaining = r.get::<i64, _>("secs_remaining").max(0) as u64;
            crate::models::RateLimitEntry {
                ip: r.get::<String, _>("ip"),
                requests: count,
                limit: limit as u32,
                blocked: count as i64 >= limit,
                window_secs_remaining: remaining,
            }
        })
        .collect();

    (StatusCode::OK, Json(serde_json::to_value(entries).expect("BUG: serialization of known types cannot fail")))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn require_admin_key(
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<Value>)> {
    let unauthorized = || (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::to_value(ApiError::new("Unauthorized")).expect("BUG: serialization of known types cannot fail")),
    );

    let expected = state.api_key.as_ref().ok_or_else(unauthorized)?;

    let provided = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    if provided.map(|k| k != expected.as_str()).unwrap_or(true) {
        return Err(unauthorized());
    }
    Ok(())
}
