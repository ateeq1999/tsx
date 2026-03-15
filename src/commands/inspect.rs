use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;
use walkdir::WalkDir;

use crate::json::error::ErrorResponse;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;
use crate::utils::paths::find_project_root;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectResult {
    pub project_root: String,
    pub tsx_version: String,
    pub app_name: String,
    pub structure: ProjectStructure,
    pub database: DatabaseInfo,
    pub auth: AuthInfo,
    pub config: ConfigInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectStructure {
    pub schemas: Vec<String>,
    pub server_functions: Vec<String>,
    pub queries: Vec<String>,
    pub forms: Vec<String>,
    pub tables: Vec<String>,
    pub routes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub provider: String,
    pub url: String,
    pub migrations_pending: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    pub configured: bool,
    pub providers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigInfo {
    pub tsconfig_path: String,
    pub shadcn_path: String,
}

pub fn inspect(verbose: bool) -> CommandResult {
    let start = Instant::now();
    let duration_ms = start.elapsed().as_millis() as u64;

    let project_root = match find_project_root() {
        Ok(root) => root,
        Err(_) => {
            let error = ErrorResponse::project_not_found();
            let response = ResponseEnvelope::error("inspect", error, duration_ms);
            response.print();
            return CommandResult::err("inspect", "Not running inside a TanStack Start project");
        }
    };

    let root_str = project_root.to_string_lossy().to_string();

    let mut structure = ProjectStructure::default();

    if let Ok(entries) = std::fs::read_dir(project_root.join("db/schema")) {
        structure.schemas = entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                e.path()
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
            })
            .collect();
    }

    if let Ok(entries) = std::fs::read_dir(project_root.join("server-functions")) {
        structure.server_functions = entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                e.path()
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
            })
            .collect();
    }

    if let Ok(entries) = std::fs::read_dir(project_root.join("queries")) {
        structure.queries = entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                e.path()
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
            })
            .collect();
    }

    if let Ok(entries) = std::fs::read_dir(project_root.join("routes")) {
        structure.routes = collect_routes(&project_root.join("routes"));
    }

    let app_name = project_root
        .join("package.json")
        .exists()
        .then(|| {
            std::fs::read_to_string(project_root.join("package.json"))
                .ok()
                .and_then(|content| {
                    serde_json::from_str::<serde_json::Value>(&content)
                        .ok()
                        .and_then(|v| v.get("name").and_then(|n| n.as_str()).map(String::from))
                })
        })
        .flatten()
        .unwrap_or_else(|| "Unknown".to_string());

    let db_path = project_root.join("drizzle.config.ts");
    let provider = if db_path.exists() {
        "sqlite"
    } else {
        "unknown"
    };

    let auth_path = project_root.join("lib/auth.ts");
    let auth = AuthInfo {
        configured: auth_path.exists(),
        providers: if auth_path.exists() {
            vec!["github".to_string(), "google".to_string()]
        } else {
            vec![]
        },
    };

    let result = InspectResult {
        project_root: root_str.clone(),
        tsx_version: env!("CARGO_PKG_VERSION").to_string(),
        app_name,
        structure,
        database: DatabaseInfo {
            provider: provider.to_string(),
            url: "sqlite://app.db".to_string(),
            migrations_pending: 0,
        },
        auth,
        config: ConfigInfo {
            tsconfig_path: "tsconfig.json".to_string(),
            shadcn_path: "components/ui".to_string(),
        },
    };

    let response = ResponseEnvelope::success(
        "inspect",
        serde_json::to_value(result).unwrap(),
        duration_ms,
    );

    if verbose {
        let context = crate::json::response::Context {
            project_root: root_str,
            tsx_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        let response = response.with_context(context);
        response.print();
    } else {
        response.print();
    }

    CommandResult::ok("inspect", vec![])
}

fn collect_routes(routes_dir: &Path) -> Vec<String> {
    let mut routes = Vec::new();

    for entry in WalkDir::new(routes_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "tsx" || ext == "ts" {
                if let Ok(relative) = path.strip_prefix(routes_dir) {
                    let route = relative
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if route != "index" && route != "_index" {
                        routes.push(route);
                    }
                }
            }
        }
    }

    routes
}
