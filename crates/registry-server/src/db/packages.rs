use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::PgPool;

use crate::models::Package;

// ── Row types ─────────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow, Debug)]
pub struct PackageRow {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub author_id: Option<String>,
    pub author_name: String,
    pub license: String,
    pub tsx_min: String,
    pub tags: Vec<String>,
    pub lang: Vec<String>,
    pub runtime: Vec<String>,
    pub provides: Vec<String>,
    pub integrates: Vec<String>,
    pub readme: Option<String>,
    pub downloads: i64,
    pub published_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct VersionRow {
    pub id: i64,
    pub version: String,
    pub manifest: sqlx::types::JsonValue,
    pub checksum: String,
    pub size_bytes: i64,
    pub tarball_path: String,
    pub download_count: i64,
    pub yanked: bool,
    pub published_at: chrono::DateTime<Utc>,
}

impl PackageRow {
    /// Convert to the API `Package` shape, supplying the latest version string.
    pub fn into_package(self, latest_version: String) -> Package {
        Package {
            name: self.name,
            version: latest_version,
            description: self.description,
            author: self.author_name,
            license: self.license,
            tsx_min: self.tsx_min,
            tags: self.tags,
            created_at: self.published_at.to_rfc3339(),
            updated_at: self.updated_at.to_rfc3339(),
            download_count: self.downloads,
            lang: self.lang.into_iter().next(),
            runtime: self.runtime.into_iter().next(),
            provides: Some(self.provides),
            integrates_with: Some(self.integrates),
        }
    }
}

// ── Input types ───────────────────────────────────────────────────────────────

pub struct UpsertPkg {
    pub name: String,
    pub slug: String,
    pub description: String,
    pub author_id: Option<String>,
    pub author_name: String,
    pub license: String,
    pub tsx_min: String,
    pub tags: Vec<String>,
    pub lang: Vec<String>,
    pub runtime: Vec<String>,
    pub provides: Vec<String>,
    pub integrates: Vec<String>,
}

pub struct UpsertVersion {
    pub version: String,
    pub manifest: serde_json::Value,
    pub checksum: String,
    pub size_bytes: i64,
    pub tarball_path: String,
}

// ── Write queries ─────────────────────────────────────────────────────────────

pub async fn upsert_package(pool: &PgPool, pkg: &UpsertPkg) -> Result<i64> {
    let row = sqlx::query!(
        r#"
        INSERT INTO packages (name, slug, description, author_id, author_name, license, tsx_min,
                              tags, lang, runtime, provides, integrates, published_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())
        ON CONFLICT(name) DO UPDATE SET
            description  = EXCLUDED.description,
            author_id    = COALESCE(EXCLUDED.author_id, packages.author_id),
            author_name  = CASE WHEN EXCLUDED.author_name = '' THEN packages.author_name ELSE EXCLUDED.author_name END,
            license      = EXCLUDED.license,
            tsx_min      = EXCLUDED.tsx_min,
            tags         = EXCLUDED.tags,
            lang         = EXCLUDED.lang,
            runtime      = EXCLUDED.runtime,
            provides     = EXCLUDED.provides,
            integrates   = EXCLUDED.integrates,
            updated_at   = NOW()
        RETURNING id
        "#,
        pkg.name, pkg.slug, pkg.description,
        pkg.author_id.as_deref(),
        pkg.author_name, pkg.license, pkg.tsx_min,
        &pkg.tags, &pkg.lang, &pkg.runtime, &pkg.provides, &pkg.integrates,
    )
    .fetch_one(pool)
    .await
    .context("Failed to upsert package")?;

    Ok(row.id)
}

pub async fn upsert_version(pool: &PgPool, pkg_id: i64, ver: &UpsertVersion) -> Result<i64> {
    let manifest_json = serde_json::to_value(&ver.manifest)?;
    let row = sqlx::query!(
        r#"
        INSERT INTO versions (package_id, version, manifest, checksum, size_bytes, tarball_path, published_at)
        VALUES ($1, $2, $3, $4, $5, $6, NOW())
        ON CONFLICT(package_id, version) DO UPDATE SET
            manifest     = EXCLUDED.manifest,
            checksum     = EXCLUDED.checksum,
            size_bytes   = EXCLUDED.size_bytes,
            tarball_path = EXCLUDED.tarball_path
        RETURNING id
        "#,
        pkg_id, ver.version, manifest_json,
        ver.checksum, ver.size_bytes, ver.tarball_path,
    )
    .fetch_one(pool)
    .await
    .context("Failed to upsert version")?;

    sqlx::query!("UPDATE packages SET updated_at = NOW() WHERE id = $1", pkg_id)
        .execute(pool)
        .await?;

    Ok(row.id)
}

pub async fn update_readme(pool: &PgPool, pkg_id: i64, readme: &str) -> Result<()> {
    sqlx::query!(
        "UPDATE packages SET readme = $1, updated_at = NOW() WHERE id = $2",
        readme, pkg_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_description(pool: &PgPool, pkg_id: i64, description: &str) -> Result<()> {
    sqlx::query!(
        "UPDATE packages SET description = $1, updated_at = NOW() WHERE id = $2",
        description, pkg_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn yank_version(pool: &PgPool, pkg_id: i64, version: &str) -> Result<bool> {
    let result = sqlx::query!(
        "UPDATE versions SET yanked = TRUE WHERE package_id = $1 AND version = $2",
        pkg_id, version
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_package(pool: &PgPool, pkg_id: i64) -> Result<()> {
    sqlx::query!("DELETE FROM packages WHERE id = $1", pkg_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ── Read queries ──────────────────────────────────────────────────────────────

pub async fn get_package(pool: &PgPool, name: &str) -> Result<Option<PackageRow>> {
    let row = sqlx::query_as!(
        PackageRow,
        r#"SELECT id, name, slug, description, author_id, author_name, license, tsx_min,
                  tags, lang, runtime, provides, integrates, readme, downloads,
                  published_at, updated_at
           FROM packages
           WHERE name = $1 OR slug = $1"#,
        name
    )
    .fetch_optional(pool)
    .await
    .context("Failed to get package")?;
    Ok(row)
}

pub async fn get_package_by_id(pool: &PgPool, id: i64) -> Result<Option<PackageRow>> {
    let row = sqlx::query_as!(
        PackageRow,
        r#"SELECT id, name, slug, description, author_id, author_name, license, tsx_min,
                  tags, lang, runtime, provides, integrates, readme, downloads,
                  published_at, updated_at
           FROM packages WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn get_versions(pool: &PgPool, pkg_id: i64) -> Result<Vec<VersionRow>> {
    let mut rows = sqlx::query_as!(
        VersionRow,
        r#"SELECT id, version, manifest as "manifest: sqlx::types::JsonValue",
                  checksum, size_bytes, tarball_path, download_count, yanked, published_at
           FROM versions
           WHERE package_id = $1
           ORDER BY published_at DESC"#,
        pkg_id
    )
    .fetch_all(pool)
    .await
    .context("Failed to get versions")?;

    // Sort by semver descending; fall back to publish date
    rows.sort_by(|a, b| {
        match (semver::Version::parse(&a.version), semver::Version::parse(&b.version)) {
            (Ok(va), Ok(vb)) => vb.cmp(&va),
            _ => b.published_at.cmp(&a.published_at),
        }
    });
    Ok(rows)
}

pub async fn get_tarball_path(pool: &PgPool, pkg_id: i64, version: &str) -> Result<Option<(i64, String)>> {
    let row = sqlx::query!(
        "SELECT id, tarball_path FROM versions WHERE package_id = $1 AND version = $2 AND yanked = FALSE",
        pkg_id, version
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| (r.id, r.tarball_path)))
}

pub async fn get_recent(pool: &PgPool, limit: i64) -> Result<Vec<(PackageRow, String)>> {
    let pkgs = sqlx::query_as!(
        PackageRow,
        r#"SELECT id, name, slug, description, author_id, author_name, license, tsx_min,
                  tags, lang, runtime, provides, integrates, readme, downloads,
                  published_at, updated_at
           FROM packages
           ORDER BY updated_at DESC
           LIMIT $1"#,
        limit
    )
    .fetch_all(pool)
    .await?;

    let mut result = Vec::with_capacity(pkgs.len());
    for pkg in pkgs {
        let latest = get_versions(pool, pkg.id)
            .await?
            .into_iter()
            .next()
            .map(|v| v.version)
            .unwrap_or_else(|| "unknown".to_string());
        result.push((pkg, latest));
    }
    Ok(result)
}

pub async fn search(
    pool: &PgPool,
    query: &str,
    lang: Option<&str>,
    sort: &str,
    page: i64,
    per_page: i64,
) -> Result<(Vec<(PackageRow, String)>, i64)> {
    let like = format!("%{}%", query.to_lowercase());
    let offset = (page - 1) * per_page;

    let order_clause = match sort {
        "newest"  => "published_at DESC",
        "updated" => "updated_at DESC",
        "name"    => "name ASC",
        _         => "downloads DESC",
    };

    let count_sql = if lang.is_some() {
        "SELECT COUNT(*) FROM packages WHERE (LOWER(name) LIKE $1 OR LOWER(description) LIKE $1 OR $1 = '%%') AND lang && $2::TEXT[]"
    } else {
        "SELECT COUNT(*) FROM packages WHERE (LOWER(name) LIKE $1 OR LOWER(description) LIKE $1 OR $1 = '%%')"
    };

    let total: i64 = if let Some(l) = lang {
        sqlx::query_scalar(count_sql)
            .bind(&like)
            .bind(vec![l.to_lowercase()])
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_scalar(count_sql)
            .bind(&like)
            .fetch_one(pool)
            .await?
    }
    .unwrap_or(0);

    let data_sql = format!(
        r#"SELECT id, name, slug, description, author_id, author_name, license, tsx_min,
                  tags, lang, runtime, provides, integrates, readme, downloads,
                  published_at, updated_at
           FROM packages
           WHERE (LOWER(name) LIKE $1 OR LOWER(description) LIKE $1 OR $1 = '%%')
           {}
           ORDER BY {}
           LIMIT $3 OFFSET $4"#,
        if lang.is_some() { "AND lang && $2::TEXT[]" } else { "AND ($2::TEXT[] IS NULL OR TRUE)" },
        order_clause
    );

    let lang_arr: Vec<String> = lang.map(|l| vec![l.to_lowercase()]).unwrap_or_default();

    let pkgs: Vec<PackageRow> = sqlx::query_as(&data_sql)
        .bind(&like)
        .bind(&lang_arr)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let mut result = Vec::with_capacity(pkgs.len());
    for pkg in pkgs {
        let latest = get_versions(pool, pkg.id)
            .await?
            .into_iter()
            .next()
            .map(|v| v.version)
            .unwrap_or_else(|| "unknown".to_string());
        result.push((pkg, latest));
    }
    Ok((result, total))
}
