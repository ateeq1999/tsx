use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::Path;

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .context("Failed to open SQLite database")?;
        let db = Db { conn };
        db.migrate()?;
        Ok(db)
    }

    /// In-memory database for tests / dry-run
    pub fn memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Db { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch("
            PRAGMA journal_mode=WAL;
            PRAGMA foreign_keys=ON;

            CREATE TABLE IF NOT EXISTS packages (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL UNIQUE,        -- @tsx-pkg/drizzle-pg
                slug        TEXT NOT NULL UNIQUE,        -- drizzle-pg
                description TEXT NOT NULL DEFAULT '',
                lang        TEXT NOT NULL DEFAULT '[]',  -- JSON array
                runtime     TEXT NOT NULL DEFAULT '[]',
                provides    TEXT NOT NULL DEFAULT '[]',
                integrates  TEXT NOT NULL DEFAULT '[]',
                downloads   INTEGER NOT NULL DEFAULT 0,
                published_at TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS versions (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                package_id  INTEGER NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
                version     TEXT NOT NULL,
                manifest    TEXT NOT NULL DEFAULT '{}',
                checksum    TEXT NOT NULL DEFAULT '',
                size_bytes  INTEGER NOT NULL DEFAULT 0,
                tarball_path TEXT NOT NULL DEFAULT '',
                published_at TEXT NOT NULL,
                UNIQUE(package_id, version)
            );

            CREATE INDEX IF NOT EXISTS idx_versions_package ON versions(package_id);
        ")?;
        Ok(())
    }

    pub fn upsert_package(&self, pkg: &UpsertPkg) -> Result<i64> {
        let now = iso_now();
        self.conn.execute(
            "INSERT INTO packages (name, slug, description, lang, runtime, provides, integrates, published_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
             ON CONFLICT(name) DO UPDATE SET
               description = excluded.description,
               lang        = excluded.lang,
               runtime     = excluded.runtime,
               provides    = excluded.provides,
               integrates  = excluded.integrates,
               updated_at  = excluded.updated_at",
            params![
                pkg.name,
                pkg.slug,
                pkg.description,
                serde_json::to_string(&pkg.lang)?,
                serde_json::to_string(&pkg.runtime)?,
                serde_json::to_string(&pkg.provides)?,
                serde_json::to_string(&pkg.integrates)?,
                now,
            ],
        )?;
        let id = self.conn.last_insert_rowid();
        if id == 0 {
            // it was an update — re-fetch id
            let id: i64 = self.conn.query_row(
                "SELECT id FROM packages WHERE name = ?1",
                params![pkg.name],
                |r| r.get(0),
            )?;
            return Ok(id);
        }
        Ok(id)
    }

    pub fn upsert_version(&self, pkg_id: i64, ver: &UpsertVersion) -> Result<()> {
        let now = iso_now();
        self.conn.execute(
            "INSERT INTO versions (package_id, version, manifest, checksum, size_bytes, tarball_path, published_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(package_id, version) DO UPDATE SET
               manifest     = excluded.manifest,
               checksum     = excluded.checksum,
               size_bytes   = excluded.size_bytes,
               tarball_path = excluded.tarball_path",
            params![
                pkg_id,
                ver.version,
                ver.manifest,
                ver.checksum,
                ver.size_bytes,
                ver.tarball_path,
                now,
            ],
        )?;
        // Update package's updated_at
        self.conn.execute(
            "UPDATE packages SET updated_at = ?1 WHERE id = ?2",
            params![now, pkg_id],
        )?;
        Ok(())
    }

    pub fn increment_downloads(&self, slug: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE packages SET downloads = downloads + 1 WHERE slug = ?1",
            params![slug],
        )?;
        Ok(())
    }

    pub fn get_package(&self, name: &str) -> Result<Option<PackageRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, slug, description, lang, runtime, provides, integrates, downloads, published_at, updated_at
             FROM packages WHERE name = ?1 OR slug = ?1"
        )?;
        let mut rows = stmt.query(params![name])?;
        if let Some(row) = rows.next()? {
            Ok(Some(PackageRow {
                id:           row.get(0)?,
                name:         row.get(1)?,
                slug:         row.get(2)?,
                description:  row.get(3)?,
                lang:         row.get(4)?,
                runtime:      row.get(5)?,
                provides:     row.get(6)?,
                integrates:   row.get(7)?,
                downloads:    row.get(8)?,
                published_at: row.get(9)?,
                updated_at:   row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_versions(&self, pkg_id: i64) -> Result<Vec<VersionRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT version, manifest, checksum, size_bytes, tarball_path, published_at
             FROM versions WHERE package_id = ?1 ORDER BY published_at DESC"
        )?;
        let rows = stmt.query_map(params![pkg_id], |r| {
            Ok(VersionRow {
                version:      r.get(0)?,
                manifest:     r.get(1)?,
                checksum:     r.get(2)?,
                size_bytes:   r.get(3)?,
                tarball_path: r.get(4)?,
                published_at: r.get(5)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    pub fn get_tarball_path(&self, pkg_id: i64, version: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT tarball_path FROM versions WHERE package_id = ?1 AND version = ?2"
        )?;
        let mut rows = stmt.query(params![pkg_id, version])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(&self, query: &str, lang: Option<&str>) -> Result<Vec<PackageRow>> {
        let like = format!("%{}%", query.to_lowercase());
        let lang_filter = lang.map(|l| format!("%\"{}\" %", l));

        let sql = "SELECT id, name, slug, description, lang, runtime, provides, integrates, downloads, published_at, updated_at
                   FROM packages
                   WHERE (LOWER(name) LIKE ?1 OR LOWER(description) LIKE ?1 OR LOWER(provides) LIKE ?1)
                   ORDER BY downloads DESC
                   LIMIT 30";

        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params![like], |r| {
            Ok(PackageRow {
                id:           r.get(0)?,
                name:         r.get(1)?,
                slug:         r.get(2)?,
                description:  r.get(3)?,
                lang:         r.get(4)?,
                runtime:      r.get(5)?,
                provides:     r.get(6)?,
                integrates:   r.get(7)?,
                downloads:    r.get(8)?,
                published_at: r.get(9)?,
                updated_at:   r.get(10)?,
            })
        })?;
        let mut results: Vec<PackageRow> = rows
            .collect::<rusqlite::Result<Vec<_>>>()?;
        // Optional lang filter (SQLite JSON search is tricky without extensions)
        if let Some(lf) = lang_filter {
            results.retain(|p| {
                let lang_json = p.lang.to_lowercase();
                lf.trim_matches('%').split('"').any(|tok| lang_json.contains(tok))
            });
        }
        Ok(results)
    }
}

// ── Row types ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct PackageRow {
    pub id:           i64,
    pub name:         String,
    pub slug:         String,
    pub description:  String,
    /// JSON-encoded Vec<String>
    pub lang:         String,
    pub runtime:      String,
    pub provides:     String,
    pub integrates:   String,
    pub downloads:    u64,
    pub published_at: String,
    pub updated_at:   String,
}

impl PackageRow {
    pub fn lang_vec(&self) -> Vec<String> {
        serde_json::from_str(&self.lang).unwrap_or_default()
    }
    pub fn runtime_vec(&self) -> Vec<String> {
        serde_json::from_str(&self.runtime).unwrap_or_default()
    }
    pub fn provides_vec(&self) -> Vec<String> {
        serde_json::from_str(&self.provides).unwrap_or_default()
    }
    pub fn integrates_vec(&self) -> Vec<String> {
        serde_json::from_str(&self.integrates).unwrap_or_default()
    }
}

#[derive(Debug)]
pub struct VersionRow {
    pub version:      String,
    pub manifest:     String,
    pub checksum:     String,
    pub size_bytes:   u64,
    pub tarball_path: String,
    pub published_at: String,
}

// ── Input types ───────────────────────────────────────────────────────────────

pub struct UpsertPkg {
    pub name:        String,
    pub slug:        String,
    pub description: String,
    pub lang:        Vec<String>,
    pub runtime:     Vec<String>,
    pub provides:    Vec<String>,
    pub integrates:  Vec<String>,
}

pub struct UpsertVersion {
    pub version:      String,
    pub manifest:     String,
    pub checksum:     String,
    pub size_bytes:   u64,
    pub tarball_path: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn iso_now() -> String {
    // Simple ISO timestamp without chrono dependency
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    let year = 1970 + days / 365;
    let doy = (days % 365) + 1;
    let month = (doy / 30).min(11) + 1;
    let day = (doy % 30) + 1;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, h, m, s)
}
