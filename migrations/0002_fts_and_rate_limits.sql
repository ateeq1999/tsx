-- Migration 0002: Full-text search and persistent rate limiting
--
-- Uses ADD COLUMN IF NOT EXISTS / CREATE TABLE IF NOT EXISTS so this migration
-- is safe to run against an existing production database that already has these
-- objects from the previous inline-DDL approach.  On fresh databases these
-- guards are no-ops and the objects are created normally.

-- ── Full-text search ──────────────────────────────────────────────────────────
-- Generated stored column: automatically updated when name/description/tags change.
ALTER TABLE packages
    ADD COLUMN IF NOT EXISTS search_vector tsvector
    GENERATED ALWAYS AS (
        to_tsvector('english',
            name || ' ' ||
            COALESCE(description, '') || ' ' ||
            array_to_string(tags, ' ')
        )
    ) STORED;

CREATE INDEX IF NOT EXISTS idx_packages_fts ON packages USING GIN(search_vector);

-- ── Persistent rate limiting ──────────────────────────────────────────────────
-- Windows are epoch-aligned so state is consistent across restarts and replicas.
CREATE TABLE IF NOT EXISTS rate_limits (
    ip            TEXT        NOT NULL,
    window_start  TIMESTAMPTZ NOT NULL,
    request_count INT         NOT NULL DEFAULT 1,
    PRIMARY KEY (ip, window_start)
);
