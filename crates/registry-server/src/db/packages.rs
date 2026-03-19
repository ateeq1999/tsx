use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{PgPool, Row};

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
#[allow(dead_code)]
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
    let id: i64 = sqlx::query_scalar(
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
    )
    .bind(&pkg.name)
    .bind(&pkg.slug)
    .bind(&pkg.description)
    .bind(pkg.author_id.as_deref())
    .bind(&pkg.author_name)
    .bind(&pkg.license)
    .bind(&pkg.tsx_min)
    .bind(&pkg.tags)
    .bind(&pkg.lang)
    .bind(&pkg.runtime)
    .bind(&pkg.provides)
    .bind(&pkg.integrates)
    .fetch_one(pool)
    .await
    .context("Failed to upsert package")?;

    Ok(id)
}

pub async fn upsert_version(pool: &PgPool, pkg_id: i64, ver: &UpsertVersion) -> Result<i64> {
    let id: i64 = sqlx::query_scalar(
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
    )
    .bind(pkg_id)
    .bind(&ver.version)
    .bind(&ver.manifest)
    .bind(&ver.checksum)
    .bind(ver.size_bytes)
    .bind(&ver.tarball_path)
    .fetch_one(pool)
    .await
    .context("Failed to upsert version")?;

    sqlx::query("UPDATE packages SET updated_at = NOW() WHERE id = $1")
        .bind(pkg_id)
        .execute(pool)
        .await?;

    Ok(id)
}

pub async fn update_readme(pool: &PgPool, pkg_id: i64, readme: &str) -> Result<()> {
    sqlx::query("UPDATE packages SET readme = $1, updated_at = NOW() WHERE id = $2")
        .bind(readme)
        .bind(pkg_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_description(pool: &PgPool, pkg_id: i64, description: &str) -> Result<()> {
    sqlx::query("UPDATE packages SET description = $1, updated_at = NOW() WHERE id = $2")
        .bind(description)
        .bind(pkg_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn yank_version(pool: &PgPool, pkg_id: i64, version: &str) -> Result<bool> {
    let result = sqlx::query(
        "UPDATE versions SET yanked = TRUE WHERE package_id = $1 AND version = $2",
    )
    .bind(pkg_id)
    .bind(version)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_package(pool: &PgPool, pkg_id: i64) -> Result<()> {
    sqlx::query("DELETE FROM packages WHERE id = $1")
        .bind(pkg_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ── Read queries ──────────────────────────────────────────────────────────────

pub async fn get_package(pool: &PgPool, name: &str) -> Result<Option<PackageRow>> {
    let row = sqlx::query_as::<_, PackageRow>(
        r#"SELECT id, name, slug, description, author_id, author_name, license, tsx_min,
                  tags, lang, runtime, provides, integrates, readme, downloads,
                  published_at, updated_at
           FROM packages
           WHERE name = $1 OR slug = $1"#,
    )
    .bind(name)
    .fetch_optional(pool)
    .await
    .context("Failed to get package")?;
    Ok(row)
}

#[allow(dead_code)]
pub async fn get_package_by_id(pool: &PgPool, id: i64) -> Result<Option<PackageRow>> {
    let row = sqlx::query_as::<_, PackageRow>(
        r#"SELECT id, name, slug, description, author_id, author_name, license, tsx_min,
                  tags, lang, runtime, provides, integrates, readme, downloads,
                  published_at, updated_at
           FROM packages WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn get_versions(pool: &PgPool, pkg_id: i64) -> Result<Vec<VersionRow>> {
    let rows = sqlx::query_as::<_, VersionRow>(
        r#"SELECT id, version, manifest,
                  checksum, size_bytes, tarball_path, download_count, yanked, published_at
           FROM versions
           WHERE package_id = $1
           ORDER BY published_at DESC"#,
    )
    .bind(pkg_id)
    .fetch_all(pool)
    .await
    .context("Failed to get versions")?;

    // Partition into valid-semver and invalid-semver groups.
    // Sort valid ones by semver DESC; sort invalid ones by publish date DESC.
    // Valid versions always appear before invalid ones so a bad version string
    // never floats to the "latest" position.
    let (mut valid, mut invalid): (Vec<_>, Vec<_>) = rows
        .into_iter()
        .partition(|r| semver::Version::parse(&r.version).is_ok());

    valid.sort_by(|a, b| {
        let va = semver::Version::parse(&a.version).unwrap();
        let vb = semver::Version::parse(&b.version).unwrap();
        vb.cmp(&va)
    });
    invalid.sort_by(|a, b| b.published_at.cmp(&a.published_at));

    valid.extend(invalid);
    Ok(valid)
}

/// Returns the highest semver version string for a package, or `None` if it has no versions.
pub async fn get_latest_version(pool: &PgPool, pkg_id: i64) -> Result<Option<String>> {
    let mut rows = get_versions(pool, pkg_id).await?;
    // get_versions sorts by semver DESC; first element is the latest.
    // Filter to only non-yanked versions for the "latest" label.
    rows.retain(|v| !v.yanked);
    Ok(rows.into_iter().next().map(|v| v.version))
}

pub async fn get_tarball_path(pool: &PgPool, pkg_id: i64, version: &str) -> Result<Option<(i64, String)>> {
    let row = sqlx::query(
        "SELECT id, tarball_path FROM versions WHERE package_id = $1 AND version = $2 AND yanked = FALSE",
    )
    .bind(pkg_id)
    .bind(version)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| (r.get::<i64, _>("id"), r.get::<String, _>("tarball_path"))))
}

pub async fn get_recent(pool: &PgPool, limit: i64) -> Result<Vec<(PackageRow, String)>> {
    let pkgs = sqlx::query_as::<_, PackageRow>(
        r#"SELECT id, name, slug, description, author_id, author_name, license, tsx_min,
                  tags, lang, runtime, provides, integrates, readme, downloads,
                  published_at, updated_at
           FROM packages
           ORDER BY updated_at DESC
           LIMIT $1"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::with_capacity(pkgs.len());
    for pkg in pkgs {
        let latest = get_latest_version(pool, pkg.id)
            .await?
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
    let offset = (page - 1) * per_page;
    let lang_arr: Vec<String> = lang.map(|l| vec![l.to_lowercase()]).unwrap_or_default();
    let has_lang = lang.is_some();

    let order_clause = match sort {
        "newest"  => "published_at DESC",
        "updated" => "updated_at DESC",
        "name"    => "name ASC",
        _         => "downloads DESC",
    };

    // Use full-text search when a query is provided, ILIKE fallback otherwise.
    let (total, pkgs) = if query.is_empty() {
        // No query — list all, optionally filtered by lang.
        let total: i64 = if has_lang {
            sqlx::query_scalar("SELECT COUNT(*) FROM packages WHERE lang && $1::TEXT[]")
                .bind(&lang_arr)
                .fetch_one(pool)
                .await?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM packages")
                .fetch_one(pool)
                .await?
        };

        let data_sql = format!(
            r#"SELECT id, name, slug, description, author_id, author_name, license, tsx_min,
                      tags, lang, runtime, provides, integrates, readme, downloads,
                      published_at, updated_at
               FROM packages
               {}
               ORDER BY {} LIMIT $2 OFFSET $3"#,
            if has_lang { "WHERE lang && $1::TEXT[]" } else { "WHERE ($1::TEXT[] IS NULL OR TRUE)" },
            order_clause
        );
        let pkgs: Vec<PackageRow> = sqlx::query_as(&data_sql)
            .bind(&lang_arr)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?;
        (total, pkgs)
    } else {
        // Full-text search via tsvector GIN index.
        let total: i64 = if has_lang {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM packages WHERE search_vector @@ plainto_tsquery('english', $1) AND lang && $2::TEXT[]",
            )
            .bind(query)
            .bind(&lang_arr)
            .fetch_one(pool)
            .await?
        } else {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM packages WHERE search_vector @@ plainto_tsquery('english', $1)",
            )
            .bind(query)
            .fetch_one(pool)
            .await?
        };

        let data_sql = format!(
            r#"SELECT id, name, slug, description, author_id, author_name, license, tsx_min,
                      tags, lang, runtime, provides, integrates, readme, downloads,
                      published_at, updated_at
               FROM packages
               WHERE search_vector @@ plainto_tsquery('english', $1)
               {}
               ORDER BY {} LIMIT $3 OFFSET $4"#,
            if has_lang { "AND lang && $2::TEXT[]" } else { "AND ($2::TEXT[] IS NULL OR TRUE)" },
            order_clause
        );
        let pkgs: Vec<PackageRow> = sqlx::query_as(&data_sql)
            .bind(query)
            .bind(&lang_arr)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?;
        (total, pkgs)
    };

    let mut result = Vec::with_capacity(pkgs.len());
    for pkg in pkgs {
        let latest = get_latest_version(pool, pkg.id)
            .await?
            .unwrap_or_else(|| "unknown".to_string());
        result.push((pkg, latest));
    }
    Ok((result, total))
}
