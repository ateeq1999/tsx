use serde::Serialize;
use std::time::Instant;

use crate::framework::package_cache::PackageCache;
use crate::json::error::ErrorResponse;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;
use crate::utils::paths::get_frameworks_dir;

// ── framework init ────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct FrameworkInitResult {
    path: String,
    files_created: Vec<String>,
}

/// Scaffold a new framework package directory at `<frameworks_dir>/<name>/`.
pub fn framework_init(name: String, verbose: bool) -> CommandResult {
    let start = Instant::now();
    let frameworks_dir = get_frameworks_dir();
    let pkg_dir = frameworks_dir.join(&name);

    if pkg_dir.exists() {
        let duration_ms = start.elapsed().as_millis() as u64;
        let error = ErrorResponse::validation(&format!(
            "Framework '{}' already exists at {}",
            name,
            pkg_dir.display()
        ));
        ResponseEnvelope::error("framework:init", error, duration_ms).print();
        return CommandResult::err("framework:init", "Framework already exists");
    }

    let mut files_created: Vec<String> = vec![];

    let dirs = [
        pkg_dir.join("knowledge"),
        pkg_dir.join("integrations"),
        pkg_dir.join("starters"),
        pkg_dir.join("templates").join("atoms"),
        pkg_dir.join("templates").join("molecules"),
    ];

    for dir in &dirs {
        if let Err(e) = std::fs::create_dir_all(dir) {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(crate::json::error::ErrorCode::InternalError, &format!("Failed to create directory: {}", e));
            ResponseEnvelope::error("framework:init", error, duration_ms).print();
            return CommandResult::err("framework:init", "Failed to create directory");
        }
    }

    // manifest.json
    let manifest = serde_json::json!({
        "id": name,
        "name": name,
        "version": "0.1.0",
        "category": "fullstack",
        "generators": [],
        "starters": ["basic"],
        "integrations": []
    });
    let manifest_path = pkg_dir.join("manifest.json");
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest).unwrap()).ok();
    files_created.push(manifest_path.to_string_lossy().to_string());

    // knowledge/overview.md
    let overview = format!(
        "---\ntitle: {name} Overview\ntoken_estimate: 80\n---\n\n# {name}\n\nDescribe your framework here.\n"
    );
    let overview_path = pkg_dir.join("knowledge").join("overview.md");
    std::fs::write(&overview_path, overview).ok();
    files_created.push(overview_path.to_string_lossy().to_string());

    // starters/basic.json
    let basic_starter = serde_json::json!({
        "id": "basic",
        "name": "Basic Starter",
        "description": "Minimal project",
        "token_estimate": 30,
        "steps": [
            { "cmd": "init", "args": {} }
        ]
    });
    let starter_path = pkg_dir.join("starters").join("basic.json");
    std::fs::write(&starter_path, serde_json::to_string_pretty(&basic_starter).unwrap()).ok();
    files_created.push(starter_path.to_string_lossy().to_string());

    let duration_ms = start.elapsed().as_millis() as u64;

    let result = FrameworkInitResult {
        path: pkg_dir.to_string_lossy().to_string(),
        files_created: files_created.clone(),
    };

    let response = ResponseEnvelope::success(
        "framework:init",
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

    CommandResult::ok("framework:init", files_created)
}

// ── framework validate ────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ValidateResult {
    framework: String,
    path: String,
    valid: bool,
    issues: Vec<String>,
    warnings: Vec<String>,
}

/// Lint a framework package directory: check manifest.json, knowledge files, starter schemas.
pub fn framework_validate(path: Option<String>, verbose: bool) -> CommandResult {
    let start = Instant::now();

    let pkg_dir = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => std::env::current_dir().unwrap_or_default(),
    };

    let mut issues: Vec<String> = vec![];
    let mut warnings: Vec<String> = vec![];

    // 1. Check manifest.json exists and parses
    let manifest_path = pkg_dir.join("manifest.json");
    let framework_id = if manifest_path.exists() {
        match std::fs::read_to_string(&manifest_path) {
            Ok(content) => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(m) => {
                        // Required fields
                        for field in ["id", "name", "version"] {
                            if m.get(field).is_none() {
                                issues.push(format!("manifest.json: missing required field '{}'", field));
                            }
                        }
                        m.get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string()
                    }
                    Err(e) => {
                        issues.push(format!("manifest.json: invalid JSON — {}", e));
                        "unknown".to_string()
                    }
                }
            }
            Err(e) => {
                issues.push(format!("manifest.json: cannot read — {}", e));
                "unknown".to_string()
            }
        }
    } else {
        issues.push("manifest.json not found".to_string());
        "unknown".to_string()
    };

    // 2. Check knowledge directory
    let knowledge_dir = pkg_dir.join("knowledge");
    if knowledge_dir.exists() {
        let sections = ["overview", "concepts", "patterns", "faq", "decisions"];
        let has_any = sections.iter().any(|s| {
            knowledge_dir.join(format!("{}.md", s)).exists()
        });
        if !has_any {
            warnings.push("knowledge/: no standard sections found (overview, concepts, patterns, faq, decisions)".to_string());
        }

        // Check each .md file has frontmatter token_estimate
        if let Ok(entries) = std::fs::read_dir(&knowledge_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().map_or(false, |e| e == "md") {
                    if let Ok(content) = std::fs::read_to_string(&p) {
                        if !content.starts_with("---") {
                            warnings.push(format!(
                                "knowledge/{}: missing frontmatter (expected token_estimate)",
                                p.file_name().unwrap_or_default().to_string_lossy()
                            ));
                        }
                    }
                }
            }
        }
    } else {
        warnings.push("knowledge/ directory not found".to_string());
    }

    // 3. Check starters
    let starters_dir = pkg_dir.join("starters");
    if starters_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&starters_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().map_or(false, |e| e == "json") {
                    match std::fs::read_to_string(&p) {
                        Ok(content) => {
                            if let Err(e) = serde_json::from_str::<serde_json::Value>(&content) {
                                issues.push(format!(
                                    "starters/{}: invalid JSON — {}",
                                    p.file_name().unwrap_or_default().to_string_lossy(),
                                    e
                                ));
                            }
                        }
                        Err(e) => {
                            issues.push(format!(
                                "starters/{}: cannot read — {}",
                                p.file_name().unwrap_or_default().to_string_lossy(),
                                e
                            ));
                        }
                    }
                }
            }
        }
    } else {
        warnings.push("starters/ directory not found".to_string());
    }

    let valid = issues.is_empty();
    let duration_ms = start.elapsed().as_millis() as u64;

    let result = ValidateResult {
        framework: framework_id,
        path: pkg_dir.to_string_lossy().to_string(),
        valid,
        issues,
        warnings,
    };

    let response = ResponseEnvelope::success(
        "framework:validate",
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

    CommandResult::ok("framework:validate", vec![])
}

// ── framework preview ─────────────────────────────────────────────────────────

/// Render a forge template with test data and print to stdout.
pub fn framework_preview(template: String, data: Option<String>, verbose: bool) -> CommandResult {
    let start = Instant::now();

    let template_path = std::path::Path::new(&template);
    if !template_path.exists() {
        let duration_ms = start.elapsed().as_millis() as u64;
        let error = ErrorResponse::validation(&format!(
            "Template not found: {}",
            template
        ));
        ResponseEnvelope::error("framework:preview", error, duration_ms).print();
        return CommandResult::err("framework:preview", "Template not found");
    }

    // Parse context from --data JSON or use empty context
    let ctx_value: serde_json::Value = match data.as_deref() {
        Some(d) => match serde_json::from_str(d) {
            Ok(v) => v,
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let error = ErrorResponse::validation(&format!("Invalid --data JSON: {}", e));
                ResponseEnvelope::error("framework:preview", error, duration_ms).print();
                return CommandResult::err("framework:preview", "Invalid data JSON");
            }
        },
        None => serde_json::json!({}),
    };

    let mut engine = forge::Engine::default();

    // Load the single template file
    let file_name = template_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let content = match std::fs::read_to_string(template_path) {
        Ok(c) => c,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(crate::json::error::ErrorCode::InternalError, &format!("Failed to read template: {}", e));
            ResponseEnvelope::error("framework:preview", error, duration_ms).print();
            return CommandResult::err("framework:preview", "Failed to read template");
        }
    };

    if let Err(e) = engine.add_raw(&file_name, &content) {
        let duration_ms = start.elapsed().as_millis() as u64;
        let error = ErrorResponse::new(crate::json::error::ErrorCode::InternalError, &format!("Failed to load template: {}", e));
        ResponseEnvelope::error("framework:preview", error, duration_ms).print();
        return CommandResult::err("framework:preview", "Failed to load template");
    }

    let forge_ctx = match forge::ForgeContext::from_serialize(&ctx_value) {
        Ok(c) => c,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(crate::json::error::ErrorCode::InternalError, &format!("Context error: {}", e));
            ResponseEnvelope::error("framework:preview", error, duration_ms).print();
            return CommandResult::err("framework:preview", "Context error");
        }
    };

    match engine.render(&file_name, &forge_ctx) {
        Ok(rendered) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let result = serde_json::json!({
                "template": file_name,
                "output": rendered,
                "tier": engine.tier_of(&file_name).to_string(),
            });
            let response = ResponseEnvelope::success("framework:preview", result, duration_ms);
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
            CommandResult::ok("framework:preview", vec![])
        }
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(crate::json::error::ErrorCode::InternalError, &format!("Render error: {}", e));
            ResponseEnvelope::error("framework:preview", error, duration_ms).print();
            CommandResult::err("framework:preview", "Render failed")
        }
    }
}

// ── framework add ─────────────────────────────────────────────────────────────

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

// ── framework list ────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct FrameworkEntry {
    id: String,
    name: String,
    version: String,
    starters: Vec<String>,
    source: Option<String>,
    installed_at: Option<u64>,
    path: String,
}

pub fn framework_list(verbose: bool) -> CommandResult {
    let start = Instant::now();
    let frameworks_dir = get_frameworks_dir();
    let cache = PackageCache::load();
    let mut entries: Vec<FrameworkEntry> = vec![];

    if frameworks_dir.exists() {
        if let Ok(dir_entries) = std::fs::read_dir(&frameworks_dir) {
            for entry in dir_entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let manifest_path = path.join("manifest.json");
                if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                    if let Ok(m) = serde_json::from_str::<serde_json::Value>(&content) {
                        let id = m.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let name = m.get("name").and_then(|v| v.as_str()).unwrap_or(&id).to_string();
                        let version = m.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();
                        let starters = m
                            .get("starters")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|s| s.as_str())
                                    .map(|s| s.to_string())
                                    .collect()
                            })
                            .unwrap_or_default();
                        let cached = cache.get(&id);
                        entries.push(FrameworkEntry {
                            id,
                            name,
                            version,
                            starters,
                            source: cached.map(|c| c.source.clone()),
                            installed_at: cached.map(|c| c.installed_at),
                            path: path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let response = ResponseEnvelope::success(
        "framework:list",
        serde_json::to_value(&entries).unwrap(),
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

    CommandResult::ok("framework:list", vec![])
}

// ── framework publish ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct PublishResult {
    framework: String,
    version: String,
    package_name: String,
    published: bool,
    dry_run: bool,
}

/// Generate a publish-ready `package.json` for the framework directory and run
/// `npm publish` (or validate what would be published in dry-run mode).
pub fn framework_publish(path: Option<String>, dry_run: bool, verbose: bool) -> CommandResult {
    let start = Instant::now();

    let pkg_dir = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => std::env::current_dir().unwrap_or_default(),
    };

    // Read manifest.json
    let manifest_path = pkg_dir.join("manifest.json");
    if !manifest_path.exists() {
        let duration_ms = start.elapsed().as_millis() as u64;
        let error = ErrorResponse::validation("No manifest.json found. Run from a framework package directory.");
        ResponseEnvelope::error("framework:publish", error, duration_ms).print();
        return CommandResult::err("framework:publish", "No manifest.json");
    }

    let manifest_content = match std::fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(
                crate::json::error::ErrorCode::InternalError,
                &format!("Failed to read manifest.json: {}", e),
            );
            ResponseEnvelope::error("framework:publish", error, duration_ms).print();
            return CommandResult::err("framework:publish", "Failed to read manifest.json");
        }
    };

    let manifest: serde_json::Value = match serde_json::from_str(&manifest_content) {
        Ok(m) => m,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::validation(&format!("Invalid manifest.json: {}", e));
            ResponseEnvelope::error("framework:publish", error, duration_ms).print();
            return CommandResult::err("framework:publish", "Invalid manifest.json");
        }
    };

    let fw_id = manifest.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
    let fw_version = manifest.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0");
    let fw_name = manifest.get("name").and_then(|v| v.as_str()).unwrap_or(fw_id);
    let fw_description = manifest.get("description").and_then(|v| v.as_str()).unwrap_or("");

    // The npm package name follows the @tsx-pkg/<id> convention
    let package_name = format!("@tsx-pkg/{}", fw_id);

    // Generate package.json if it doesn't exist
    let npm_pkg_path = pkg_dir.join("package.json");
    if !npm_pkg_path.exists() {
        let npm_pkg = serde_json::json!({
            "name": package_name,
            "version": fw_version,
            "description": fw_description,
            "keywords": ["tsx-framework", fw_id],
            "license": "MIT",
            "files": [
                "manifest.json",
                "knowledge/",
                "integrations/",
                "starters/",
                "generators/",
                "templates/"
            ]
        });
        if !dry_run {
            std::fs::write(
                &npm_pkg_path,
                serde_json::to_string_pretty(&npm_pkg).unwrap(),
            )
            .ok();
        }
    }

    let published = if dry_run {
        false
    } else {
        // Run npm publish
        let output = std::process::Command::new("npm")
            .arg("publish")
            .arg("--access")
            .arg("public")
            .current_dir(&pkg_dir)
            .output();

        match output {
            Ok(o) if o.status.success() => true,
            Ok(o) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let stderr = String::from_utf8_lossy(&o.stderr);
                let error = ErrorResponse::validation(&format!(
                    "npm publish failed: {}",
                    stderr.trim()
                ));
                ResponseEnvelope::error("framework:publish", error, duration_ms).print();
                return CommandResult::err("framework:publish", "npm publish failed");
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let error = ErrorResponse::new(
                    crate::json::error::ErrorCode::InternalError,
                    &format!("Failed to run npm: {}", e),
                );
                ResponseEnvelope::error("framework:publish", error, duration_ms).print();
                return CommandResult::err("framework:publish", "npm not found");
            }
        }
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    let result = PublishResult {
        framework: fw_name.to_string(),
        version: fw_version.to_string(),
        package_name,
        published,
        dry_run,
    };

    let response = ResponseEnvelope::success(
        "framework:publish",
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

    CommandResult::ok("framework:publish", vec![])
}
