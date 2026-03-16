use serde::{Deserialize, Serialize};

/// Full package metadata returned by GET /v1/packages/:name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMeta {
    pub name: String,
    pub description: String,
    pub latest_version: String,
    pub lang: Vec<String>,
    pub runtime: Vec<String>,
    pub provides: Vec<String>,
    pub integrates_with: Vec<String>,
    pub downloads: u64,
    pub published_at: String,
    pub updated_at: String,
    pub versions: Vec<VersionMeta>,
    pub manifest: Option<serde_json::Value>,
}

/// One entry in a package's version history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMeta {
    pub version: String,
    pub published_at: String,
    pub tarball_url: String,
    pub checksum: String,
    pub size_bytes: u64,
}

/// One result in a search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub description: String,
    pub latest_version: String,
    pub lang: Vec<String>,
    pub provides: Vec<String>,
    pub downloads: u64,
    pub install: String,
}

/// Response envelope for all API endpoints
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub ok: bool,
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { ok: true, data }
    }
}

/// Response for error cases
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub ok: bool,
    pub error: String,
}

impl ApiError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            error: msg.into(),
        }
    }
}

/// Registry-wide aggregate statistics returned by GET /v1/stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_packages: u64,
    pub total_versions: u64,
    pub total_downloads: u64,
    pub packages_this_week: u64,
}

/// Body for POST /v1/packages/publish (multipart form)
#[derive(Debug, Deserialize)]
pub struct PublishRequest {
    /// @tsx-pkg/name
    pub name: String,
    pub version: String,
    /// JSON-encoded manifest
    pub manifest: String,
}
