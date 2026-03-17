use serde::{Deserialize, Serialize};

// ── Public API types — match TypeScript interfaces in apps/registry-web/src/lib/types.ts ──

/// Full package metadata returned by GET /v1/packages/:name
/// and each element of GET /v1/packages (recent list).
/// Field names and types match the TypeScript `Package` interface exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    /// Latest semver version string
    pub version: String,
    pub description: String,
    /// Display name of the author
    pub author: String,
    pub license: String,
    pub tags: Vec<String>,
    /// Minimum required tsx CLI version
    pub tsx_min: String,
    pub created_at: String,
    pub updated_at: String,
    pub download_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provides: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrates_with: Option<Vec<String>>,
}

/// One entry in a package's version history.
/// Matches the TypeScript `PackageVersion` interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageVersion {
    pub version: String,
    pub published_at: String,
    pub download_count: i64,
}

/// Paginated search response.
/// Matches the TypeScript `SearchResult` interface.
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub packages: Vec<Package>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

/// Registry-wide aggregate statistics.
/// Matches the TypeScript `RegistryStats` interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_packages: i64,
    pub total_downloads: i64,
    pub total_versions: i64,
    pub packages_this_week: i64,
}

/// Per-day download count for the trend chart.
#[derive(Debug, Clone, Serialize)]
pub struct DailyDownloads {
    pub date: String,
    pub downloads: i64,
}

/// Audit log entry for GET /v1/admin/audit-log
#[derive(Debug, Clone, Serialize)]
pub struct AuditEntry {
    pub id: i64,
    pub action: String,
    pub package_name: String,
    pub version: Option<String>,
    pub author_name: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: String,
}

/// Rate limit status per IP for GET /v1/admin/rate-limits
#[derive(Debug, Clone, Serialize)]
pub struct RateLimitEntry {
    pub ip: String,
    pub requests: u32,
    pub limit: u32,
    pub blocked: bool,
    pub window_secs_remaining: u64,
}

/// Error response shape
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
}

impl ApiError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { error: msg.into() }
    }
}
