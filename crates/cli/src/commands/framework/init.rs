use serde::Serialize;
use std::time::Instant;

use crate::json::error::ErrorResponse;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;
use crate::utils::paths::get_frameworks_dir;

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
