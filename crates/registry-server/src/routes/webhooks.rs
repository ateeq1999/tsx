use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use crate::{db, models::ApiError, AppState};

#[derive(Deserialize)]
pub struct CreateWebhookBody {
    pub url: String,
    pub secret: Option<String>,
    #[serde(default = "default_events")]
    pub events: Vec<String>,
}

fn default_events() -> Vec<String> {
    vec!["package:publish".to_string()]
}

// ── POST /v1/webhooks ─────────────────────────────────────────────────────────

pub async fn create_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CreateWebhookBody>,
) -> (StatusCode, Json<Value>) {
    let user = match require_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => return e,
    };

    if body.url.is_empty() {
        return err400("url is required");
    }
    if !body.url.starts_with("https://") && !body.url.starts_with("http://") {
        return err400("url must start with http:// or https://");
    }
    if body.events.is_empty() {
        return err400("events must not be empty");
    }

    let valid_events = ["package:publish", "package:yank", "package:delete", "package:*"];
    for ev in &body.events {
        if !valid_events.contains(&ev.as_str()) {
            return err400(format!(
                "Unknown event '{}'. Valid events: {}",
                ev,
                valid_events.join(", ")
            ));
        }
    }

    match db::create_webhook(
        &state.pool,
        &user.user_id,
        &body.url,
        body.secret.as_deref(),
        &body.events,
    )
    .await
    {
        Ok(wh) => (StatusCode::CREATED, Json(serde_json::to_value(wh).unwrap())),
        Err(e) => err500(e.to_string()),
    }
}

// ── GET /v1/webhooks ──────────────────────────────────────────────────────────

pub async fn list_webhooks(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    let user = match require_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => return e,
    };

    match db::list_webhooks(&state.pool, &user.user_id).await {
        Ok(whs) => (StatusCode::OK, Json(serde_json::to_value(whs).unwrap())),
        Err(e) => err500(e.to_string()),
    }
}

// ── DELETE /v1/webhooks/:id ───────────────────────────────────────────────────

pub async fn delete_webhook(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    let user = match require_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => return e,
    };

    match db::delete_webhook(&state.pool, id, &user.user_id).await {
        Ok(true) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))),
        Ok(false) => err404(format!("Webhook {} not found", id)),
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
        .ok_or_else(|| err401("Authorization header required"))?;

    match db::validate_session_token(&state.pool, token).await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => Err(err401("Invalid or expired token")),
        Err(e) => Err(err500(e.to_string())),
    }
}

fn err400(msg: impl Into<String>) -> (StatusCode, Json<Value>) {
    (StatusCode::BAD_REQUEST, Json(serde_json::to_value(ApiError::new(msg)).unwrap()))
}

fn err401(msg: impl Into<String>) -> (StatusCode, Json<Value>) {
    (StatusCode::UNAUTHORIZED, Json(serde_json::to_value(ApiError::new(msg)).unwrap()))
}

fn err404(msg: impl Into<String>) -> (StatusCode, Json<Value>) {
    (StatusCode::NOT_FOUND, Json(serde_json::to_value(ApiError::new(msg)).unwrap()))
}

fn err500(msg: impl Into<String>) -> (StatusCode, Json<Value>) {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::to_value(ApiError::new(msg)).unwrap()))
}
