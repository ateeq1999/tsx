use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Webhook {
    pub id: i64,
    pub owner_id: String,
    pub url: String,
    pub secret: Option<String>,
    pub events: Vec<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

pub async fn create_webhook(
    pool: &PgPool,
    owner_id: &str,
    url: &str,
    secret: Option<&str>,
    events: &[String],
) -> Result<Webhook> {
    let row = sqlx::query_as::<_, Webhook>(
        "INSERT INTO webhooks (owner_id, url, secret, events)
         VALUES ($1, $2, $3, $4)
         RETURNING id, owner_id, url, secret, events, active, created_at",
    )
    .bind(owner_id)
    .bind(url)
    .bind(secret)
    .bind(events)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_webhooks(pool: &PgPool, owner_id: &str) -> Result<Vec<Webhook>> {
    let rows = sqlx::query_as::<_, Webhook>(
        "SELECT id, owner_id, url, secret, events, active, created_at
         FROM webhooks WHERE owner_id = $1 ORDER BY created_at DESC",
    )
    .bind(owner_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_webhook(pool: &PgPool, id: i64, owner_id: &str) -> Result<Option<Webhook>> {
    let row = sqlx::query_as::<_, Webhook>(
        "SELECT id, owner_id, url, secret, events, active, created_at
         FROM webhooks WHERE id = $1 AND owner_id = $2",
    )
    .bind(id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn delete_webhook(pool: &PgPool, id: i64, owner_id: &str) -> Result<bool> {
    let result = sqlx::query("DELETE FROM webhooks WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(owner_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Fetch all active webhooks subscribed to a given event, regardless of owner.
pub async fn get_active_webhooks_for_event(pool: &PgPool, event: &str) -> Result<Vec<Webhook>> {
    let rows = sqlx::query_as::<_, Webhook>(
        "SELECT id, owner_id, url, secret, events, active, created_at
         FROM webhooks
         WHERE active = TRUE AND ($1 = ANY(events) OR 'package:*' = ANY(events))",
    )
    .bind(event)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
