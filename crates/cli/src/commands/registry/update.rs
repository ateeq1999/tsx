use std::time::Instant;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;
use crate::utils::paths::find_project_root;

use super::install::{install_fpf_package, install_legacy_package};
use super::types::{load_registries_index, save_registries_index};
use super::utils::iso_now;

/// Check all installed packages for newer versions on npm and reinstall if available.
pub fn registry_update(_verbose: bool) -> CommandResult {
    let start = Instant::now();

    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(ErrorCode::ProjectNotFound, e.to_string());
            ResponseEnvelope::error("registry:update", error, duration_ms).print();
            return CommandResult::err("registry:update", e.to_string());
        }
    };

    let index = load_registries_index(&root);
    if index.is_empty() {
        let duration_ms = start.elapsed().as_millis() as u64;
        ResponseEnvelope::success(
            "registry:update",
            serde_json::json!({ "message": "No packages installed. Run `tsx registry install <pkg>` first." }),
            duration_ms,
        )
        .print();
        return CommandResult::ok("registry:update", vec![]);
    }

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => return CommandResult::err("registry:update", e.to_string()),
    };

    let mut updated: Vec<serde_json::Value> = vec![];
    let mut already_latest: Vec<serde_json::Value> = vec![];

    for entry in &index {
        let npm_url = format!(
            "https://registry.npmjs.org/{}",
            entry.package.replace('/', "%2F")
        );
        let latest = match client
            .get(&npm_url)
            .header("Accept", "application/json")
            .send()
            .and_then(|r| r.json::<serde_json::Value>())
        {
            Ok(meta) => meta
                .get("dist-tags")
                .and_then(|t| t.get("latest"))
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            Err(_) => {
                already_latest.push(serde_json::json!({
                    "slug": entry.slug,
                    "package": entry.package,
                    "status": "error",
                    "reason": "failed to fetch npm metadata"
                }));
                continue;
            }
        };

        if latest == entry.version || latest == "?" {
            already_latest.push(serde_json::json!({
                "slug": entry.slug,
                "package": entry.package,
                "current": entry.version,
                "status": "up_to_date"
            }));
            continue;
        }

        // Reinstall
        let install_result = if entry.package.starts_with("@tsx-pkg/") {
            install_fpf_package(&entry.package, &latest, &root, &client)
        } else {
            install_legacy_package(&entry.package, &latest, &root, &client)
        };

        match install_result {
            Ok(_) => updated.push(serde_json::json!({
                "slug": entry.slug,
                "package": entry.package,
                "from": entry.version,
                "to": latest,
                "status": "updated"
            })),
            Err(e) => already_latest.push(serde_json::json!({
                "slug": entry.slug,
                "package": entry.package,
                "status": "error",
                "reason": e.to_string()
            })),
        }
    }

    // Persist updated versions
    if !updated.is_empty() {
        let mut new_index = load_registries_index(&root);
        for u in &updated {
            let slug = u["slug"].as_str().unwrap_or("");
            let to = u["to"].as_str().unwrap_or("");
            if let Some(entry) = new_index.iter_mut().find(|e| e.slug == slug) {
                entry.version = to.to_string();
                entry.installed_at = iso_now();
            }
        }
        let _ = save_registries_index(&root, &new_index);
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    ResponseEnvelope::success(
        "registry:update",
        serde_json::json!({
            "updated": updated,
            "already_latest": already_latest,
        }),
        duration_ms,
    )
    .print();

    CommandResult::ok("registry:update", vec![])
}
