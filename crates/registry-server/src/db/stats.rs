use anyhow::Result;
use sqlx::PgPool;

use crate::models::RegistryStats;

pub async fn get_stats(pool: &PgPool) -> Result<RegistryStats> {
    let total_packages: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM packages")
        .fetch_one(pool).await?;
    let total_versions: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM versions")
        .fetch_one(pool).await?;
    let total_downloads: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(downloads), 0)::BIGINT FROM packages"
    )
    .fetch_one(pool).await?;
    let packages_this_week: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM packages WHERE published_at >= NOW() - INTERVAL '7 days'"
    )
    .fetch_one(pool).await?;

    Ok(RegistryStats { total_packages, total_versions, total_downloads, packages_this_week })
}
