-- Migration 0004: Package starring
--
-- One row per (user, package) pair.  Composite PK prevents duplicate stars.

CREATE TABLE IF NOT EXISTS stars (
    user_id      TEXT   NOT NULL,
    package_name TEXT   NOT NULL,
    starred_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, package_name)
);

CREATE INDEX IF NOT EXISTS idx_stars_package ON stars(package_name);
CREATE INDEX IF NOT EXISTS idx_stars_user    ON stars(user_id);
