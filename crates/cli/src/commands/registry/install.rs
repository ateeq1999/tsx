use std::path::Path;
use std::time::Instant;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;
use crate::utils::paths::find_project_root;

use super::types::{load_registries_index, save_registries_index, InstalledRegistry};
use super::utils::{iso_now, registry_url};

/// Install a tsx package into the project.
///
/// Two package formats are supported:
/// - **FPF packages** (`@tsx-pkg/<name>`): Full Framework Package Format — downloads
///   `manifest.json` and all `generators/*.json` from unpkg, installs to `.tsx/packages/<slug>/`.
///   These packages are picked up automatically by `CommandRegistry::load_all()`.
/// - **Legacy registry packages** (`tsx-framework-*`): downloads a single `registry.json`
///   from unpkg, installs to `.tsx/frameworks/<slug>/`.
///
/// The installed package is tracked in `.tsx/registries.json`.
pub fn registry_install(package: String, verbose: bool) -> CommandResult {
    let start = Instant::now();

    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(ErrorCode::ProjectNotFound, e.to_string());
            ResponseEnvelope::error("registry:install", error, duration_ms).print();
            return CommandResult::err("registry:install", e.to_string());
        }
    };

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => return CommandResult::err("registry:install", e.to_string()),
    };

    // Fetch package metadata from npm to resolve latest version
    let npm_url = format!(
        "https://registry.npmjs.org/{}",
        package.replace('/', "%2F")
    );
    let pkg_meta: serde_json::Value = match client
        .get(&npm_url)
        .header("Accept", "application/json")
        .send()
        .and_then(|r| r.json())
    {
        Ok(v) => v,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(
                ErrorCode::InternalError,
                format!("Failed to fetch '{}' from npm: {}", package, e),
            );
            ResponseEnvelope::error("registry:install", error, duration_ms).print();
            return CommandResult::err("registry:install", e.to_string());
        }
    };

    let latest = pkg_meta
        .get("dist-tags")
        .and_then(|t| t.get("latest"))
        .and_then(|v| v.as_str())
        .unwrap_or("latest")
        .to_string();

    let (slug, files_written) = if package.starts_with("@tsx-pkg/") {
        // FPF install: prefer hosted registry server; fall back to unpkg
        let fpf_result = if let Some(server) = registry_url() {
            install_fpf_from_server(&package, &root, &client, &server)
                .map(|(slug, version, files)| {
                    // update `latest` in the outer scope via return value
                    let _ = version;
                    (slug, files)
                })
                .or_else(|_| install_fpf_package(&package, &latest, &root, &client))
        } else {
            install_fpf_package(&package, &latest, &root, &client)
        };
        match fpf_result {
            Ok(result) => result,
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let error = ErrorResponse::new(ErrorCode::InternalError, e.to_string());
                ResponseEnvelope::error("registry:install", error, duration_ms).print();
                return CommandResult::err("registry:install", e.to_string());
            }
        }
    } else {
        // Legacy install: fetch registry.json → .tsx/frameworks/<slug>/
        match install_legacy_package(&package, &latest, &root, &client) {
            Ok(result) => result,
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let error = ErrorResponse::new(ErrorCode::InternalError, e.to_string());
                ResponseEnvelope::error("registry:install", error, duration_ms).print();
                return CommandResult::err("registry:install", e.to_string());
            }
        }
    };

    // Update registries index
    let mut index = load_registries_index(&root);
    index.retain(|r| r.slug != slug);
    index.push(InstalledRegistry {
        slug: slug.clone(),
        package: package.clone(),
        version: latest.clone(),
        source: format!("https://unpkg.com/{}", package),
        installed_at: iso_now(),
    });
    let _ = save_registries_index(&root, &index);

    let duration_ms = start.elapsed().as_millis() as u64;
    let response = ResponseEnvelope::success(
        "registry:install",
        serde_json::json!({
            "installed": { "slug": slug, "package": package, "version": latest },
            "files": files_written,
        }),
        duration_ms,
    )
    .with_next_steps(vec![
        format!("Run `tsx stack add {}` to activate this package in your project", slug),
        "Run `tsx list` to see the new commands".to_string(),
    ]);

    if verbose {
        let context = crate::json::response::Context {
            project_root: root.to_string_lossy().to_string(),
            tsx_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        response.with_context(context).print();
    } else {
        response.print();
    }

    CommandResult::ok("registry:install", files_written)
}

/// Install a FPF `@tsx-pkg/` package from the hosted registry: downloads tarball,
/// extracts to `.tsx/packages/<slug>/`, and checks `tsx_min` compatibility.
/// Returns `(slug, latest_version, files_written)`.
fn install_fpf_from_server(
    package: &str,
    root: &Path,
    client: &reqwest::blocking::Client,
    server_url: &str,
) -> anyhow::Result<(String, String, Vec<String>)> {
    let encoded = package.replace('@', "%40").replace('/', "%2F");
    let info_url = format!("{}/v1/packages/{}", server_url, encoded);

    let info: serde_json::Value = client
        .get(&info_url)
        .header("Accept", "application/json")
        .send()?
        .json()?;

    let data = info
        .get("data")
        .ok_or_else(|| anyhow::anyhow!("Invalid response from registry"))?;

    let latest_version = data
        .get("latest_version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Check tsx_min compatibility
    if let Some(manifest) = data.get("manifest") {
        check_tsx_min(manifest)?;
    }

    let slug = package.split('/').last().unwrap_or(package).to_string();

    // Download tarball
    let tarball_url = format!(
        "{}/v1/packages/{}/{}/tarball",
        server_url, encoded, latest_version
    );
    let tarball_bytes = client.get(&tarball_url).send()?.bytes()?;

    // Extract to .tsx/packages/<slug>/
    let dest = root.join(".tsx").join("packages").join(&slug);
    std::fs::create_dir_all(&dest)?;

    let decoder = flate2::read::GzDecoder::new(std::io::Cursor::new(tarball_bytes));
    let mut archive = tar::Archive::new(decoder);
    let mut files = vec![];

    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = entry.path()?.into_owned();
        // Strip leading component (e.g. "package/" added by npm pack)
        let stripped: std::path::PathBuf = entry_path.components().skip(1).collect();
        if stripped.as_os_str().is_empty() {
            continue;
        }
        let dest_path = dest.join(&stripped);
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        entry.unpack(&dest_path)?;
        files.push(dest_path.to_string_lossy().to_string());
    }

    Ok((slug, latest_version, files))
}

/// Check the `tsx_min` field in a manifest JSON value against the running CLI version.
fn check_tsx_min(manifest: &serde_json::Value) -> anyhow::Result<()> {
    let tsx_min = match manifest.get("tsx_min").and_then(|v| v.as_str()) {
        Some(v) => v,
        None => return Ok(()),
    };
    let cli_ver = env!("CARGO_PKG_VERSION");
    match (
        semver::Version::parse(tsx_min),
        semver::Version::parse(cli_ver),
    ) {
        (Ok(min), Ok(cli)) if cli < min => Err(anyhow::anyhow!(
            "Package requires tsx >= {} but you are running {}. Run `cargo install tsx` to upgrade.",
            tsx_min,
            cli_ver
        )),
        _ => Ok(()),
    }
}

/// Install a FPF `@tsx-pkg/` package: downloads manifest.json and all generators to
/// `.tsx/packages/<slug>/`.  Returns `(slug, files_written)`.
pub(super) fn install_fpf_package(
    package: &str,
    version: &str,
    root: &Path,
    client: &reqwest::blocking::Client,
) -> anyhow::Result<(String, Vec<String>)> {
    let base_url = format!("https://unpkg.com/{}@{}", package, version);

    // Fetch manifest.json
    let manifest_url = format!("{}/manifest.json", base_url);
    let manifest: serde_json::Value = client
        .get(&manifest_url)
        .header("Accept", "application/json")
        .send()?
        .json()?;

    // Check tsx_min compatibility
    check_tsx_min(&manifest)?;

    // Derive slug from package name: @tsx-pkg/tanstack-start → tanstack-start
    let slug = package
        .split('/')
        .last()
        .unwrap_or(package)
        .to_string();

    let dest = root.join(".tsx").join("packages").join(&slug);
    std::fs::create_dir_all(dest.join("generators"))?;

    // Write manifest.json
    let manifest_path = dest.join("manifest.json");
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    let mut files = vec![manifest_path.to_string_lossy().to_string()];

    // Fetch each generator listed in `provides`
    let provides: Vec<String> = manifest
        .get("provides")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|cmd| {
                    // "add:feature" → "add-feature"
                    cmd.replace(':', "-")
                })
                .collect()
        })
        .unwrap_or_default();

    for gen_id in &provides {
        let gen_url = format!("{}/generators/{}.json", base_url, gen_id);
        if let Ok(resp) = client.get(&gen_url).send() {
            if resp.status().is_success() {
                if let Ok(gen_json) = resp.json::<serde_json::Value>() {
                    let gen_path = dest.join("generators").join(format!("{}.json", gen_id));
                    std::fs::write(&gen_path, serde_json::to_string_pretty(&gen_json)?)?;
                    files.push(gen_path.to_string_lossy().to_string());
                }
            }
        }
    }

    Ok((slug, files))
}

/// Install a legacy `tsx-framework-*` package: fetches `registry.json` and writes to
/// `.tsx/frameworks/<slug>/`.  Returns `(slug, files_written)`.
pub(super) fn install_legacy_package(
    package: &str,
    version: &str,
    root: &Path,
    client: &reqwest::blocking::Client,
) -> anyhow::Result<(String, Vec<String>)> {
    let registry_url = format!("https://unpkg.com/{}@{}/registry.json", package, version);
    let registry_json: serde_json::Value = client
        .get(&registry_url)
        .header("Accept", "application/json")
        .send()?
        .json()?;

    let slug = registry_json
        .get("slug")
        .and_then(|s| s.as_str())
        .ok_or_else(|| anyhow::anyhow!("registry.json is missing required 'slug' field"))?
        .to_string();

    let dest_dir = root.join(".tsx").join("frameworks").join(&slug);
    std::fs::create_dir_all(&dest_dir)?;
    let dest_file = dest_dir.join("registry.json");
    std::fs::write(&dest_file, serde_json::to_string_pretty(&registry_json)?)?;

    Ok((slug, vec![dest_file.to_string_lossy().to_string()]))
}
