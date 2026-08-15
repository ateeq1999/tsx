use serde::Serialize;
use std::time::Instant;

use crate::json::error::ErrorResponse;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

#[derive(Serialize)]
struct PublishResult {
    framework: String,
    version: String,
    package_name: String,
    published: bool,
    dry_run: bool,
}

/// Generate a publish-ready `package.json` for the framework directory and either:
/// - `npm publish --access public` (default), or
/// - multipart-upload to a hosted registry (`--registry <url>`)
pub fn framework_publish(
    path: Option<String>,
    dry_run: bool,
    registry: Option<String>,
    api_key: Option<String>,
    verbose: bool,
) -> CommandResult {
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
    } else if let Some(registry_url) = registry {
        // ── Hosted registry upload ────────────────────────────────────────────
        // Resolve API key: --api-key flag → TSX_REGISTRY_API_KEY env var
        let key = api_key
            .or_else(|| std::env::var("TSX_REGISTRY_API_KEY").ok())
            .unwrap_or_default();

        // Build a .tar.gz of the package directory in memory
        let tarball_bytes = match build_tarball(&pkg_dir) {
            Ok(b) => b,
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let error = ErrorResponse::new(
                    crate::json::error::ErrorCode::InternalError,
                    &format!("Failed to create tarball: {}", e),
                );
                ResponseEnvelope::error("framework:publish", error, duration_ms).print();
                return CommandResult::err("framework:publish", "Tarball creation failed");
            }
        };

        let client = match reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
        {
            Ok(c) => c,
            Err(e) => return CommandResult::err("framework:publish", e.to_string()),
        };

        let url = format!(
            "{}/v1/packages/publish",
            registry_url.trim_end_matches('/')
        );
        let form = reqwest::blocking::multipart::Form::new()
            .text("name", package_name.clone())
            .text("version", fw_version.to_string())
            .text("manifest", manifest_content.clone())
            .part(
                "tarball",
                reqwest::blocking::multipart::Part::bytes(tarball_bytes)
                    .file_name(format!("{}-{}.tar.gz", fw_id, fw_version))
                    .mime_str("application/gzip")
                    .unwrap(),
            );

        let mut req = client.post(&url).multipart(form);
        if !key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        match req.send() {
            Ok(resp) if resp.status().is_success() => true,
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().unwrap_or_default();
                let duration_ms = start.elapsed().as_millis() as u64;
                let error = ErrorResponse::validation(&format!(
                    "Registry publish failed (HTTP {}): {}",
                    status, body.trim()
                ));
                ResponseEnvelope::error("framework:publish", error, duration_ms).print();
                return CommandResult::err("framework:publish", "Registry publish failed");
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let error = ErrorResponse::new(
                    crate::json::error::ErrorCode::InternalError,
                    &format!("HTTP request failed: {}", e),
                );
                ResponseEnvelope::error("framework:publish", error, duration_ms).print();
                return CommandResult::err("framework:publish", "HTTP error");
            }
        }
    } else {
        // ── npm publish ───────────────────────────────────────────────────────
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

/// Build an in-memory `.tar.gz` of a directory for registry upload.
fn build_tarball(dir: &std::path::Path) -> anyhow::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let enc = flate2::write::GzEncoder::new(&mut buf, flate2::Compression::default());
    let mut archive = tar::Builder::new(enc);
    append_dir_to_archive(&mut archive, dir, std::path::Path::new("package"))?;
    let gz_enc = archive.into_inner()?;
    gz_enc.finish()?;
    Ok(buf)
}

fn append_dir_to_archive<W: std::io::Write>(
    builder: &mut tar::Builder<W>,
    dir: &std::path::Path,
    prefix: &std::path::Path,
) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        let rel = prefix.join(entry.file_name());
        if path.is_dir() {
            append_dir_to_archive(builder, &path, &rel)?;
        } else {
            let mut f = std::fs::File::open(&path)?;
            builder.append_file(&rel, &mut f)?;
        }
    }
    Ok(())
}
