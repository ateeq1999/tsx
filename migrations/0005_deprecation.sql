-- Migration 0005: package deprecation
ALTER TABLE packages
    ADD COLUMN IF NOT EXISTS deprecated_message TEXT;
