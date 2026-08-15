use std::path::PathBuf;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;
use crate::stack::StackProfile;

pub fn stack_init(
    lang: Option<String>,
    packages: Option<String>,
    dry_run: bool,
    _verbose: bool,
) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let detected = StackProfile::detect(&cwd);

    let mut profile = StackProfile::default();

    // Prefer explicit lang arg, fall back to detection
    profile.lang = lang.unwrap_or_else(|| {
        if detected.lang.is_empty() {
            "typescript".to_string()
        } else {
            detected.lang.clone()
        }
    });
    profile.runtime = detected.runtime;

    // Prefer explicit package list, fall back to detection
    if let Some(pkgs) = packages {
        for p in pkgs.split(',').map(|s| s.trim()) {
            if !p.is_empty() {
                profile.add_package(p);
            }
        }
    } else {
        profile.packages = detected.packages.clone();
    }

    // Auto-discover and install tsx registry packages for the detected npm deps.
    let auto_installed = discover_and_install(&detected.packages);

    let file_path = StackProfile::stack_file(&cwd);
    let profile_json = serde_json::to_value(&profile).unwrap_or_default();

    if dry_run {
        return ResponseEnvelope::success(
            "stack init",
            serde_json::json!({
                "dry_run": true,
                "would_write": file_path.to_string_lossy(),
                "profile": profile_json,
                "auto_installed": auto_installed
            }),
            0,
        );
    }

    match profile.save(&cwd) {
        Ok(_) => {
            let mut next_steps = vec![
                "Run `tsx stack show` to verify the profile".to_string(),
                "Run `tsx list` to see all available commands".to_string(),
                "Run `tsx context` to get your agent onboarding prompt".to_string(),
            ];
            if !auto_installed.is_empty() {
                next_steps.insert(
                    0,
                    format!("Auto-installed tsx packages: {}", auto_installed.join(", ")),
                );
            }
            ResponseEnvelope::success(
                "stack init",
                serde_json::json!({
                    "written": file_path.to_string_lossy(),
                    "profile": profile_json,
                    "auto_installed": auto_installed
                }),
                0,
            )
            .with_next_steps(next_steps)
        }
        Err(e) => ResponseEnvelope::error(
            "stack init",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}

/// Query the registry discovery endpoint for the given npm package names,
/// then install each matched tsx package into the global cache.
/// Returns the ids of successfully installed packages.
/// Failures are silently ignored so stack init works offline.
fn discover_and_install(npm_packages: &[String]) -> Vec<String> {
    if npm_packages.is_empty() {
        return vec![];
    }

    let registry = std::env::var("TSX_REGISTRY_URL")
        .unwrap_or_else(|_| "https://registry.tsx.dev".to_string());

    // Build comma-separated npm list for the discovery query.
    // Strip version suffixes (e.g. @tanstack/start@1.2 → @tanstack/start).
    let npm_csv: String = npm_packages
        .iter()
        .map(|p| npm_base(p))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(",");

    let url = format!("{}/v1/discovery?npm={}", registry, npm_csv);

    // Synchronous HTTP call via ureq — no async needed in CLI.
    let body = match ureq::get(&url).call() {
        Ok(resp) => {
            let mut s = String::new();
            use std::io::Read;
            let _ = resp.into_reader().read_to_string(&mut s);
            s
        }
        Err(_) => return vec![], // Offline or registry unreachable — silently skip
    };

    let discovery: tsx_shared::DiscoveryResponse = match serde_json::from_str(&body) {
        Ok(d) => d,
        Err(_) => return vec![],
    };

    let mut installed = Vec::new();
    for m in &discovery.matches {
        // Skip if already installed
        let store = crate::packages::PackageStore::default();
        if store.get(&m.tsx_package).is_some() {
            continue;
        }
        if crate::packages::installer::install(&registry, &m.tsx_package, None).is_ok() {
            installed.push(m.tsx_package.clone());
        }
    }
    installed
}

/// Strip version suffix from an npm package name.
/// `@tanstack/start@1.2` → `@tanstack/start`
/// `drizzle-orm@0.30` → `drizzle-orm`
fn npm_base(pkg: &str) -> &str {
    if let Some(rest) = pkg.strip_prefix('@') {
        // Scoped: find the second `@` (version separator)
        if let Some(at_idx) = rest.find('@') {
            &pkg[..at_idx + 1]
        } else {
            pkg
        }
    } else {
        pkg.split('@').next().unwrap_or(pkg)
    }
}
