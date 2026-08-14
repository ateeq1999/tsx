use std::time::Instant;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;
use crate::utils::paths::find_project_root;

use super::types::load_registries_index;

/// List community registries installed in `.tsx/registries.json` and packages in
/// `.tsx/packages/`.
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

    // Also list FPF packages under .tsx/packages/
    let packages_dir = root.join(".tsx").join("packages");
    let fpf_packages: Vec<serde_json::Value> = if let Ok(entries) = std::fs::read_dir(&packages_dir) {
        entries
            .flatten()
            .filter(|e| e.path().is_dir())
            .filter_map(|e| {
                let slug = e.file_name().to_string_lossy().to_string();
                let manifest_path = e.path().join("manifest.json");
                let version = std::fs::read_to_string(&manifest_path)
                    .ok()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                    .and_then(|v| v.get("version").and_then(|v| v.as_str()).map(String::from))
                    .unwrap_or_else(|| "?".to_string());
                let gen_count = std::fs::read_dir(e.path().join("generators"))
                    .map(|d| d.count())
                    .unwrap_or(0);
                Some(serde_json::json!({
                    "slug": slug,
                    "version": version,
                    "generators": gen_count,
                    "path": e.path().to_string_lossy()
                }))
            })
            .collect()
    } else {
        vec![]
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    let response = ResponseEnvelope::success(
        "registry:list",
        serde_json::json!({
            "legacy_registries": registries,
            "fpf_packages": fpf_packages,
            "total": registries.len() + fpf_packages.len(),
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

    CommandResult::ok("registry:list", vec![])
}
