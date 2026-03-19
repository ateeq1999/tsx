use anyhow::Result;
use sqlx::PgPool;

/// Check and increment the rate limit for an IP address.
///
/// Returns `true` if the request is **allowed**, `false` if the rate limit
/// is exceeded. Windows are aligned to Unix-epoch boundaries
/// (e.g., for 60 s: 00:00–01:00, 01:00–02:00, …) so state is consistent
/// across restarts and multiple instances.
///
/// Uses PostgreSQL `INSERT … ON CONFLICT DO UPDATE` for an atomic
/// check-and-increment with no Mutex required.
pub async fn check_rate_limit(
    pool: &PgPool,
    ip: &str,
    window_secs: i64,
    max_requests: i64,
) -> Result<bool> {
    let count: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO rate_limits (ip, window_start, request_count)
        VALUES (
            $1,
            to_timestamp(floor(extract(epoch from now()) / $2) * $2),
            1
        )
        ON CONFLICT (ip, window_start) DO UPDATE
            SET request_count = rate_limits.request_count + 1
        RETURNING request_count::BIGINT
        "#,
    )
    .bind(ip)
    .bind(window_secs)
    .fetch_one(pool)
    .await?;

    // Prune rows older than one window — best-effort, ignore errors.
    let _ = sqlx::query(
        "DELETE FROM rate_limits WHERE window_start < to_timestamp(floor(extract(epoch from now()) / $1) * $1)"
    )
    .bind(window_secs)
    .execute(pool)
    .await;

    Ok(count <= max_requests)
}
