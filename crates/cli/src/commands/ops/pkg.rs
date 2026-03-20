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

    let pkg_url = format!("{registry_url}/v1/packages/{}", url_encode(&name));
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

// ── Helpers ───────────────────────────────────────────────────────────────────

fn url_encode(s: &str) -> String {
    s.replace('@', "%40").replace('/', "%2F")
}

fn str_field<'a>(v: &'a serde_json::Value, key: &str) -> String {
    v.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}
