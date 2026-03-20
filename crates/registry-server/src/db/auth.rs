use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

pub struct AuthUser {
    pub user_id: String,
    pub name: String,
    pub email: String,
}

/// Validate a better-auth API key (from the `api_key` table).
///
/// better-auth stores `SHA-256(raw_key)` in hex as `key_hash`.
/// Returns `None` if the key is disabled, expired, or not found.
pub async fn validate_api_key(pool: &PgPool, raw_key: &str) -> Result<Option<AuthUser>> {
    // SHA-256 hash of the raw key, encoded as lowercase hex
    let key_hash = {
        let mut hasher = Sha256::new();
        hasher.update(raw_key.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    let row = sqlx::query_as::<_, (String, String, String)>(
        r#"
        SELECT ak.user_id, u.name, u.email
        FROM api_key ak
        JOIN "user" u ON u.id = ak.user_id
        WHERE ak.key_hash = $1
          AND ak.enabled = true
          AND (ak.expires_at IS NULL OR ak.expires_at > NOW())
        "#,
    )
    .bind(&key_hash)
    .fetch_optional(pool)
    .await
    .context("Failed to validate API key")?;

    Ok(row.map(|(user_id, name, email)| AuthUser { user_id, name, email }))
}

/// Validate a better-auth session token against the better-auth schema.
/// Returns `None` if the token is missing, expired, or not found.
///
/// Uses the non-macro `sqlx::query` form because the `session` / `user`
/// tables are owned by the better-auth layer in `apps/registry-web` and are
/// not part of this crate's migrations.  Runtime-only validation is correct
/// here; compile-time checks would fail unless the auth tables are pre-created.
pub async fn validate_session_token(pool: &PgPool, token: &str) -> Result<Option<AuthUser>> {
    let row = sqlx::query_as::<_, (String, String, String)>(
        r#"
        SELECT s.user_id, u.name, u.email
        FROM session s
        JOIN "user" u ON u.id = s.user_id
        WHERE s.token = $1 AND s.expires_at > NOW()
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .context("Failed to validate session token")?;

    Ok(row.map(|(user_id, name, email)| AuthUser { user_id, name, email }))
}
