/// Registry discovery and community sharing for tsx framework registries.
///
/// `tsx registry search <query>`  — search npm for tsx-framework-* packages
/// `tsx registry install <pkg>`   — install a community registry into .tsx/frameworks/
/// `tsx registry list`            — list installed community registries
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;
use crate::utils::paths::find_project_root;

/// Metadata stored in `.tsx/registries.json` tracking installed community registries.
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct InstalledRegistry {
    pub slug: String,
    pub package: String,
    pub version: String,
    pub source: String,
    pub installed_at: String,
}

fn registries_index_path(root: &Path) -> PathBuf {
    root.join(".tsx").join("registries.json")
}

fn load_registries_index(root: &Path) -> Vec<InstalledRegistry> {
    let path = registries_index_path(root);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_registries_index(root: &Path, registries: &[InstalledRegistry]) -> anyhow::Result<()> {
    let path = registries_index_path(root);
    std::fs::create_dir_all(path.parent().unwrap())?;
    let content = serde_json::to_string_pretty(registries)?;
    std::fs::write(&path, content)?;
    Ok(())
}

fn iso_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    let year = 1970 + days / 365;
    let doy = days % 365 + 1;
    let month = (doy / 30).min(11) + 1;
    let day = doy % 30 + 1;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, h, m, s)
}

/// Search npm for `tsx-framework-*` packages matching a query.
pub fn registry_search(query: String, verbose: bool) -> CommandResult {
    let start = Instant::now();

    // npm registry search API — returns up to 20 results
    let search_text = if query.trim().is_empty() {
        "tsx-framework".to_string()
    } else {
        format!("tsx-framework {}", query)
    };

    let url = format!(
        "https://registry.npmjs.org/-/v1/search?text={}&size=20",
        urlencoding(&search_text)
    );

    let result = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .and_then(|c| c.get(&url).header("Accept", "application/json").send())
        .and_then(|r| r.json::<serde_json::Value>());

    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(json) => {
            let objects = json
                .get("objects")
                .and_then(|o| o.as_array())
                .cloned()
                .unwrap_or_default();

            let packages: Vec<serde_json::Value> = objects
                .iter()
                .filter_map(|o| {
                    let pkg = o.get("package")?;
                    Some(serde_json::json!({
                        "name": pkg.get("name")?.as_str()?,
                        "version": pkg.get("version")?.as_str().unwrap_or("?"),
                        "description": pkg.get("description").and_then(|d| d.as_str()).unwrap_or(""),
                        "publisher": pkg.get("publisher").and_then(|p| p.get("username")).and_then(|u| u.as_str()).unwrap_or(""),
                    }))
                })
                .collect();

            let response = ResponseEnvelope::success(
                "registry:search",
                serde_json::json!({ "query": query, "results": packages }),
                duration_ms,
            );
            if verbose {
                let context = crate::json::response::Context {
                    project_root: std::env::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    tsx_version: env!("CARGO_PKG_VERSION").to_string(),
                };
                response.with_context(context).print();
            } else {
                response.print();
            }
            CommandResult::ok("registry:search", vec![])
        }
        Err(e) => {
            let error = ErrorResponse::new(
                ErrorCode::InternalError,
                format!("npm search failed: {}", e),
            );
            ResponseEnvelope::error("registry:search", error, duration_ms).print();
            CommandResult::err("registry:search", e.to_string())
        }
    }
}

/// Install a community registry from an npm package into `.tsx/frameworks/<slug>/`.
///
/// The package must:
/// 1. Exist on npm
/// 2. Have a `registry.json` in its package root (fetched via the npm `dist.tarball` URL)
///
/// The installed registry is tracked in `.tsx/registries.json`.
pub fn registry_install(package: String, verbose: bool) -> CommandResult {
    let start = Instant::now();

    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(ErrorCode::ProjectNotFound, e.to_string());
            ResponseEnvelope::error("registry:install", error, duration_ms).print();
            return CommandResult::err("registry:install", e.to_string());
        }
    };

    // Fetch package metadata from npm registry
    let npm_url = format!(
        "https://registry.npmjs.org/{}",
        package.replace("/", "%2F")
    );

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return CommandResult::err("registry:install", e.to_string());
        }
    };

    let pkg_meta: serde_json::Value = match client
        .get(&npm_url)
        .header("Accept", "application/json")
        .send()
        .and_then(|r| r.json())
    {
        Ok(v) => v,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(
                ErrorCode::InternalError,
                format!("Failed to fetch package '{}' from npm: {}", package, e),
            );
            ResponseEnvelope::error("registry:install", error, duration_ms).print();
            return CommandResult::err("registry:install", e.to_string());
        }
    };

    let latest = pkg_meta
        .get("dist-tags")
        .and_then(|t| t.get("latest"))
        .and_then(|v| v.as_str())
        .unwrap_or("latest");

    // Try the direct registry.json URL convention: unpkg.com/<pkg>/registry.json
    let registry_url = format!("https://unpkg.com/{}/registry.json", package);

    let registry_json: serde_json::Value = match client
        .get(&registry_url)
        .header("Accept", "application/json")
        .send()
        .and_then(|r| r.json())
    {
        Ok(v) => v,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(
                ErrorCode::InternalError,
                format!(
                    "Package '{}' does not expose a registry.json via unpkg: {}",
                    package, e
                ),
            );
            ResponseEnvelope::error("registry:install", error, duration_ms).print();
            return CommandResult::err("registry:install", e.to_string());
        }
    };

    // Extract slug from registry.json
    let slug = match registry_json.get("slug").and_then(|s| s.as_str()) {
        Some(s) => s.to_string(),
        None => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(
                ErrorCode::ValidationError,
                "registry.json is missing required 'slug' field",
            );
            ResponseEnvelope::error("registry:install", error, duration_ms).print();
            return CommandResult::err("registry:install", "missing slug".to_string());
        }
    };

    // Write registry.json to .tsx/frameworks/<slug>/
    let dest_dir = root.join(".tsx").join("frameworks").join(&slug);
    std::fs::create_dir_all(&dest_dir).ok();
    let dest_file = dest_dir.join("registry.json");

    let registry_content = serde_json::to_string_pretty(&registry_json)
        .unwrap_or_else(|_| "{}".to_string());

    if let Err(e) = std::fs::write(&dest_file, &registry_content) {
        let duration_ms = start.elapsed().as_millis() as u64;
        let error = ErrorResponse::new(
            ErrorCode::PermissionDenied,
            format!("Failed to write registry: {}", e),
        );
        ResponseEnvelope::error("registry:install", error, duration_ms).print();
        return CommandResult::err("registry:install", e.to_string());
    }

    // Update registries index
    let mut index = load_registries_index(&root);
    index.retain(|r| r.slug != slug); // remove old entry if re-installing
    index.push(InstalledRegistry {
        slug: slug.clone(),
        package: package.clone(),
        version: latest.to_string(),
        source: registry_url,
        installed_at: iso_now(),
    });
    let _ = save_registries_index(&root, &index);

    let duration_ms = start.elapsed().as_millis() as u64;
    let response = ResponseEnvelope::success(
        "registry:install",
        serde_json::json!({
            "installed": {
                "slug": slug,
                "package": package,
                "version": latest,
            }
        }),
        duration_ms,
    );

    if verbose {
        let context = crate::json::response::Context {
            project_root: root.to_string_lossy().to_string(),
            tsx_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        response.with_context(context).print();
    } else {
        response.print();
    }

    CommandResult::ok("registry:install", vec![dest_file.to_string_lossy().to_string()])
}

/// List community registries installed in `.tsx/registries.json`.
pub fn registry_list(verbose: bool) -> CommandResult {
    let start = Instant::now();

    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(ErrorCode::ProjectNotFound, e.to_string());
            ResponseEnvelope::error("registry:list", error, duration_ms).print();
            return CommandResult::err("registry:list", e.to_string());
        }
    };

    let registries = load_registries_index(&root);
    let duration_ms = start.elapsed().as_millis() as u64;

    let response = ResponseEnvelope::success(
        "registry:list",
        serde_json::json!({ "registries": registries }),
        duration_ms,
    );

    if verbose {
        let context = crate::json::response::Context {
            project_root: root.to_string_lossy().to_string(),
            tsx_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        response.with_context(context).print();
    } else {
        response.print();
    }

    CommandResult::ok("registry:list", vec![])
}

/// Minimal percent-encoding for URL query parameters.
fn urlencoding(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                vec![c]
            }
            ' ' => vec!['+'],
            c => {
                let mut buf = [0u8; 4];
                let bytes = c.encode_utf8(&mut buf);
                bytes
                    .bytes()
                    .flat_map(|b| {
                        let hex: Vec<char> =
                            format!("%{:02X}", b).chars().collect();
                        hex
                    })
                    .collect()
            }
        })
        .collect()
}
