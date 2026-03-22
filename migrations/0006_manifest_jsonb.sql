-- Migration 0006: add manifest JSONB column to versions
-- Stores the full package manifest so discovery and commands endpoints
-- can query npm_packages and commands arrays without a separate table.

ALTER TABLE versions
    ADD COLUMN IF NOT EXISTS manifest JSONB NOT NULL DEFAULT '{}'::jsonb;

CREATE INDEX IF NOT EXISTS idx_versions_manifest
    ON versions USING GIN (manifest);
