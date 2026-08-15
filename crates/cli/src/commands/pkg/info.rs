use crate::commands::auth::load_credentials;
use crate::output::CommandResult;

use super::utils::{split_name_version, url_encode, DEFAULT_REGISTRY_URL};

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

fn str_field<'a>(v: &'a serde_json::Value, key: &str) -> String {
    v.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}
