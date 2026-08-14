use std::time::Instant;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

use super::utils::{registry_url, urlencoding};

/// Fetch metadata for a single package. Prefers `TSX_REGISTRY_URL` when set,
/// falls back to npm + unpkg.
pub fn registry_info(package: String, _verbose: bool) -> CommandResult {
    let start = Instant::now();

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => return CommandResult::err("registry:info", e.to_string()),
    };

    // ── hosted registry path ──────────────────────────────────────────────────
    if let Some(server) = registry_url() {
        let encoded = package.replace('@', "%40").replace('/', "%2F");
        let url = format!("{}/v1/packages/{}", server, encoded);
        let result = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .and_then(|r| r.json::<serde_json::Value>());

        let duration_ms = start.elapsed().as_millis() as u64;

        return match result {
            Ok(json) => {
                let data = json.get("data").cloned().unwrap_or(serde_json::json!({}));
                let payload = serde_json::json!({
                    "package": data.get("name").and_then(|v| v.as_str()).unwrap_or(&package),
                    "version": data.get("latest_version").and_then(|v| v.as_str()).unwrap_or("unknown"),
                    "description": data.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                    "lang": data.get("lang").cloned().unwrap_or(serde_json::json!([])),
                    "provides": data.get("provides").cloned().unwrap_or(serde_json::json!([])),
                    "integrates_with": data.get("integrates_with").cloned().unwrap_or(serde_json::json!([])),
                    "downloads": data.get("downloads").cloned().unwrap_or(serde_json::json!(0)),
                    "manifest": data.get("manifest").cloned().unwrap_or(serde_json::Value::Null),
                    "install": format!("tsx registry install {}", package),
                    "source": server,
                });
                ResponseEnvelope::success("registry:info", payload, duration_ms).print();
                CommandResult::ok("registry:info", vec![])
            }
            Err(e) => {
                let error = ErrorResponse::new(ErrorCode::InternalError, format!("Registry request failed: {e}"));
                ResponseEnvelope::error("registry:info", error, duration_ms).print();
                CommandResult::err("registry:info", e.to_string())
            }
        };
    }

    // ── npm fallback ──────────────────────────────────────────────────────────
    let url = format!("https://registry.npmjs.org/{}", urlencoding(&package));

    let result = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .and_then(|r| r.json::<serde_json::Value>());

    let duration_ms = start.elapsed().as_millis() as u64;

    let json = match result {
        Ok(j) => j,
        Err(e) => {
            let error = ErrorResponse::new(ErrorCode::InternalError, format!("npm registry request failed: {e}"));
            ResponseEnvelope::error("registry:info", error, duration_ms).print();
            return CommandResult::err("registry:info", e.to_string());
        }
    };

    let latest_version = json
        .get("dist-tags")
        .and_then(|t| t.get("latest"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let description = json
        .get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("");

    let homepage = json
        .get("homepage")
        .and_then(|h| h.as_str())
        .unwrap_or("");

    // Try to fetch the FPF manifest from unpkg to get provides[] / integrates_with
    let manifest_url = format!("https://unpkg.com/{}/manifest.json", package);
    let manifest = client
        .get(&manifest_url)
        .send()
        .and_then(|r| r.json::<serde_json::Value>())
        .ok();

    let (provides, integrates_with, lang, generators) = if let Some(ref m) = manifest {
        (
            m.get("provides").cloned().unwrap_or(serde_json::Value::Null),
            m.get("integrates_with").cloned().unwrap_or(serde_json::Value::Null),
            m.get("lang").cloned().unwrap_or(serde_json::Value::Null),
            m.get("generators").cloned().unwrap_or(serde_json::Value::Null),
        )
    } else {
        (serde_json::Value::Null, serde_json::Value::Null, serde_json::Value::Null, serde_json::Value::Null)
    };

    let payload = serde_json::json!({
        "package": package,
        "version": latest_version,
        "description": description,
        "homepage": homepage,
        "lang": lang,
        "provides": provides,
        "generators": generators,
        "integrates_with": integrates_with,
        "install": format!("tsx registry install {}", package),
    });

    ResponseEnvelope::success("registry:info", payload, duration_ms).print();
    CommandResult::ok("registry:info", vec![])
}
