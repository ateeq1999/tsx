//! tsx-shared — API types shared between the tsx CLI and the registry server.
//!
//! All types implement [`serde::Serialize`] + [`serde::Deserialize`] so they
//! can cross the HTTP boundary in both directions.
//!
//! Field names use `snake_case` to match the JSON API contract.

use serde::{Deserialize, Serialize};

// ── PackageManifest ───────────────────────────────────────────────────────────

/// The `manifest.json` file that every tsx registry package must contain.
/// It declares the package's identity, what commands it provides, which npm
/// packages it maps to, and optional style/path preset stacks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Unique slug, e.g. `"tanstack-start"`.
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    /// `"framework"`, `"orm"`, `"auth"`, `"ui"`, `"tool"`
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub docs: String,
    #[serde(default)]
    pub github: Option<String>,
    /// npm package names that should trigger auto-install of this tsx package.
    #[serde(default)]
    pub npm_packages: Vec<String>,
    /// Commands this package provides.
    #[serde(default)]
    pub commands: Vec<CommandEntry>,
    /// Named stack presets (style + path defaults).
    #[serde(default)]
    pub stacks: std::collections::HashMap<String, StackPreset>,
    /// Other tsx package IDs that work alongside this one.
    #[serde(default)]
    pub peer_packages: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Template format: `"jinja"` (default) or `"forge"`.
    #[serde(default = "default_template_format")]
    pub template_format: String,
    /// Minimum tsx CLI version required.
    #[serde(default)]
    pub tsx_min: String,
}

fn default_category() -> String { "framework".to_string() }
fn default_template_format() -> String { "jinja".to_string() }

impl Default for PackageManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version: "0.1.0".to_string(),
            description: String::new(),
            category: default_category(),
            author: String::new(),
            license: "MIT".to_string(),
            docs: String::new(),
            github: None,
            npm_packages: vec![],
            commands: vec![],
            stacks: std::collections::HashMap::new(),
            peer_packages: vec![],
            tags: vec![],
            template_format: default_template_format(),
            tsx_min: String::new(),
        }
    }
}

/// A single command declared in `manifest.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    /// Unique command id, e.g. `"add:schema"`.
    pub id: String,
    pub description: String,
    /// Template file name inside the package's `templates/` directory.
    #[serde(default)]
    pub template: String,
}

/// A named stack preset — style + path defaults bundled into the package.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StackPreset {
    #[serde(default)]
    pub style: std::collections::HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub paths: std::collections::HashMap<String, String>,
}

/// Slim summary of an installed package (returned by `PackageStore::list()`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: String,
    pub npm_packages: Vec<String>,
    pub commands: Vec<String>,
    pub install_path: String,
}

impl PackageSummary {
    pub fn from_manifest(manifest: &PackageManifest, install_path: &std::path::Path) -> Self {
        Self {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            description: manifest.description.clone(),
            category: manifest.category.clone(),
            npm_packages: manifest.npm_packages.clone(),
            commands: manifest.commands.iter().map(|c| c.id.clone()).collect(),
            install_path: install_path.to_string_lossy().to_string(),
        }
    }
}

// ── Discovery ─────────────────────────────────────────────────────────────────

/// Response from `GET /v1/discovery?npm[]=...`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    pub matches: Vec<DiscoveryMatch>,
    pub unmatched: Vec<String>,
}

/// A single npm → tsx package mapping returned by the discovery endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMatch {
    pub npm: String,
    pub tsx_package: String,
    pub version: String,
}

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
