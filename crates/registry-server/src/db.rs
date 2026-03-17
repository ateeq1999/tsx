use anyhow::{Context, Result};
use sqlx::PgPool;
use chrono::Utc;

use crate::models::{Package, PackageVersion, RegistryStats, AuditEntry, DailyDownloads};

// ── Row types for internal use ────────────────────────────────────────────────

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
    /// Convert to the API Package shape, supplying the latest version string.
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

pub struct AuthUser {
    pub user_id: String,
    pub name: String,
    pub email: String,
}

// ── Migration ─────────────────────────────────────────────────────────────────

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS packages (
            id           BIGSERIAL PRIMARY KEY,
            name         TEXT NOT NULL UNIQUE,
            slug         TEXT NOT NULL UNIQUE,
            description  TEXT NOT NULL DEFAULT '',
            author_id    TEXT,
            author_name  TEXT NOT NULL DEFAULT '',
            license      TEXT NOT NULL DEFAULT 'MIT',
            tsx_min      TEXT NOT NULL DEFAULT '0.1.0',
            tags         TEXT[] NOT NULL DEFAULT '{}',
            lang         TEXT[] NOT NULL DEFAULT '{}',
            runtime      TEXT[] NOT NULL DEFAULT '{}',
            provides     TEXT[] NOT NULL DEFAULT '{}',
            integrates   TEXT[] NOT NULL DEFAULT '{}',
            readme       TEXT,
            downloads    BIGINT NOT NULL DEFAULT 0,
            published_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );

        CREATE INDEX IF NOT EXISTS idx_packages_downloads ON packages(downloads DESC);
        CREATE INDEX IF NOT EXISTS idx_packages_updated ON packages(updated_at DESC);

        CREATE TABLE IF NOT EXISTS versions (
            id             BIGSERIAL PRIMARY KEY,
            package_id     BIGINT NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
            version        TEXT NOT NULL,
            manifest       JSONB NOT NULL DEFAULT '{}',
            checksum       TEXT NOT NULL DEFAULT '',
            size_bytes     BIGINT NOT NULL DEFAULT 0,
            tarball_path   TEXT NOT NULL DEFAULT '',
            download_count BIGINT NOT NULL DEFAULT 0,
            yanked         BOOLEAN NOT NULL DEFAULT FALSE,
            published_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(package_id, version)
        );

        CREATE INDEX IF NOT EXISTS idx_versions_package ON versions(package_id);

        CREATE TABLE IF NOT EXISTS download_logs (
            id            BIGSERIAL PRIMARY KEY,
            package_id    BIGINT NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
            version_id    BIGINT REFERENCES versions(id) ON DELETE SET NULL,
            ip_address    TEXT,
            user_agent    TEXT,
            downloaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );

        CREATE INDEX IF NOT EXISTS idx_download_logs_package ON download_logs(package_id);
        CREATE INDEX IF NOT EXISTS idx_download_logs_time   ON download_logs(downloaded_at DESC);

        CREATE TABLE IF NOT EXISTS audit_log (
            id           BIGSERIAL PRIMARY KEY,
            action       TEXT NOT NULL,
            package_name TEXT NOT NULL,
            version      TEXT,
            user_id      TEXT,
            author_name  TEXT,
            ip_address   TEXT,
            detail       JSONB,
            created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );

        CREATE INDEX IF NOT EXISTS idx_audit_log_time ON audit_log(created_at DESC);
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to run registry migrations")?;
    Ok(())
}

// ── Auth ──────────────────────────────────────────────────────────────────────

/// Validate a better-auth session token and return the user identity.
/// Returns None if the token is invalid or expired.
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

// ── Package queries ───────────────────────────────────────────────────────────

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
        pkg.name,
        pkg.slug,
        pkg.description,
        pkg.author_id.as_deref(),
        pkg.author_name,
        pkg.license,
        pkg.tsx_min,
        &pkg.tags,
        &pkg.lang,
        &pkg.runtime,
        &pkg.provides,
        &pkg.integrates,
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
        pkg_id,
        ver.version,
        manifest_json,
        ver.checksum,
        ver.size_bytes,
        ver.tarball_path,
    )
    .fetch_one(pool)
    .await
    .context("Failed to upsert version")?;

    // Update package updated_at
    sqlx::query!("UPDATE packages SET updated_at = NOW() WHERE id = $1", pkg_id)
        .execute(pool)
        .await?;

    Ok(row.id)
}

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

pub async fn increment_downloads(pool: &PgPool, pkg_id: i64, version_id: i64, ip: Option<&str>, ua: Option<&str>) -> Result<()> {
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

pub async fn get_stats(pool: &PgPool) -> Result<RegistryStats> {
    let total_packages: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM packages")
        .fetch_one(pool).await?.unwrap_or(0);
    let total_versions: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM versions")
        .fetch_one(pool).await?.unwrap_or(0);
    let total_downloads: i64 = sqlx::query_scalar!("SELECT COALESCE(SUM(downloads), 0) FROM packages")
        .fetch_one(pool).await?.unwrap_or(0);
    let packages_this_week: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM packages WHERE published_at >= NOW() - INTERVAL '7 days'"
    )
    .fetch_one(pool).await?.unwrap_or(0);

    Ok(RegistryStats { total_packages, total_versions, total_downloads, packages_this_week })
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
        _         => "downloads DESC", // default: most downloaded / relevant
    };

    // Build the query dynamically based on filters
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
    }.unwrap_or(0);

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

// ── Package mutations ─────────────────────────────────────────────────────────

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

// ── Download stats ────────────────────────────────────────────────────────────

pub async fn get_download_stats(pool: &PgPool, pkg_id: i64, days: i64) -> Result<Vec<DailyDownloads>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            DATE(downloaded_at)::TEXT AS "date!",
            COUNT(*)::BIGINT AS "downloads!"
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

// ── Audit log ─────────────────────────────────────────────────────────────────

pub async fn insert_audit(
    pool: &PgPool,
    action: &str,
    package_name: &str,
    version: Option<&str>,
    user_id: Option<&str>,
    author_name: Option<&str>,
    ip: Option<&str>,
    detail: Option<serde_json::Value>,
) -> Result<()> {
    sqlx::query!(
        r#"INSERT INTO audit_log (action, package_name, version, user_id, author_name, ip_address, detail)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        action, package_name, version, user_id, author_name, ip,
        detail as Option<serde_json::Value>
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_audit_log(pool: &PgPool, limit: i64) -> Result<Vec<AuditEntry>> {
    let rows = sqlx::query!(
        r#"SELECT id, action, package_name, version, author_name, ip_address,
                  created_at::TEXT AS "created_at!"
           FROM audit_log
           ORDER BY created_at DESC
           LIMIT $1"#,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| AuditEntry {
        id: r.id,
        action: r.action,
        package_name: r.package_name,
        version: r.version,
        author_name: r.author_name,
        ip_address: r.ip_address,
        created_at: r.created_at,
    }).collect())
}
