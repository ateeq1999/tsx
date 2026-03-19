pub mod auth;
pub mod audit;
pub mod downloads;
pub mod packages;
pub mod rate_limit;
pub mod stats;
pub mod webhooks;

pub use auth::{AuthUser, validate_session_token};
pub use audit::{insert_audit, get_audit_log};
pub use downloads::{increment_downloads, get_download_stats};
pub use packages::{
    UpsertPkg, UpsertVersion,
    upsert_package, upsert_version,
    get_package,
    get_versions, get_tarball_path,
    get_recent, get_latest_version, search,
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
// The canonical SQL source lives in `migrations/0001_initial_schema.sql` at
// the repo root.  It is reproduced inline here so the binary is self-contained
// and does not require the file to be present at runtime.
// Future: switch to `sqlx::migrate!("../../../migrations")` once sqlx offline
// mode is configured for the CI pipeline.

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    // Execute each DDL statement individually — PostgreSQL's extended query
    // protocol (used by SQLx) does not allow multiple commands per statement.
    let statements = [
        r#"CREATE TABLE IF NOT EXISTS packages (
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
        )"#,
        "CREATE INDEX IF NOT EXISTS idx_packages_downloads ON packages(downloads DESC)",
        "CREATE INDEX IF NOT EXISTS idx_packages_updated   ON packages(updated_at DESC)",
        r#"CREATE TABLE IF NOT EXISTS versions (
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
        )"#,
        "CREATE INDEX IF NOT EXISTS idx_versions_package ON versions(package_id)",
        r#"CREATE TABLE IF NOT EXISTS download_logs (
            id            BIGSERIAL PRIMARY KEY,
            package_id    BIGINT NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
            version_id    BIGINT REFERENCES versions(id) ON DELETE SET NULL,
            ip_address    TEXT,
            user_agent    TEXT,
            downloaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
        "CREATE INDEX IF NOT EXISTS idx_download_logs_package ON download_logs(package_id)",
        "CREATE INDEX IF NOT EXISTS idx_download_logs_time    ON download_logs(downloaded_at DESC)",
        r#"CREATE TABLE IF NOT EXISTS audit_log (
            id           BIGSERIAL PRIMARY KEY,
            action       TEXT NOT NULL,
            package_name TEXT NOT NULL,
            version      TEXT,
            user_id      TEXT,
            author_name  TEXT,
            ip_address   TEXT,
            detail       JSONB,
            created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
        "CREATE INDEX IF NOT EXISTS idx_audit_log_time ON audit_log(created_at DESC)",
        // ── Full-text search (added in v0.2) ──────────────────────────────────
        // Generated stored column — auto-updated when name/description/tags change.
        r#"ALTER TABLE packages
           ADD COLUMN IF NOT EXISTS search_vector tsvector
           GENERATED ALWAYS AS (
               to_tsvector('english',
                   name || ' ' ||
                   COALESCE(description, '') || ' ' ||
                   array_to_string(tags, ' ')
               )
           ) STORED"#,
        "CREATE INDEX IF NOT EXISTS idx_packages_fts ON packages USING GIN(search_vector)",
        // ── Persistent rate limiting (added in v0.2) ──────────────────────────
        r#"CREATE TABLE IF NOT EXISTS rate_limits (
            ip            TEXT NOT NULL,
            window_start  TIMESTAMPTZ NOT NULL,
            request_count INT NOT NULL DEFAULT 1,
            PRIMARY KEY (ip, window_start)
        )"#,
        "DELETE FROM rate_limits WHERE window_start < NOW() - INTERVAL '1 hour'",
        // ── Webhooks (added in v0.3) ──────────────────────────────────────────
        r#"CREATE TABLE IF NOT EXISTS webhooks (
            id         BIGSERIAL PRIMARY KEY,
            owner_id   TEXT NOT NULL,
            url        TEXT NOT NULL,
            secret     TEXT,
            events     TEXT[] NOT NULL DEFAULT '{"package:publish"}',
            active     BOOLEAN NOT NULL DEFAULT TRUE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
        "CREATE INDEX IF NOT EXISTS idx_webhooks_owner ON webhooks(owner_id)",
    ];

    for sql in &statements {
        sqlx::query(sql)
            .execute(pool)
            .await
            .context("Failed to run registry migrations")?;
    }
    Ok(())
}
