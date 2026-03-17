/**
 * @tsx/api-types
 *
 * TypeScript interfaces that mirror the JSON shapes produced by the Rust
 * registry server (crates/registry-server / crates/shared).
 *
 * Keep in sync with crates/shared/src/lib.rs.
 * Future: auto-generate via `utoipa` (OpenAPI) → `openapi-typescript`.
 */

// ── Package ───────────────────────────────────────────────────────────────────

/** Full package metadata returned by GET /v1/packages/:name */
export interface Package {
  name: string
  /** Latest semver version string */
  version: string
  description: string
  /** Display name of the author */
  author: string
  license: string
  tags: string[]
  /** Minimum required tsx CLI version */
  tsx_min: string
  created_at: string
  updated_at: string
  download_count: number
  lang?: string
  runtime?: string
  provides?: string[]
  integrates_with?: string[]
}

// ── PackageVersion ────────────────────────────────────────────────────────────

/** One entry in a package's version history */
export interface PackageVersion {
  version: string
  published_at: string
  download_count: number
}

// ── Search ────────────────────────────────────────────────────────────────────

/** Paginated search response from GET /v1/search */
export interface SearchResult {
  packages: Package[]
  total: number
  page: number
  per_page: number
}

// ── Stats ─────────────────────────────────────────────────────────────────────

/** Registry-wide aggregate statistics from GET /v1/stats */
export interface RegistryStats {
  total_packages: number
  total_downloads: number
  total_versions: number
  packages_this_week: number
}

// ── Downloads ─────────────────────────────────────────────────────────────────

/** Per-day download count for the trend chart */
export interface DailyDownloads {
  date: string
  downloads: number
}

// ── Admin ─────────────────────────────────────────────────────────────────────

/** Audit log entry from GET /v1/admin/audit-log */
export interface AuditEntry {
  id: number
  action: string
  package_name: string
  version?: string
  author_name?: string
  ip_address?: string
  created_at: string
}

/** Rate limit status per IP from GET /v1/admin/rate-limits */
export interface RateLimitEntry {
  ip: string
  requests: number
  limit: number
  blocked: boolean
  window_secs_remaining: number
}

// ── Error ─────────────────────────────────────────────────────────────────────

/** Standard error response shape on 4xx/5xx */
export interface ApiError {
  error: string
}

// ── Publish ───────────────────────────────────────────────────────────────────

/** Success response from POST /v1/packages/publish */
export interface PublishResult {
  name: string
  version: string
}
