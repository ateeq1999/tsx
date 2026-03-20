//! tsx-shared — API types shared between the tsx CLI and the registry server.
//!
//! All types implement [`serde::Serialize`] + [`serde::Deserialize`] so they
//! can cross the HTTP boundary in both directions.
//!
//! Field names use `snake_case` to match the JSON API contract.

use serde::{Deserialize, Serialize};

// ── Package ──────────────────────────────────────────────────────────────────

/// Full package metadata returned by `GET /v1/packages/:name`
/// and each element of `GET /v1/packages` (recent list).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Package {
    pub name: String,
    /// Latest semver version string.
    pub version: String,
    pub description: String,
    /// Display name of the author.
    pub author: String,
    pub license: String,
    pub tags: Vec<String>,
    /// Minimum required tsx CLI version.
    pub tsx_min: String,
    pub created_at: String,
    pub updated_at: String,
    pub download_count: i64,
    #[serde(default)]
    pub star_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provides: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrates_with: Option<Vec<String>>,
}

// ── PackageVersion ────────────────────────────────────────────────────────────

/// One entry in a package's version history.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PackageVersion {
    pub version: String,
    pub published_at: String,
    pub download_count: i64,
}

// ── Search ────────────────────────────────────────────────────────────────────

/// Paginated search response from `GET /v1/search`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SearchResult {
    pub packages: Vec<Package>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

// ── Stats ─────────────────────────────────────────────────────────────────────

/// Registry-wide aggregate statistics from `GET /v1/stats`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RegistryStats {
    pub total_packages: i64,
    pub total_downloads: i64,
    pub total_versions: i64,
    pub packages_this_week: i64,
}

// ── Downloads ─────────────────────────────────────────────────────────────────

/// Per-day download count for the trend chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DailyDownloads {
    pub date: String,
    pub downloads: i64,
}

// ── Admin ─────────────────────────────────────────────────────────────────────

/// Audit log entry for `GET /v1/admin/audit-log`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AuditEntry {
    pub id: i64,
    pub action: String,
    pub package_name: String,
    pub version: Option<String>,
    pub author_name: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: String,
}

/// Rate limit status per IP for `GET /v1/admin/rate-limits`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RateLimitEntry {
    pub ip: String,
    pub requests: u32,
    pub limit: u32,
    pub blocked: bool,
    pub window_secs_remaining: u64,
}

// ── Publish ───────────────────────────────────────────────────────────────────

/// Success response from `POST /v1/packages/publish`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PublishResult {
    pub name: String,
    pub version: String,
    pub checksum: String,
    pub tarball_url: String,
}

// ── Error ─────────────────────────────────────────────────────────────────────

/// Standard error response shape returned on 4xx/5xx.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ApiError {
    pub error: String,
}

impl ApiError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { error: msg.into() }
    }
}
