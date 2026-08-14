use std::path::Path;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

use super::lint::pattern_lint;
use super::utils::read_registry_url;

/// Publish a pack to the configured registry.
pub fn pattern_publish(id: String, registry: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    let Some(pack) = forge::PackManifest::load(&cwd, &id) else {
        return ResponseEnvelope::error(
            "pattern publish",
            ErrorResponse::new(ErrorCode::ProjectNotFound, format!("Pack '{}' not found. Run `tsx pattern new` or `tsx pattern list`.", id)),
            0,
        );
    };

    let pack_dir = forge::PackManifest::dir(&cwd, &id);

    // Lint check before publish
    let lint_resp = pattern_lint(id.clone(), false);
    if !lint_resp.success {
        return lint_resp;
    }

    let registry_url = registry.unwrap_or_else(|| read_registry_url(&cwd));

    // Bundle pack directory into an in-memory tar.gz
    let tarball = match bundle_pack_dir(&pack_dir) {
        Ok(b) => b,
        Err(e) => return ResponseEnvelope::error(
            "pattern publish",
            ErrorResponse::new(ErrorCode::InternalError, format!("Bundle error: {e}")),
            0,
        ),
    };

    // Read README if present
    let readme = std::fs::read_to_string(pack_dir.join("README.md")).ok();

    // POST multipart to /v1/patterns/publish
    let manifest_json = serde_json::to_string(&serde_json::json!({
        "id":          pack.id,
        "name":        pack.name,
        "version":     pack.version,
        "description": pack.description,
        "author":      pack.author,
        "framework":   pack.framework,
        "tags":        pack.tags,
    })).unwrap_or_default();

    let publish_url = format!("{}/v1/patterns/publish", registry_url.trim_end_matches('/'));

    let client = match reqwest::blocking::Client::builder().user_agent("tsx-cli/0.1").build() {
        Ok(c) => c,
        Err(e) => return ResponseEnvelope::error("pattern publish", ErrorResponse::new(ErrorCode::InternalError, e.to_string()), 0),
    };

    let form = reqwest::blocking::multipart::Form::new()
        .part("tarball", reqwest::blocking::multipart::Part::bytes(tarball)
            .file_name(format!("{}-{}.tar.gz", id, pack.version))
            .mime_str("application/gzip").unwrap())
        .text("manifest", manifest_json)
        .text("author", pack.author.clone());

    let form = if let Some(r) = readme {
        form.text("readme", r)
    } else {
        form
    };

    let slug = if id.contains('/') { id.clone() } else { format!("{}/{}", pack.author, id) };

    match client.post(&publish_url).multipart(form).send() {
        Ok(resp) if resp.status().is_success() => {
            ResponseEnvelope::success(
                "pattern publish",
                serde_json::json!({
                    "slug":     slug,
                    "version":  pack.version,
                    "registry": registry_url,
                    "url":      format!("{}/v1/patterns/{}", registry_url.trim_end_matches('/'), slug),
                }),
                0,
            )
        }
        Ok(resp) => {
            let status = resp.status().as_u16();
            let body = resp.text().unwrap_or_default();
            ResponseEnvelope::error(
                "pattern publish",
                ErrorResponse::new(ErrorCode::InternalError, format!("Registry returned {status}: {body}")),
                0,
            )
        }
        Err(e) => ResponseEnvelope::error(
            "pattern publish",
            ErrorResponse::new(ErrorCode::InternalError, format!("Request failed: {e}")),
            0,
        ),
    }
}

/// Bundle a pack directory into an in-memory .tar.gz.
fn bundle_pack_dir(dir: &Path) -> anyhow::Result<Vec<u8>> {
    use flate2::{write::GzEncoder, Compression};
    let mut buf = Vec::new();
    {
        let gz = GzEncoder::new(&mut buf, Compression::default());
        let mut archive = tar::Builder::new(gz);
        archive.append_dir_all(".", dir)?;
        archive.finish()?;
    }
    Ok(buf)
}
