# api.todo.md — Registry API Spec, Gaps & Implementation Tracker

Full audit of the registry-server (Rust/Axum) against the registry-web (TanStack Start) client,
the TypeScript types in `apps/registry-web/src/lib/types.ts`, and the better-auth PostgreSQL schema.

---

## 1. Database Migration

### Current state
| Layer | Database | ORM/Driver |
|-------|----------|------------|
| `registry-web` (better-auth) | PostgreSQL `tsx_db` | Drizzle ORM |
| `registry-server` (packages) | SQLite `data/registry.db` | rusqlite |

### Target state
Both services share **one** PostgreSQL database: `postgresql://postgres:@localhost:5432/tsx_db`

| Service | Tables owned |
|---------|-------------|
| `registry-web` | `user`, `session`, `account`, `verification` (better-auth managed) |
| `registry-server` | `packages`, `versions`, `download_logs`, `audit_log` |

### Action items
- [x] Add `sqlx` (postgres, chrono, json) to `crates/registry-server/Cargo.toml`
- [x] Remove `rusqlite` from `Cargo.toml`
- [x] Rewrite `db.rs` to use `sqlx::PgPool`
- [x] `main.rs`: read `DATABASE_URL` env var, create `PgPool`, run migrations at startup
- [x] Add `packages` table with all required fields (see §3)
- [x] Add `versions` table with `download_count` + `yanked` columns
- [x] Add `download_logs` table for per-request analytics
- [x] Add `audit_log` table for admin event tracking
- [x] Add `dotenvy` to load `.env` from `crates/registry-server/` (or parent)
- [x] Add Drizzle schema for `packages`, `versions`, `download_logs`, `audit_log` in `registry-web` so server fns can query them directly

---

## 2. TypeScript ↔ Rust Response Shape Mismatches

All mismatches must be fixed so the frontend `fetchJson<T>()` call returns the correct shape.

### 2a. GET /v1/stats

| Field | Frontend expects (`RegistryStats`) | Rust currently returns |
|-------|------------------------------------|------------------------|
| `total_packages` | `number` | `total_packages: u64` ✅ (wrapped in `{ ok, data }`) |
| `total_downloads` | `number` | `total_downloads: u64` ✅ |
| `total_versions` | `number` | `total_versions: u64` ✅ |
| `packages_this_week` | `number` | `packages_this_week: u64` ✅ |
| Wrapper | **flat** (no wrapper) | `{ ok: true, data: { ... } }` ❌ |

**Fix:** Return flat object, no `ApiResponse` wrapper.

---

### 2b. GET /v1/search

| Field | Frontend expects (`SearchResult`) | Rust currently returns |
|-------|-----------------------------------|------------------------|
| `packages` | `Package[]` | `results: SearchResult[]` ❌ (wrong key, wrong type) |
| `total` | `number` | `total: usize` ✅ (but wrapped) |
| `page` | `number` | missing ❌ |
| `per_page` | `number` | missing ❌ |
| Package shape | see §2c | minimal `SearchResult` struct ❌ |
| Pagination | `?page=N` supported | not implemented ❌ |
| Sort | `?sort=downloads\|newest\|updated\|name` | not implemented ❌ |

**Fix:** Return `{ packages: Package[], total, page, per_page }`, full Package shape, add pagination + sort.

---

### 2c. GET /v1/packages/:name and GET /v1/packages (recent)

| Field | Frontend expects (`Package`) | Rust currently returns (`PackageMeta`) |
|-------|------------------------------|----------------------------------------|
| `version` | `string` (latest) | `latest_version: String` ❌ (wrong key) |
| `author` | `string` | missing ❌ |
| `license` | `string` | missing ❌ |
| `tags` | `string[]` | missing ❌ |
| `tsx_min` | `string` | missing ❌ |
| `created_at` | `string` | `published_at` ❌ (wrong key) |
| `updated_at` | `string` | `updated_at` ✅ |
| `download_count` | `number` | `downloads` ❌ (wrong key) |
| `lang` | `string` (singular) | `lang: Vec<String>` ❌ (wrong type, should be primary lang string) |
| `runtime` | `string` (singular) | `runtime: Vec<String>` ❌ (same issue) |
| `provides` | `string[]` | `provides: Vec<String>` ✅ |
| `integrates_with` | `string[]` | `integrates_with: Vec<String>` ✅ |
| Wrapper | **flat** | `{ ok: true, data: { ... } }` ❌ |
| Nested `versions` | not in Package type | included in PackageMeta ❌ (remove or separate) |

**Fix:** Rename fields, add missing fields, return flat Package shape.

---

### 2d. GET /v1/packages/:name/versions

| Field | Frontend expects (`PackageVersion`) | Rust currently returns (`VersionMeta`) |
|-------|-------------------------------------|----------------------------------------|
| `version` | `string` | `version` ✅ |
| `published_at` | `string` | `published_at` ✅ |
| `download_count` | `number` | missing ❌ (`size_bytes`, `checksum`, `tarball_url` returned instead) |
| Wrapper | **flat array** | `{ ok: true, data: [...] }` ❌ |

**Fix:** Add `download_count` to versions table and response, remove tarball metadata from this endpoint.

---

### 2e. POST /v1/packages/publish

| Field | Frontend sends | Rust expects |
|-------|---------------|-------------|
| `name` | multipart field | ✅ |
| `version` | multipart field | ✅ |
| `manifest` | JSON string | ✅ |
| `tarball` | file bytes | ✅ |
| Auth header | `Authorization: Bearer <token>` | checks static API key only ❌ — should also accept better-auth session tokens |

**Fix:** Support both static API key and better-auth session token validation.

---

## 3. Database Schema — Canonical Definitions

### packages
```sql
CREATE TABLE IF NOT EXISTS packages (
    id           BIGSERIAL PRIMARY KEY,
    name         TEXT NOT NULL UNIQUE,       -- @tsx-pkg/drizzle-pg
    slug         TEXT NOT NULL UNIQUE,       -- drizzle-pg
    description  TEXT NOT NULL DEFAULT '',
    author_id    TEXT REFERENCES "user"(id) ON DELETE SET NULL,
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
CREATE INDEX IF NOT EXISTS idx_packages_updated ON packages(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_packages_name_gin ON packages USING gin(to_tsvector('english', name || ' ' || description));
```

### versions
```sql
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
```

### download_logs
```sql
CREATE TABLE IF NOT EXISTS download_logs (
    id            BIGSERIAL PRIMARY KEY,
    package_id    BIGINT NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    version_id    BIGINT REFERENCES versions(id) ON DELETE SET NULL,
    ip_address    TEXT,
    user_agent    TEXT,
    downloaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_download_logs_package ON download_logs(package_id);
CREATE INDEX IF NOT EXISTS idx_download_logs_time ON download_logs(downloaded_at DESC);
```

### audit_log
```sql
CREATE TABLE IF NOT EXISTS audit_log (
    id           BIGSERIAL PRIMARY KEY,
    action       TEXT NOT NULL,   -- 'publish' | 'yank' | 'delete' | 'update_readme' | 'update_meta'
    package_name TEXT NOT NULL,
    version      TEXT,
    user_id      TEXT REFERENCES "user"(id) ON DELETE SET NULL,
    author_name  TEXT,
    ip_address   TEXT,
    detail       JSONB,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_audit_log_time ON audit_log(created_at DESC);
```

---

## 4. API Endpoint Inventory

### Implemented ✅

| Method | Path | Description | Status |
|--------|------|-------------|--------|
| GET | `/health` | Health check | ✅ works |
| GET | `/v1/stats` | Aggregate stats | ✅ shape fixed |
| GET | `/v1/search` | Search packages | ✅ shape + pagination fixed |
| GET | `/v1/packages` | Recent packages | ✅ shape fixed |
| GET | `/v1/packages/:name` | Package metadata | ✅ shape fixed |
| GET | `/v1/packages/:name/versions` | Version list | ✅ shape fixed |
| GET | `/v1/packages/:name/:version/tarball` | Download tarball | ✅ |
| POST | `/v1/packages/publish` | Publish package | ✅ auth improved |

### New / Missing ❌ → ✅

| Method | Path | Description | Status |
|--------|------|-------------|--------|
| GET | `/v1/packages/:name/readme` | Fetch README markdown | ✅ added |
| PUT | `/v1/packages/:name/readme` | Update README | ✅ added |
| PUT | `/v1/packages/:name` | Update description / metadata | ✅ added |
| DELETE | `/v1/packages/:name/versions/:version` | Yank a version | ✅ added |
| DELETE | `/v1/packages/:name` | Delete package | ✅ added |
| GET | `/v1/packages/:name/stats/downloads` | Per-day download stats | ✅ added |
| GET | `/v1/admin/audit-log` | Publish audit log | ✅ added |
| GET | `/v1/admin/rate-limits` | Rate limit status per IP | ✅ added |

---

## 5. Auth Validation in Registry-Server

### Current
- Static `TSX_REGISTRY_API_KEY` env var — Bearer token checked only on `POST /v1/packages/publish`
- No user identity — packages have no `author_id`

### Target
1. **Static API key** — still supported for CLI publishing without web login
2. **better-auth session token** — validate by querying `session` table in PostgreSQL:
   ```sql
   SELECT s.user_id, u.name, u.email, u.email_verified
   FROM session s
   JOIN "user" u ON u.id = s.user_id
   WHERE s.token = $1 AND s.expires_at > NOW()
   ```
3. **Author binding** — on `publish`, record `author_id` + `author_name` from session
4. **PUT/DELETE authz** — only the package author or an admin-role user may update/delete

- [x] Add `validate_token()` helper in `db.rs`
- [x] Thread user identity through publish handler
- [x] Add authz guards to PUT/DELETE endpoints

---

## 6. TanStack Start Server Functions — Sync Checklist

All server fns in `apps/registry-web/src/` that proxy to the Rust registry server:

| Server fn | File | Calls | Synced? |
|-----------|------|-------|---------|
| Browse packages | `lib/api.ts` `registryApi.search()` | `GET /v1/search` | ✅ after fix |
| Get package | `lib/api.ts` `registryApi.getPackage()` | `GET /v1/packages/:name` | ✅ after fix |
| Get versions | `lib/api.ts` `registryApi.getVersions()` | `GET /v1/packages/:name/versions` | ✅ after fix |
| Get README | `lib/api.ts` `registryApi.getReadme()` | `GET /v1/packages/:name/readme` | ✅ after add |
| Recent packages | `lib/api.ts` `registryApi.getRecent()` | `GET /v1/packages?sort=recent` | ✅ after fix |
| Stats | `lib/api.ts` `registryApi.getStats()` | `GET /v1/stats` | ✅ after fix |
| Publish | `routes/_protected/publish.tsx` | `POST /v1/packages/publish` | ✅ |
| Update README | `routes/_protected/packages/$name.edit.tsx` | `PUT /v1/packages/:name/readme` | ✅ after add |
| Yank version | `routes/_protected/packages/$name.edit.tsx` | `DELETE /v1/packages/:name/versions/:version` | ✅ after add |
| Admin users | `server/admin/queries.ts` `getAdminUsers()` | Drizzle direct (no Rust) | ✅ |
| Admin audit log | `routes/_protected/admin/audit-log.tsx` | `GET /v1/admin/audit-log` | needs wire-up |
| Admin rate limits | `routes/_protected/admin/rate-limits.tsx` | `GET /v1/admin/rate-limits` | needs wire-up |
| Download stats | `routes/packages/$name.tsx` | `GET /v1/packages/:name/stats/downloads` | needs wire-up |

---

## 7. Drizzle Schema for registry-web

The `registry-web` app needs Drizzle-mapped tables for packages/versions so server functions
(admin queries, "my packages" page) can use them without calling the Rust HTTP API.

- [x] `apps/registry-web/src/db/schema/packages.ts` — Drizzle schema for all 4 tables
- [x] `apps/registry-web/src/db/schema/index.ts` — re-export packages schema

---

## 8. Environment Variables

### registry-server (`crates/registry-server/`)
| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | ✅ | PostgreSQL connection string |
| `PORT` | optional (8080) | TCP port |
| `DATA_DIR` | optional (`./data`) | Tarball storage directory |
| `TSX_REGISTRY_API_KEY` | optional | Static Bearer key for publish |

### registry-web (`apps/registry-web/`)
| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | ✅ | Same PostgreSQL `tsx_db` |
| `VITE_REGISTRY_URL` | optional (`http://localhost:8080`) | Rust registry base URL |
| `BETTER_AUTH_SECRET` | ✅ | Cookie signing secret |
| `BETTER_AUTH_URL` | ✅ | Auth base URL |
| `GITHUB_CLIENT_ID` / `GITHUB_CLIENT_SECRET` | optional | OAuth |
| `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET` | optional | OAuth |

---

## 9. Priority Order

| Priority | Item |
|----------|------|
| P0 — blocking | DB migration SQLite → PostgreSQL (§1) |
| P0 — blocking | Fix response shape mismatches (§2) |
| P0 — blocking | Add `packages` + `versions` + `download_logs` + `audit_log` tables (§3) |
| P1 — required | Missing endpoints: readme, PUT, DELETE, yank (§4) |
| P1 — required | Auth: session token validation in Rust (§5) |
| P2 — data quality | Download logs per-day stats endpoint (§4) |
| P2 — data quality | Admin audit-log and rate-limits wired to real data (§6) |
| P3 — DX | Drizzle schema for packages in registry-web (§7) |
