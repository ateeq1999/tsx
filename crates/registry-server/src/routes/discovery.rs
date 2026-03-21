//! GET /v1/discovery?npm=@tanstack/start,drizzle-orm
//!
//! Given a comma-separated list of npm package names, returns which tsx
//! registry packages provide each one, and which npm names had no match.

use axum::{
    extract::{Query, State},
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{models::DiscoveryMatch, models::DiscoveryResponse, AppState};

#[derive(Deserialize)]
pub struct DiscoveryQuery {
    /// Comma-separated npm package names, e.g. `@tanstack/start,drizzle-orm`
    #[serde(default)]
    pub npm: String,
}

// ── GET /v1/discovery ────────────────────────────────────────────────────────

pub async fn discovery(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DiscoveryQuery>,
) -> Json<DiscoveryResponse> {
    let npm_names: Vec<String> = query
        .npm
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if npm_names.is_empty() {
        return Json(DiscoveryResponse {
            matches: vec![],
            unmatched: vec![],
        });
    }

    let matches = match db_discover(&state.pool, &npm_names).await {
        Ok(m) => m,
        Err(_) => vec![],
    };

    let matched_npm: std::collections::HashSet<&str> =
        matches.iter().map(|m| m.npm.as_str()).collect();

    let unmatched = npm_names
        .iter()
        .filter(|n| !matched_npm.contains(n.as_str()))
        .cloned()
        .collect();

    Json(DiscoveryResponse { matches, unmatched })
}

// ── DB query ─────────────────────────────────────────────────────────────────

/// Find tsx packages whose manifest lists any of the given npm package names
/// in `npm_packages`. Uses the `versions.manifest` JSONB column.
async fn db_discover(
    pool: &sqlx::PgPool,
    npm_names: &[String],
) -> anyhow::Result<Vec<DiscoveryMatch>> {
    // Use the runtime query API (no compile-time DB connection needed).
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT ON (p.slug, npm_pkg.value)
            p.slug                AS tsx_package,
            v.version,
            npm_pkg.value         AS npm
        FROM packages p
        JOIN versions v ON v.package_id = p.id
        CROSS JOIN LATERAL jsonb_array_elements_text(
            COALESCE(v.manifest->'npm_packages', '[]'::jsonb)
        ) AS npm_pkg(value)
        WHERE npm_pkg.value = ANY($1)
          AND v.yanked = FALSE
        ORDER BY p.slug, npm_pkg.value, v.published_at DESC
        "#,
    )
    .bind(npm_names)
    .fetch_all(pool)
    .await?;

    use sqlx::Row;
    let result = rows
        .into_iter()
        .map(|r| DiscoveryMatch {
            tsx_package: r.get("tsx_package"),
            version: r.get("version"),
            npm: r.get("npm"),
        })
        .collect();

    Ok(result)
}
