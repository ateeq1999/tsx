use std::time::Instant;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;
use crate::plugin::PluginRegistry;

pub fn plugin_list(verbose: bool) -> CommandResult {
    let start = Instant::now();

    let registry = match PluginRegistry::open() {
        Ok(r) => r,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(ErrorCode::InternalError, &e.to_string());
            ResponseEnvelope::error("plugin:list", error, duration_ms).print();
            return CommandResult::err("plugin:list", e.to_string());
        }
    };

    let plugins = registry.list();
    let duration_ms = start.elapsed().as_millis() as u64;

    let data = serde_json::json!({
        "plugins": plugins.iter().map(|p| serde_json::json!({
            "package": p.package,
            "name": p.name,
            "version": p.version,
            "description": p.description,
            "generators": p.generators.len(),
            "overrides": p.overrides.len(),
        })).collect::<Vec<_>>()
    });

    let response = ResponseEnvelope::success("plugin:list", data, duration_ms);

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

    CommandResult::ok("plugin:list", vec![])
}

pub fn plugin_install(source: String, verbose: bool) -> CommandResult {
    let start = Instant::now();

    let registry = match PluginRegistry::open() {
        Ok(r) => r,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(ErrorCode::InternalError, &e.to_string());
            ResponseEnvelope::error("plugin:install", error, duration_ms).print();
            return CommandResult::err("plugin:install", e.to_string());
        }
    };

    let path = std::path::Path::new(&source);
    let result = if path.exists() {
        registry.install_from_dir(path)
    } else {
        registry.install_from_npm(&source)
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(manifest) => {
            let data = serde_json::json!({
                "installed": {
                    "package": manifest.package,
                    "name": manifest.name,
                    "version": manifest.version,
                    "generators": manifest.generators.len(),
                    "overrides": manifest.overrides.len(),
                }
            });
            let response = ResponseEnvelope::success("plugin:install", data, duration_ms);
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
            CommandResult::ok("plugin:install", vec![manifest.package])
        }
        Err(e) => {
            let error = ErrorResponse::new(ErrorCode::InternalError, &e.to_string());
            ResponseEnvelope::error("plugin:install", error, duration_ms).print();
            CommandResult::err("plugin:install", e.to_string())
        }
    }
}

pub fn plugin_remove(package: String, verbose: bool) -> CommandResult {
    let start = Instant::now();

    let registry = match PluginRegistry::open() {
        Ok(r) => r,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(ErrorCode::InternalError, &e.to_string());
            ResponseEnvelope::error("plugin:remove", error, duration_ms).print();
            return CommandResult::err("plugin:remove", e.to_string());
        }
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    match registry.remove(&package) {
        Ok(()) => {
            let data = serde_json::json!({ "removed": package });
            let response = ResponseEnvelope::success("plugin:remove", data, duration_ms);
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
            CommandResult::ok("plugin:remove", vec![])
        }
        Err(e) => {
            let error = ErrorResponse::new(ErrorCode::InternalError, &e.to_string());
            ResponseEnvelope::error("plugin:remove", error, duration_ms).print();
            CommandResult::err("plugin:remove", e.to_string())
        }
    }
}
