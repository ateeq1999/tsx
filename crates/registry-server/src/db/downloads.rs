use anyhow::Result;
use sqlx::PgPool;

use crate::models::DailyDownloads;

/// Increment the download counters for a package + version and insert a log entry.
pub async fn increment_downloads(
    pool: &PgPool,
    pkg_id: i64,
    version_id: i64,
    ip: Option<&str>,
    ua: Option<&str>,
) -> Result<()> {
    sqlx::query!("UPDATE packages SET downloads = downloads + 1 WHERE id = $1", pkg_id)
        .execute(pool)
        .await?;
    sqlx::query!(
        "UPDATE versions SET download_count = download_count + 1 WHERE id = $1",
        version_id
    )
    .execute(pool)
    .await?;
    sqlx::query!(
        "INSERT INTO download_logs (package_id, version_id, ip_address, user_agent) VALUES ($1, $2, $3, $4)",
        pkg_id, version_id, ip, ua
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Return per-day download counts for a package over the last `days` days.
pub async fn get_download_stats(pool: &PgPool, pkg_id: i64, days: i64) -> Result<Vec<DailyDownloads>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            DATE(downloaded_at)::TEXT AS "date!",
            COUNT(*)::BIGINT          AS "downloads!"
        FROM download_logs
        WHERE package_id = $1
          AND downloaded_at >= NOW() - ($2::BIGINT || ' days')::INTERVAL
        GROUP BY DATE(downloaded_at)
        ORDER BY DATE(downloaded_at) ASC
        "#,
        pkg_id, days
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| DailyDownloads {
        date: r.date,
        downloads: r.downloads,
    }).collect())
}
