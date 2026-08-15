use serde::Serialize;
use std::time::Instant;

use crate::framework::package_cache::PackageCache;
use crate::json::error::ErrorResponse;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;
use crate::utils::paths::get_frameworks_dir;

#[derive(Serialize)]
struct FrameworkAddResult {
    source: String,
    installed_to: String,
    files_copied: u32,
}

/// Install a framework package — routes to npm or local copy based on the source string.
/// - `@scope/pkg` or `pkg-name` (no path separators) → npm install
/// - `./path` or `/abs/path` → local directory copy
pub fn framework_add(source: String, verbose: bool) -> CommandResult {
    let is_npm = !source.starts_with('.') && !source.starts_with('/') && !source.contains('\\');
    if is_npm {
        framework_add_npm(source, verbose)
    } else {
        framework_add_local(source, verbose)
    }
}

/// Install a framework package from an npm package (F.1).
/// Runs `npm install --prefix <tempdir> <package>` then copies to frameworks dir.
fn framework_add_npm(package: String, verbose: bool) -> CommandResult {
    let start = Instant::now();

    // Create a temp directory for the npm install
    let temp_dir = std::env::temp_dir().join(format!("tsx-fw-{}", std::process::id()));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        let duration_ms = start.elapsed().as_millis() as u64;
        let error = ErrorResponse::new(
            crate::json::error::ErrorCode::InternalError,
            &format!("Failed to create temp dir: {}", e),
        );
        ResponseEnvelope::error("framework:add", error, duration_ms).print();
        return CommandResult::err("framework:add", "Failed to create temp dir");
    }

    // Run: npm install --prefix <temp_dir> <package>
    let install_result = std::process::Command::new("npm")
        .args(["install", "--prefix", &temp_dir.to_string_lossy(), &package])
        .output();

    match install_result {
        Ok(o) if !o.status.success() => {
            let _ = std::fs::remove_dir_all(&temp_dir);
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::validation(&format!(
                "npm install failed: {}",
                String::from_utf8_lossy(&o.stderr).trim()
            ));
            ResponseEnvelope::error("framework:add", error, duration_ms).print();
            return CommandResult::err("framework:add", "npm install failed");
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&temp_dir);
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(
                crate::json::error::ErrorCode::InternalError,
                &format!("Failed to run npm: {}", e),
            );
            ResponseEnvelope::error("framework:add", error, duration_ms).print();
            return CommandResult::err("framework:add", "npm not found");
        }
        _ => {}
    }

    // Locate the package in node_modules
    // npm may strip the scope prefix for scoped packages in the dir name
    let pkg_dir_name = package.trim_start_matches('@')
        .replace('/', "__");
    let node_modules = temp_dir.join("node_modules");

    // Try both @scope/name and flat name
    let candidate_paths: Vec<std::path::PathBuf> = vec![
        node_modules.join(&package),
        node_modules.join(&pkg_dir_name),
        // scoped: @scope/name → node_modules/@scope/name
        {
            if package.starts_with('@') {
                let parts: Vec<&str> = package.splitn(2, '/').collect();
                if parts.len() == 2 {
                    node_modules.join(parts[0]).join(parts[1])
                } else {
                    node_modules.join(&package)
                }
            } else {
                node_modules.join(&package)
            }
        },
    ];

    let pkg_path = candidate_paths.into_iter().find(|p| p.exists());

    let pkg_path = match pkg_path {
        Some(p) => p,
        None => {
            let _ = std::fs::remove_dir_all(&temp_dir);
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(
                crate::json::error::ErrorCode::InternalError,
                &format!("Package installed but directory not found in node_modules: {}", package),
            );
            ResponseEnvelope::error("framework:add", error, duration_ms).print();
            return CommandResult::err("framework:add", "Package directory not found");
        }
    };

    // Now copy from pkg_path into frameworks dir (same as local add)
    let result = framework_add_local_path(&pkg_path, &package, "npm", verbose, start);
    let _ = std::fs::remove_dir_all(&temp_dir);
    result
}

/// Install a framework package extracted from a GitHub download (used by `tsx create github:...`).
pub fn framework_add_github(source: String, verbose: bool) -> CommandResult {
    let start = Instant::now();
    let source_path = std::path::PathBuf::from(&source);
    framework_add_local_path(&source_path, &source, "github", verbose, start)
}

/// Install a framework package from a local directory path.
pub fn framework_add_local(source: String, verbose: bool) -> CommandResult {
    let start = Instant::now();
    let source_path = std::path::PathBuf::from(&source);
    framework_add_local_path(&source_path, &source, "local", verbose, start)
}

fn framework_add_local_path(
    source_path: &std::path::Path,
    source_label: &str,
    source_kind: &str,
    verbose: bool,
    start: std::time::Instant,
) -> CommandResult {
    if !source_path.exists() || !source_path.is_dir() {
        let duration_ms = start.elapsed().as_millis() as u64;
        let error = ErrorResponse::validation(&format!("Source path not found: {}", source_label));
        ResponseEnvelope::error("framework:add", error, duration_ms).print();
        return CommandResult::err("framework:add", "Source not found");
    }

    // Read manifest to get the framework id
    let manifest_path = source_path.join("manifest.json");
    let framework_id = if manifest_path.exists() {
        match std::fs::read_to_string(&manifest_path) {
            Ok(content) => {
                let m: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                m.get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| {
                        source_path.file_name().unwrap_or_default().to_str().unwrap_or("unknown")
                    })
                    .to_string()
            }
            Err(_) => source_path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or("unknown")
                .to_string(),
        }
    } else {
        source_path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("unknown")
            .to_string()
    };

    let frameworks_dir = get_frameworks_dir();
    let dest = frameworks_dir.join(&framework_id);

    if let Err(e) = std::fs::create_dir_all(&dest) {
        let duration_ms = start.elapsed().as_millis() as u64;
        let error = ErrorResponse::new(crate::json::error::ErrorCode::InternalError, &format!("Failed to create destination: {}", e));
        ResponseEnvelope::error("framework:add", error, duration_ms).print();
        return CommandResult::err("framework:add", "Failed to create destination");
    }

    let files_copied = copy_dir_recursive(source_path, &dest);

    // Record the install in the package cache
    let fw_version = {
        let m_path = dest.join("manifest.json");
        std::fs::read_to_string(&m_path)
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .and_then(|m| m.get("version").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| "0.0.0".to_string())
    };
    {
        let mut cache = PackageCache::load();
        cache.record(&framework_id, &fw_version, source_kind);
        let _ = cache.save();
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let result = FrameworkAddResult {
        source: source_label.to_string(),
        installed_to: dest.to_string_lossy().to_string(),
        files_copied,
    };

    let response = ResponseEnvelope::success(
        "framework:add",
        serde_json::to_value(result).unwrap(),
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

    CommandResult::ok("framework:add", vec![])
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> u32 {
    let mut count = 0u32;
    let Ok(entries) = std::fs::read_dir(src) else { return 0; };
    for entry in entries.flatten() {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            let _ = std::fs::create_dir_all(&dst_path);
            count += copy_dir_recursive(&src_path, &dst_path);
        } else {
            if std::fs::copy(&src_path, &dst_path).is_ok() {
                count += 1;
            }
        }
    }
    count
}
