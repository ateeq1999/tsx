pub mod auth;
pub mod audit;
pub mod downloads;
pub mod packages;
pub mod rate_limit;
pub mod stats;
pub mod webhooks;

pub use auth::{AuthUser, validate_session_token, validate_api_key};
pub use audit::{insert_audit, get_audit_log};
pub use downloads::{increment_downloads, get_download_stats};
pub use packages::{
    UpsertPkg, UpsertVersion,
    upsert_package, upsert_version,
    get_package,
    get_versions, get_tarball_path,
    get_recent, get_latest_version, search,
    get_packages_by_author,
    update_readme, update_description, yank_version, delete_package,
};
pub use rate_limit::check_rate_limit;
pub use stats::get_stats;
pub use webhooks::{
    Webhook,
    create_webhook, list_webhooks, get_webhook, delete_webhook,
    get_active_webhooks_for_event,
};

use anyhow::{Context, Result};
use sqlx::PgPool;

// ── Schema migrations ─────────────────────────────────────────────────────────
//
// Migration SQL lives in `migrations/` at the workspace root.
// `sqlx::migrate!` embeds the files at compile time — no DATABASE_URL needed
// at build time (unlike `sqlx::query!`).  At runtime it creates the
// `_sqlx_migrations` tracking table and runs each file exactly once, in order.
//
// Transition: 0001–0003 use IF NOT EXISTS guards so the first deploy against an
// existing production database (tables already created by the old inline-DDL
// approach) completes cleanly.

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .context("Failed to run registry migrations")?;
    Ok(())
}
