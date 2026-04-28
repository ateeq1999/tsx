-- Migration 0002: Full-text search and persistent rate limiting
--
-- Uses ADD COLUMN IF NOT EXISTS / CREATE TABLE IF NOT EXISTS so this migration
-- is safe to run against an existing production database that already has these
-- objects from the previous inline-DDL approach.  On fresh databases these
-- guards are no-ops and the objects are created normally.

-- ── Full-text search ──────────────────────────────────────────────────
-- Use a regular column with a trigger to update the search vector.
ALTER TABLE packages
    ADD COLUMN IF NOT EXISTS search_vector tsvector;

-- Create a function to update the search vector
CREATE OR REPLACE FUNCTION update_packages_search_vector() RETURNS trigger AS $$
BEGIN
    NEW.search_vector :=
        to_tsvector('english',
            NEW.name || ' ' ||
            COALESCE(NEW.description, '') || ' ' ||
            COALESCE(array_to_string(NEW.tags, ' '), '')
        );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'packages_search_vector_trigger') THEN
        CREATE TRIGGER packages_search_vector_trigger
            BEFORE INSERT OR UPDATE OF name, description, tags
            ON packages
            FOR EACH ROW
            EXECUTE FUNCTION update_packages_search_vector();
    END IF;
END;
$$;

-- Update existing rows
UPDATE packages SET search_vector =
    to_tsvector('english',
        name || ' ' ||
        COALESCE(description, '') || ' ' ||
        COALESCE(array_to_string(tags, ' '), '')
    )
WHERE search_vector IS NULL;

CREATE INDEX IF NOT EXISTS idx_packages_fts ON packages USING GIN(search_vector);

-- ── Persistent rate limiting ──────────────────────────────────────────
-- Windows are epoch-aligned so state is consistent across restarts and replicas.
CREATE TABLE IF NOT EXISTS rate_limits (
    ip            TEXT        NOT NULL,
    window_start  TIMESTAMPTZ NOT NULL,
    request_count INT         NOT NULL DEFAULT 1,
    PRIMARY KEY (ip, window_start)
);
