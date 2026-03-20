use anyhow::Result;
use sqlx::PgPool;

pub async fn star_package(pool: &PgPool, user_id: &str, package_name: &str) -> Result<()> {
    sqlx::query(
        r#"INSERT INTO stars (user_id, package_name)
           VALUES ($1, $2)
           ON CONFLICT (user_id, package_name) DO NOTHING"#,
    )
    .bind(user_id)
    .bind(package_name)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn unstar_package(pool: &PgPool, user_id: &str, package_name: &str) -> Result<()> {
    sqlx::query("DELETE FROM stars WHERE user_id = $1 AND package_name = $2")
        .bind(user_id)
        .bind(package_name)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_star_count(pool: &PgPool, package_name: &str) -> Result<i64> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM stars WHERE package_name = $1",
    )
    .bind(package_name)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn is_starred(pool: &PgPool, user_id: &str, package_name: &str) -> Result<bool> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM stars WHERE user_id = $1 AND package_name = $2)",
    )
    .bind(user_id)
    .bind(package_name)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// All packages starred by a user, most recently starred first.
pub async fn get_starred_packages(pool: &PgPool, user_id: &str) -> Result<Vec<String>> {
    let rows: Vec<String> = sqlx::query_scalar(
        "SELECT package_name FROM stars WHERE user_id = $1 ORDER BY starred_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Full package rows for all packages starred by a user, most recently starred first.
/// Uses a JOIN to avoid N+1 queries.
pub async fn get_starred_package_rows(
    pool: &PgPool,
    user_id: &str,
) -> Result<Vec<(super::packages::PackageRow, String)>> {
    use super::packages::{PackageRow, get_latest_version};
    let pkgs = sqlx::query_as::<_, PackageRow>(
        r#"SELECT p.id, p.name, p.slug, p.description, p.author_id, p.author_name,
                  p.license, p.tsx_min, p.tags, p.lang, p.runtime, p.provides,
                  p.integrates, p.readme, p.downloads, p.published_at, p.updated_at
           FROM packages p
           JOIN stars s ON s.package_name = p.name
           WHERE s.user_id = $1
           ORDER BY s.starred_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::with_capacity(pkgs.len());
    for pkg in pkgs {
        let latest = get_latest_version(pool, pkg.id)
            .await
            .unwrap_or_default()
            .unwrap_or_default();
        result.push((pkg, latest));
    }
    Ok(result)
}
