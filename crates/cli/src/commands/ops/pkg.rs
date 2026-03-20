use crate::commands::manage::auth::load_credentials;
use crate::output::CommandResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_REGISTRY_URL: &str = "https://tsx-tsnv.onrender.com";

// ── Installed packages index ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstalledPkg {
    pub name: String,
    pub version: String,
    pub registry_url: String,
    pub installed_at: String,
}

fn pkg_index_path(root: &std::path::Path) -> PathBuf {
    root.join(".tsx").join("packages.json")
}

fn load_pkg_index(root: &std::path::Path) -> Vec<InstalledPkg> {
    let path = pkg_index_path(root);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_pkg_index(root: &std::path::Path, pkgs: &[InstalledPkg]) -> anyhow::Result<()> {
    let path = pkg_index_path(root);
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(&path, serde_json::to_string_pretty(pkgs)?)?;
    Ok(())
}

fn iso_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // RFC 3339 without chrono: YYYY-MM-DDTHH:MM:SSZ
    let s = secs;
    let sec = s % 60;
    let min = (s / 60) % 60;
    let hr = (s / 3600) % 24;
    let days = s / 86400;
    // Simplified date math (approximate — good enough for a timestamp)
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    let month = day_of_year / 30 + 1;
    let day = day_of_year % 30 + 1;
    format!("{year:04}-{month:02}-{day:02}T{hr:02}:{min:02}:{sec:02}Z")
}

// ── tsx pkg info <name> ───────────────────────────────────────────────────────

/// `tsx pkg info <name>`
///
/// Fetches and displays package metadata from the tsx registry.
pub fn pkg_info(name: String) -> CommandResult {
    // Accept name@version shorthand (e.g. "my-pkg@1.2.3")
    let (name, pinned_version) = split_name_version(&name);

    let registry_url = load_credentials()
        .map(|c| c.registry_url)
        .unwrap_or_else(|| DEFAULT_REGISTRY_URL.to_string());

    let client = match reqwest::blocking::Client::builder()
        .user_agent(format!("tsx-cli/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => return CommandResult::err("pkg info", format!("HTTP client error: {e}")),
    };

    let pkg_url = match &pinned_version {
        Some(v) => format!("{registry_url}/v1/packages/{}/{}", url_encode(&name), url_encode(v)),
        None    => format!("{registry_url}/v1/packages/{}", url_encode(&name)),
    };
    let resp = match client.get(&pkg_url).send() {
        Ok(r) => r,
        Err(e) => return CommandResult::err("pkg info", format!("Request failed: {e}")),
    };

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return CommandResult::err("pkg info", format!("Package '{name}' not found"));
    }
    if !resp.status().is_success() {
        return CommandResult::err(
            "pkg info",
            format!("Registry returned {}", resp.status()),
        );
    }

    let pkg: serde_json::Value = match resp.json() {
        Ok(v) => v,
        Err(e) => return CommandResult::err("pkg info", format!("Failed to parse response: {e}")),
    };

    let version      = str_field(&pkg, "version");
    let description  = str_field(&pkg, "description");
    let author       = str_field(&pkg, "author");
    let license      = str_field(&pkg, "license");
    let downloads    = pkg.get("download_count").and_then(|v| v.as_i64()).unwrap_or(0);
    let updated_at   = str_field(&pkg, "updated_at");
    let tags: Vec<String> = pkg
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(String::from).collect())
        .unwrap_or_default();

    let install_cmd = format!("tsx pkg install {name}");
    let mut lines = vec![
        format!("{name} v{version}"),
        format!("  {description}"),
        format!("  Author:    {author}"),
        format!("  License:   {license}"),
        format!("  Downloads: {downloads}"),
        format!("  Updated:   {updated_at}"),
    ];
    if !tags.is_empty() {
        lines.push(format!("  Tags:      {}", tags.join(", ")));
    }

    let mut result = CommandResult::ok("pkg info", vec![]);
    result.next_steps = lines;
    result.next_steps.push(format!("Install: {install_cmd}"));
    result
}

// ── tsx pkg install <name>[@version] ─────────────────────────────────────────

/// `tsx pkg install <name> [--version <ver>] [--target <dir>]`
///
/// Downloads the tarball from the tsx registry and extracts it into
/// `.tsx/packages/<name>/` in the nearest project root (or `--target`).
pub fn pkg_install(name: String, version: Option<String>, target: Option<String>) -> CommandResult {
    // Accept name@version shorthand — --version flag takes precedence
    let (name, name_version) = split_name_version(&name);
    let version = version.or(name_version);

    let creds = load_credentials();
    let registry_url = creds
        .as_ref()
        .map(|c| c.registry_url.as_str())
        .unwrap_or(DEFAULT_REGISTRY_URL)
        .trim_end_matches('/')
        .to_string();

    let client = match reqwest::blocking::Client::builder()
        .user_agent(format!("tsx-cli/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(60))
        .build()
    {
        Ok(c) => c,
        Err(e) => return CommandResult::err("pkg install", format!("HTTP client error: {e}")),
    };

    // ── Resolve version ───────────────────────────────────────────────────────
    let resolved_version = if let Some(v) = version {
        v
    } else {
        let pkg_url = format!("{registry_url}/v1/packages/{}", url_encode(&name));
        let resp = match client.get(&pkg_url).send() {
            Ok(r) => r,
            Err(e) => return CommandResult::err("pkg install", format!("Failed to fetch package info: {e}")),
        };
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return CommandResult::err("pkg install", format!("Package '{name}' not found in registry"));
        }
        if !resp.status().is_success() {
            return CommandResult::err("pkg install", format!("Registry error: {}", resp.status()));
        }
        let pkg: serde_json::Value = match resp.json() {
            Ok(v) => v,
            Err(e) => return CommandResult::err("pkg install", format!("Failed to parse response: {e}")),
        };
        match pkg.get("version").and_then(|v| v.as_str()) {
            Some(v) => v.to_string(),
            None => return CommandResult::err("pkg install", "Could not determine latest version"),
        }
    };

    // ── Download tarball ──────────────────────────────────────────────────────
    let tarball_url = format!(
        "{registry_url}/v1/packages/{}/{resolved_version}/tarball",
        url_encode(&name)
    );

    let tarball_bytes = match client.get(&tarball_url).send().and_then(|r| {
        r.error_for_status()?.bytes().map(|b| b.to_vec())
    }) {
        Ok(b) => b,
        Err(e) => return CommandResult::err("pkg install", format!("Download failed: {e}")),
    };

    // ── Resolve install directory ─────────────────────────────────────────────
    let install_root = if let Some(t) = target {
        PathBuf::from(t)
    } else {
        let project_root = crate::utils::paths::find_project_root()
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        project_root.join(".tsx").join("packages")
    };

    let pkg_dir = install_root.join(name.replace('/', "__"));
    if let Err(e) = std::fs::create_dir_all(&pkg_dir) {
        return CommandResult::err("pkg install", format!("Cannot create install dir: {e}"));
    }

    // ── Extract tarball ───────────────────────────────────────────────────────
    let gz = flate2::read::GzDecoder::new(tarball_bytes.as_slice());
    let mut archive = tar::Archive::new(gz);
    if let Err(e) = archive.unpack(&pkg_dir) {
        return CommandResult::err("pkg install", format!("Extraction failed: {e}"));
    }

    // ── Update packages index ─────────────────────────────────────────────────
    let index_root = install_root.parent().unwrap_or(&install_root);
    let mut pkgs = load_pkg_index(index_root);
    pkgs.retain(|p| p.name != name);
    pkgs.push(InstalledPkg {
        name: name.clone(),
        version: resolved_version.clone(),
        registry_url: registry_url.clone(),
        installed_at: iso_now(),
    });
    let _ = save_pkg_index(index_root, &pkgs);

    let installed_path = pkg_dir.display().to_string();
    let mut result = CommandResult::ok("pkg install", vec![installed_path.clone()]);
    result.next_steps = vec![
        format!("Installed {name}@{resolved_version} → {installed_path}"),
    ];
    result
}

// ── tsx pkg upgrade <name> ────────────────────────────────────────────────────

/// `tsx pkg upgrade <name>`
///
/// Fetches the latest version from the registry and re-installs the package
/// if a newer version is available.  Prints an up-to-date message otherwise.
pub fn pkg_upgrade(name: String, target: Option<String>) -> CommandResult {
    // Accept name@version (unusual but handle gracefully — just ignore the pin)
    let (name, _pinned) = split_name_version(&name);

    let registry_url = load_credentials()
        .map(|c| c.registry_url)
        .unwrap_or_else(|| DEFAULT_REGISTRY_URL.to_string());
    let registry_url = registry_url.trim_end_matches('/').to_string();

    // ── Resolve project root & current installed version ──────────────────────
    let project_root = crate::utils::paths::find_project_root()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let pkgs = load_pkg_index(&project_root);
    let current_version = pkgs.iter().find(|p| p.name == name).map(|p| p.version.clone());

    // ── Fetch latest version from registry ───────────────────────────────────
    let client = match reqwest::blocking::Client::builder()
        .user_agent(format!("tsx-cli/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => return CommandResult::err("pkg upgrade", format!("HTTP client error: {e}")),
    };

    let pkg_url = format!("{registry_url}/v1/packages/{}", url_encode(&name));
    let resp = match client.get(&pkg_url).send() {
        Ok(r) => r,
        Err(e) => return CommandResult::err("pkg upgrade", format!("Request failed: {e}")),
    };
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return CommandResult::err("pkg upgrade", format!("Package '{name}' not found in registry"));
    }
    if !resp.status().is_success() {
        return CommandResult::err("pkg upgrade", format!("Registry returned {}", resp.status()));
    }
    let pkg: serde_json::Value = match resp.json() {
        Ok(v) => v,
        Err(e) => return CommandResult::err("pkg upgrade", format!("Failed to parse response: {e}")),
    };
    let latest = match pkg.get("version").and_then(|v| v.as_str()) {
        Some(v) => v.to_string(),
        None => return CommandResult::err("pkg upgrade", "Could not determine latest version"),
    };

    // ── Already up to date? ───────────────────────────────────────────────────
    if current_version.as_deref() == Some(&latest) {
        let mut result = CommandResult::ok("pkg upgrade", vec![]);
        result.next_steps = vec![format!("{name}@{latest} is already the latest version")];
        return result;
    }

    let prev = current_version.as_deref().unwrap_or("(not installed)");

    // ── Re-install ────────────────────────────────────────────────────────────
    let install_result = pkg_install(name.clone(), Some(latest.clone()), target);
    if !install_result.success {
        return install_result;
    }

    let mut result = CommandResult::ok("pkg upgrade", vec![]);
    result.next_steps = vec![
        format!("Upgraded {name}: {prev} → {latest}"),
        format!("  Run `tsx pkg info {name}` for release notes."),
    ];
    result
}

// ── tsx pkg publish [--path <dir>] [--name <n>] [--version <v>] [--dry-run] ──

/// `tsx pkg publish`
///
/// Packages the current directory (or `--path`) as a `.tar.gz` and uploads it
/// to the tsx registry using the API key stored by `tsx login`.
pub fn pkg_publish(
    path: Option<String>,
    name: Option<String>,
    version: Option<String>,
    dry_run: bool,
) -> CommandResult {
    // ── Credentials ───────────────────────────────────────────────────────────
    let creds = match load_credentials() {
        Some(c) => c,
        None => {
            return CommandResult::err(
                "pkg publish",
                "Not logged in. Run `tsx login --token <key>` first.",
            )
        }
    };

    // ── Resolve package directory ─────────────────────────────────────────────
    let pkg_dir = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
    };

    // ── Read manifest.json ────────────────────────────────────────────────────
    let manifest_path = pkg_dir.join("manifest.json");
    let manifest_str = match std::fs::read_to_string(&manifest_path) {
        Ok(s) => s,
        Err(e) => {
            return CommandResult::err(
                "pkg publish",
                format!("Could not read manifest.json in {}: {e}", pkg_dir.display()),
            )
        }
    };
    let manifest: serde_json::Value = match serde_json::from_str(&manifest_str) {
        Ok(v) => v,
        Err(e) => {
            return CommandResult::err("pkg publish", format!("Invalid manifest.json: {e}"))
        }
    };

    // ── Resolve name / version ────────────────────────────────────────────────
    let pkg_name = name
        .or_else(|| {
            manifest
                .get("name")
                .and_then(|v| v.as_str())
                .map(String::from)
        })
        .or_else(|| {
            manifest
                .get("id")
                .and_then(|v| v.as_str())
                .map(String::from)
        })
        .unwrap_or_default();

    let pkg_version = version
        .or_else(|| {
            manifest
                .get("version")
                .and_then(|v| v.as_str())
                .map(String::from)
        })
        .unwrap_or_default();

    if pkg_name.is_empty() {
        return CommandResult::err(
            "pkg publish",
            "Package name not found in manifest.json — add a \"name\" field or pass --name.",
        );
    }
    if pkg_version.is_empty() {
        return CommandResult::err(
            "pkg publish",
            "Version not found in manifest.json — add a \"version\" field or pass --version.",
        );
    }

    let registry_url = creds.registry_url.trim_end_matches('/').to_string();

    // ── Dry-run: print what would be published ────────────────────────────────
    if dry_run {
        let mut result = CommandResult::ok("pkg publish", vec![]);
        result.next_steps = vec![
            format!("Would publish {pkg_name}@{pkg_version} to {registry_url}"),
            format!("  Source: {}", pkg_dir.display()),
        ];
        return result;
    }

    // ── Build tarball ─────────────────────────────────────────────────────────
    let tarball_bytes = match build_tarball(&pkg_dir) {
        Ok(b) => b,
        Err(e) => {
            return CommandResult::err("pkg publish", format!("Failed to build tarball: {e}"))
        }
    };

    // ── Upload ────────────────────────────────────────────────────────────────
    let client = match reqwest::blocking::Client::builder()
        .user_agent(format!("tsx-cli/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(120))
        .build()
    {
        Ok(c) => c,
        Err(e) => return CommandResult::err("pkg publish", format!("HTTP client error: {e}")),
    };

    let file_name = format!("{}-{}.tar.gz", pkg_name.replace('/', "__"), pkg_version);
    let part = match reqwest::blocking::multipart::Part::bytes(tarball_bytes)
        .file_name(file_name)
        .mime_str("application/gzip")
    {
        Ok(p) => p,
        Err(e) => return CommandResult::err("pkg publish", format!("MIME error: {e}")),
    };

    let form = reqwest::blocking::multipart::Form::new()
        .text("name", pkg_name.clone())
        .text("version", pkg_version.clone())
        .text("manifest", manifest_str)
        .part("tarball", part);

    let resp = match client
        .post(format!("{registry_url}/v1/packages/publish"))
        .header("Authorization", format!("Bearer {}", creds.api_key))
        .multipart(form)
        .send()
    {
        Ok(r) => r,
        Err(e) => return CommandResult::err("pkg publish", format!("Request failed: {e}")),
    };

    let status = resp.status();
    let body: serde_json::Value = resp.json().unwrap_or_default();

    if !status.is_success() {
        let msg = body
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        return CommandResult::err("pkg publish", format!("Registry returned {status}: {msg}"));
    }

    let tarball_url = body
        .get("tarball_url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let mut result = CommandResult::ok("pkg publish", vec![]);
    result.next_steps = vec![
        format!("Published {pkg_name}@{pkg_version}"),
        format!("  Registry: {registry_url}"),
        format!("  Install:  tsx pkg install {pkg_name}"),
    ];
    if !tarball_url.is_empty() {
        result.next_steps.push(format!("  Tarball:  {tarball_url}"));
    }
    result
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn build_tarball(dir: &std::path::Path) -> anyhow::Result<Vec<u8>> {
    let mut buf = Vec::new();
    {
        let enc = flate2::write::GzEncoder::new(&mut buf, flate2::Compression::default());
        let mut archive = tar::Builder::new(enc);
        archive.append_dir_all(".", dir)?;
        let enc = archive.into_inner()?;
        enc.finish()?;
    }
    Ok(buf)
}

/// Split `name@version` → `(name, Some(version))`, or `name` → `(name, None)`.
/// Handles scoped packages like `@scope/pkg@1.0.0`:
///   - If the string starts with `@`, the leading `@` belongs to the scope, so
///     we look for `@` after position 1.
fn split_name_version(input: &str) -> (String, Option<String>) {
    let at_pos = if input.starts_with('@') {
        input[1..].find('@').map(|i| i + 1)
    } else {
        input.find('@')
    };
    match at_pos {
        Some(i) => (input[..i].to_string(), Some(input[i + 1..].to_string())),
        None    => (input.to_string(), None),
    }
}

fn url_encode(s: &str) -> String {
    s.replace('@', "%40").replace('/', "%2F")
}

fn str_field<'a>(v: &'a serde_json::Value, key: &str) -> String {
    v.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}
