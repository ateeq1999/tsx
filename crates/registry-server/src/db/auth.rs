use anyhow::{Context, Result};
use sqlx::PgPool;

pub struct AuthUser {
    pub user_id: String,
    pub name: String,
    pub email: String,
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
