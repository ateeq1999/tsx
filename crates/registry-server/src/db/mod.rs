pub mod audit;
pub mod auth;
pub mod downloads;
pub mod packages;
pub mod patterns;
pub mod rate_limit;
pub mod stars;
pub mod stats;
pub mod webhooks;

pub use audit::{get_audit_log, insert_audit};
pub use auth::{validate_api_key, validate_session_token, AuthUser};
pub use downloads::{get_download_stats, increment_downloads};
pub use packages::{
    delete_package, get_latest_version, get_package, get_packages_by_author, get_recent,
    get_tarball_path, get_versions, search, set_deprecated, suggest_packages, update_description,
    update_readme, upsert_package, upsert_version, yank_version, UpsertPkg, UpsertVersion,
};
pub use rate_limit::check_rate_limit;
pub use stars::{
    get_star_count, get_starred_package_rows, get_starred_packages, is_starred, star_package,
    unstar_package,
};
pub use stats::get_stats;
pub use webhooks::{
    create_webhook, delete_webhook, get_active_webhooks_for_event, get_webhook, list_webhooks,
    Webhook,
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
