use anyhow::{Context, Result};
use sqlx::PgPool;

pub struct AuthUser {
    pub user_id: String,
    pub name: String,
    pub email: String,
}

/// Validate a better-auth session token against the better-auth schema.
/// Returns `None` if the token is missing, expired, or not found.
pub async fn validate_session_token(pool: &PgPool, token: &str) -> Result<Option<AuthUser>> {
    let row = sqlx::query!(
        r#"
        SELECT s.user_id, u.name, u.email
        FROM session s
        JOIN "user" u ON u.id = s.user_id
        WHERE s.token = $1 AND s.expires_at > NOW()
        "#,
        token
    )
    .fetch_optional(pool)
    .await
    .context("Failed to validate session token")?;

    Ok(row.map(|r| AuthUser {
        user_id: r.user_id,
        name: r.name,
        email: r.email,
    }))
}
