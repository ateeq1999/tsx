//! Helpers shared by two or more `pattern` actions (install, list, publish, search).

use std::path::Path;

/// Built-in pattern packs embedded in the binary at compile time.
pub(super) static BUILTIN_PACKS: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../../patterns");

/// Walk the embedded BUILTIN_PACKS dir and return relative paths to pack directories
/// (those that contain a `pack.json` file).
pub(super) fn collect_builtin_pack_paths() -> Vec<String> {
    let mut out = Vec::new();
    collect_pack_paths_in(&BUILTIN_PACKS, "", &mut out);
    out
}

fn collect_pack_paths_in(dir: &include_dir::Dir<'_>, prefix: &str, out: &mut Vec<String>) {
    // If this dir contains pack.json it's a pack root
    let prefix_path = if prefix.is_empty() { String::new() } else { format!("{}/", prefix) };
    if dir.files().any(|f| f.path().file_name().and_then(|n| n.to_str()) == Some("pack.json")) {
        out.push(prefix.to_string());
        return; // don't recurse into packs
    }
    for sub in dir.dirs() {
        let name = sub.path().file_name().and_then(|n| n.to_str()).unwrap_or("");
        let child_prefix = format!("{}{}", prefix_path, name);
        collect_pack_paths_in(sub, &child_prefix, out);
    }
}

/// Recursively extract an embedded `include_dir::Dir` into a filesystem directory.
pub(super) fn extract_embedded_dir(dir: &include_dir::Dir<'_>, dest: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dest)?;
    for file in dir.files() {
        let rel = file.path().file_name().unwrap_or(file.path().as_os_str());
        std::fs::write(dest.join(rel), file.contents())?;
    }
    for sub in dir.dirs() {
        let name = sub.path().file_name().unwrap_or(sub.path().as_os_str());
        extract_embedded_dir(sub, &dest.join(name))?;
    }
    Ok(())
}

/// Simple timestamp without a chrono dependency.
pub(super) fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| format!("{}", d.as_secs()))
        .unwrap_or_else(|_| "0".to_string())
}

/// Read registry URL from `.tsx/config.json`, falling back to localhost.
pub(super) fn read_registry_url(root: &Path) -> String {
    let config_path = root.join(".tsx").join("config.json");
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(url) = val["registry"]["url"].as_str() {
                return url.to_string();
            }
        }
    }
    "http://localhost:4200".to_string()
}

/// Minimal percent-encode for query string values (no external dep needed).
pub(super) fn urlencoding_simple(s: &str) -> String {
    s.chars().map(|c| match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
        ' ' => "+".to_string(),
        c => format!("%{:02X}", c as u32),
    }).collect()
}

/// Download URL to bytes using reqwest blocking.
pub(super) fn download_bytes(url: &str) -> Result<Vec<u8>, String> {
    download_bytes_authed(url, None)
}

/// Download URL to bytes, optionally with a Bearer token (e.g. GITHUB_TOKEN).
pub(super) fn download_bytes_authed(url: &str, token: Option<&str>) -> Result<Vec<u8>, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("tsx-cli/0.1")
        .build()
        .map_err(|e| e.to_string())?;
    let mut req = client.get(url);
    if let Some(tok) = token {
        req = req.header("Authorization", format!("Bearer {}", tok));
    }
    req.send()
        .map_err(|e| e.to_string())?
        .bytes()
        .map(|b| b.to_vec())
        .map_err(|e| e.to_string())
}
