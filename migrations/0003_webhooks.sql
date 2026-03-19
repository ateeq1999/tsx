-- Migration 0003: Webhooks
--
-- Stores per-user webhook subscriptions for registry events.

CREATE TABLE IF NOT EXISTS webhooks (
    id         BIGSERIAL   PRIMARY KEY,
    owner_id   TEXT        NOT NULL,
    url        TEXT        NOT NULL,
    secret     TEXT,
    events     TEXT[]      NOT NULL DEFAULT '{"package:publish"}',
    active     BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_webhooks_owner ON webhooks(owner_id);
