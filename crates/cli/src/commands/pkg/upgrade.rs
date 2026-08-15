use std::path::PathBuf;

use crate::commands::auth::load_credentials;
use crate::output::CommandResult;

use super::install::pkg_install;
use super::types::load_pkg_index;
use super::utils::{split_name_version, url_encode, DEFAULT_REGISTRY_URL};

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
