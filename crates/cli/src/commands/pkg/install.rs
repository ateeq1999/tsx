use std::path::PathBuf;

use crate::commands::auth::load_credentials;
use crate::output::CommandResult;

use super::types::{load_pkg_index, save_pkg_index, InstalledPkg};
use super::utils::{split_name_version, url_encode, DEFAULT_REGISTRY_URL};

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
