use std::time::Instant;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

use super::utils::{registry_url, urlencoding};

/// Search for packages. Prefers `TSX_REGISTRY_URL/v1/search` when set,
/// falls back to the npm registry.
pub fn registry_search(query: String, verbose: bool) -> CommandResult {
    let start = Instant::now();

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => return CommandResult::err("registry:search", e.to_string()),
    };

    // ── hosted registry path ──────────────────────────────────────────────────
    if let Some(server) = registry_url() {
        let url = format!(
            "{}/v1/search?q={}&size=20",
            server,
            urlencoding(query.trim())
        );
        let result = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .and_then(|r| r.json::<serde_json::Value>());

        let duration_ms = start.elapsed().as_millis() as u64;

        return match result {
            Ok(json) => {
                let results = json
                    .get("data")
                    .and_then(|d| d.get("results"))
                    .and_then(|r| r.as_array())
                    .cloned()
                    .unwrap_or_default();

                let packages: Vec<serde_json::Value> = results
                    .into_iter()
                    .map(|r| serde_json::json!({
                        "name": r.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                        "version": r.get("latest_version").and_then(|v| v.as_str()).unwrap_or("?"),
                        "description": r.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                        "provides": r.get("provides").cloned().unwrap_or(serde_json::json!([])),
                        "lang": r.get("lang").cloned().unwrap_or(serde_json::json!([])),
                        "install": r.get("install").and_then(|v| v.as_str()).unwrap_or(""),
                    }))
                    .collect();

                ResponseEnvelope::success(
                    "registry:search",
                    serde_json::json!({ "query": query, "results": packages, "source": server }),
                    duration_ms,
                )
                .print();
                CommandResult::ok("registry:search", vec![])
            }
            Err(e) => {
                let error = ErrorResponse::new(ErrorCode::InternalError, format!("Registry search failed: {}", e));
                ResponseEnvelope::error("registry:search", error, duration_ms).print();
                CommandResult::err("registry:search", e.to_string())
            }
        };
    }

    // ── npm fallback ──────────────────────────────────────────────────────────
    let search_text = if query.trim().is_empty() {
        "tsx-framework".to_string()
    } else {
        format!("tsx-framework {}", query)
    };

    let url = format!(
        "https://registry.npmjs.org/-/v1/search?text={}&size=20",
        urlencoding(&search_text)
    );

    let result = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
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
