-- Migration 0007: pattern packs registry
-- Stores user-uploaded code generation packs (pack.json + tarball).

CREATE TABLE IF NOT EXISTS patterns (
    id              BIGSERIAL PRIMARY KEY,
    slug            TEXT        NOT NULL UNIQUE,   -- e.g. "ateeg/todo-with-auth"
    author_id       TEXT,                           -- auth user id (nullable = anonymous)
    author_name     TEXT        NOT NULL DEFAULT '',
    name            TEXT        NOT NULL,
    version         TEXT        NOT NULL DEFAULT '1.0.0',
    description     TEXT        NOT NULL DEFAULT '',
    framework       TEXT        NOT NULL DEFAULT '',
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    tarball_path    TEXT        NOT NULL DEFAULT '',
    checksum        TEXT        NOT NULL DEFAULT '',
    download_count  BIGINT      NOT NULL DEFAULT 0,
    readme          TEXT,
    published_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS pattern_versions (
    id              BIGSERIAL PRIMARY KEY,
    pattern_id      BIGINT      NOT NULL REFERENCES patterns(id) ON DELETE CASCADE,
    version         TEXT        NOT NULL,
    tarball_path    TEXT        NOT NULL,
    checksum        TEXT        NOT NULL DEFAULT '',
    size_bytes      BIGINT      NOT NULL DEFAULT 0,
    manifest        JSONB       NOT NULL DEFAULT '{}'::jsonb,
    published_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (pattern_id, version)
);

-- Full-text search index on patterns
CREATE INDEX IF NOT EXISTS idx_patterns_fts
    ON patterns USING GIN (
        to_tsvector('english', coalesce(name,'') || ' ' || coalesce(description,'') || ' ' || coalesce(framework,''))
    );

-- Fast framework filter
CREATE INDEX IF NOT EXISTS idx_patterns_framework ON patterns (framework);

-- Tags GIN index
CREATE INDEX IF NOT EXISTS idx_patterns_tags ON patterns USING GIN (tags);

-- Manifest GIN on pattern_versions
CREATE INDEX IF NOT EXISTS idx_pattern_versions_manifest
    ON pattern_versions USING GIN (manifest);
