-- Migration 0001: Initial registry schema
-- Applied by crates/registry-server at startup via db::run_migrations().
--
-- Uses CREATE TABLE IF NOT EXISTS so re-running is safe (idempotent).

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
CREATE INDEX IF NOT EXISTS idx_packages_updated   ON packages(updated_at DESC);

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
CREATE INDEX IF NOT EXISTS idx_download_logs_time    ON download_logs(downloaded_at DESC);

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
