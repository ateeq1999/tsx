//! GET /v1/commands?package=<id>
//!
//! Returns a list of all commands (generator specs) across all packages,
//! optionally filtered to a single package.

use axum::{
    extract::{Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct CommandsQuery {
    /// Filter to a specific package id (optional).
    #[serde(default)]
    pub package: Option<String>,
    /// Filter to a specific command id (optional).
    pub id: Option<String>,
}

#[derive(Serialize)]
pub struct CommandEntry {
    pub package: String,
    pub version: String,
    pub id: String,
    pub command: String,
    pub description: String,
    pub token_estimate: Option<i64>,
}

#[derive(Serialize)]
pub struct CommandsResponse {
    pub commands: Vec<CommandEntry>,
    pub total: usize,
}

pub async fn list_commands(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CommandsQuery>,
) -> Json<CommandsResponse> {
    let commands = match db_list_commands(&state.pool, &query).await {
        Ok(c) => c,
        Err(_) => vec![],
    };
    let total = commands.len();
    Json(CommandsResponse { commands, total })
}

async fn db_list_commands(
    pool: &sqlx::PgPool,
    query: &CommandsQuery,
) -> anyhow::Result<Vec<CommandEntry>> {
    // Query manifest JSONB for commands array from latest non-yanked versions.
    let sql = r#"
        SELECT DISTINCT ON (p.slug, cmd->>'id')
            p.slug                  AS package,
            v.version,
            cmd->>'id'              AS id,
            COALESCE(cmd->>'command', cmd->>'id') AS command,
            COALESCE(cmd->>'description', '')     AS description,
            (cmd->>'token_estimate')::bigint       AS token_estimate
        FROM packages p
        JOIN versions v ON v.package_id = p.id
        CROSS JOIN LATERAL jsonb_array_elements(
            COALESCE(v.manifest->'commands', '[]'::jsonb)
        ) AS cmd
        WHERE v.yanked = FALSE
          AND ($1::text IS NULL OR p.slug = $1)
          AND ($2::text IS NULL OR cmd->>'id' = $2 OR cmd->>'command' = $2)
        ORDER BY p.slug, cmd->>'id', v.published_at DESC
    "#;

    let rows = sqlx::query(sql)
        .bind(query.package.as_deref())
        .bind(query.id.as_deref())
        .fetch_all(pool)
        .await?;

    use sqlx::Row;
    let result = rows
        .into_iter()
        .map(|r| CommandEntry {
            package: r.get("package"),
            version: r.get("version"),
            id: r.get::<Option<String>, _>("id").unwrap_or_default(),
            command: r.get::<Option<String>, _>("command").unwrap_or_default(),
            description: r.get::<Option<String>, _>("description").unwrap_or_default(),
            token_estimate: r.get("token_estimate"),
        })
        .collect();

    Ok(result)
}
