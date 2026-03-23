use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::Value;
use sqlx::{PgPool, Row};

// ── Row types ─────────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct PatternRow {
    pub id: i64,
    pub slug: String,
    pub author_id: Option<String>,
    pub author_name: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub framework: String,
    pub tags: Vec<String>,
    pub tarball_path: String,
    pub checksum: String,
    pub download_count: i64,
    pub readme: Option<String>,
    pub published_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct PatternVersionRow {
    pub id: i64,
    pub pattern_id: i64,
    pub version: String,
    pub tarball_path: String,
    pub checksum: String,
    pub size_bytes: i64,
    pub manifest: sqlx::types::JsonValue,
    pub published_at: chrono::DateTime<Utc>,
}

// ── Upsert types ──────────────────────────────────────────────────────────────

pub struct UpsertPattern {
    pub slug: String,
    pub author_id: Option<String>,
    pub author_name: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub framework: String,
    pub tags: Vec<String>,
    pub tarball_path: String,
    pub checksum: String,
    pub readme: Option<String>,
}

pub struct UpsertPatternVersion {
    pub version: String,
    pub tarball_path: String,
    pub checksum: String,
    pub size_bytes: i64,
    pub manifest: Value,
}

// ── Write ops ─────────────────────────────────────────────────────────────────

pub async fn upsert_pattern(pool: &PgPool, data: UpsertPattern) -> Result<PatternRow> {
    let row = sqlx::query_as::<_, PatternRow>(
        r#"
        INSERT INTO patterns
            (slug, author_id, author_name, name, version, description, framework,
             tags, tarball_path, checksum, readme, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
        ON CONFLICT (slug) DO UPDATE
            SET author_name   = EXCLUDED.author_name,
                name          = EXCLUDED.name,
                version       = EXCLUDED.version,
                description   = EXCLUDED.description,
                framework     = EXCLUDED.framework,
                tags          = EXCLUDED.tags,
                tarball_path  = EXCLUDED.tarball_path,
                checksum      = EXCLUDED.checksum,
                readme        = COALESCE(EXCLUDED.readme, patterns.readme),
                updated_at    = NOW()
        RETURNING *
        "#,
    )
    .bind(&data.slug)
    .bind(&data.author_id)
    .bind(&data.author_name)
    .bind(&data.name)
    .bind(&data.version)
    .bind(&data.description)
    .bind(&data.framework)
    .bind(&data.tags)
    .bind(&data.tarball_path)
    .bind(&data.checksum)
    .bind(&data.readme)
    .fetch_one(pool)
    .await
    .context("upsert_pattern")?;
    Ok(row)
}

pub async fn upsert_pattern_version(
    pool: &PgPool,
    pattern_id: i64,
    data: UpsertPatternVersion,
) -> Result<PatternVersionRow> {
    let row = sqlx::query_as::<_, PatternVersionRow>(
        r#"
        INSERT INTO pattern_versions
            (pattern_id, version, tarball_path, checksum, size_bytes, manifest)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (pattern_id, version) DO UPDATE
            SET tarball_path = EXCLUDED.tarball_path,
                checksum     = EXCLUDED.checksum,
                size_bytes   = EXCLUDED.size_bytes,
                manifest     = EXCLUDED.manifest
        RETURNING *
        "#,
    )
    .bind(pattern_id)
    .bind(&data.version)
    .bind(&data.tarball_path)
    .bind(&data.checksum)
    .bind(data.size_bytes)
    .bind(&data.manifest)
    .fetch_one(pool)
    .await
    .context("upsert_pattern_version")?;
    Ok(row)
}

pub async fn increment_pattern_downloads(pool: &PgPool, pattern_id: i64) {
    let _ = sqlx::query(
        "UPDATE patterns SET download_count = download_count + 1 WHERE id = $1",
    )
    .bind(pattern_id)
    .execute(pool)
    .await;
}

// ── Read ops ──────────────────────────────────────────────────────────────────

pub async fn get_pattern(pool: &PgPool, slug: &str) -> Result<Option<PatternRow>> {
    sqlx::query_as::<_, PatternRow>("SELECT * FROM patterns WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await
        .context("get_pattern")
}

pub async fn get_pattern_version(
    pool: &PgPool,
    pattern_id: i64,
    version: &str,
) -> Result<Option<PatternVersionRow>> {
    sqlx::query_as::<_, PatternVersionRow>(
        "SELECT * FROM pattern_versions WHERE pattern_id = $1 AND version = $2",
    )
    .bind(pattern_id)
    .bind(version)
    .fetch_optional(pool)
    .await
    .context("get_pattern_version")
}

pub async fn list_patterns(
    pool: &PgPool,
    limit: i64,
    framework: Option<&str>,
) -> Result<Vec<PatternRow>> {
    match framework {
        Some(fw) => {
            sqlx::query_as::<_, PatternRow>(
                "SELECT * FROM patterns WHERE framework = $1 ORDER BY updated_at DESC LIMIT $2",
            )
            .bind(fw)
            .bind(limit)
            .fetch_all(pool)
            .await
            .context("list_patterns (framework)")
        }
        None => {
            sqlx::query_as::<_, PatternRow>(
                "SELECT * FROM patterns ORDER BY updated_at DESC LIMIT $1",
            )
            .bind(limit)
            .fetch_all(pool)
            .await
            .context("list_patterns")
        }
    }
}

pub async fn search_patterns(pool: &PgPool, query: &str, limit: i64) -> Result<Vec<PatternRow>> {
    sqlx::query_as::<_, PatternRow>(
        r#"
        SELECT *, ts_rank(
            to_tsvector('english', coalesce(name,'') || ' ' || coalesce(description,'') || ' ' || coalesce(framework,'')),
            plainto_tsquery('english', $1)
        ) AS rank
        FROM patterns
        WHERE to_tsvector('english', coalesce(name,'') || ' ' || coalesce(description,'') || ' ' || coalesce(framework,''))
              @@ plainto_tsquery('english', $1)
           OR name ILIKE '%' || $1 || '%'
        ORDER BY rank DESC, download_count DESC
        LIMIT $2
        "#,
    )
    .bind(query)
    .bind(limit)
    .fetch_all(pool)
    .await
    .context("search_patterns")
}

pub async fn list_pattern_versions(
    pool: &PgPool,
    pattern_id: i64,
) -> Result<Vec<PatternVersionRow>> {
    sqlx::query_as::<_, PatternVersionRow>(
        "SELECT * FROM pattern_versions WHERE pattern_id = $1 ORDER BY published_at DESC",
    )
    .bind(pattern_id)
    .fetch_all(pool)
    .await
    .context("list_pattern_versions")
}
