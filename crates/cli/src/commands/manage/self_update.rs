use crate::output::CommandResult;

const GITHUB_RELEASES_API: &str =
    "https://api.github.com/repos/ateeq1999/tsx/releases/latest";

/// `tsx upgrade cli [--check]`
///
/// Checks GitHub Releases for a newer tsx binary and, unless `--check` is
/// given, downloads and replaces the running executable.
pub fn self_update(check_only: bool) -> CommandResult {
    let current = env!("CARGO_PKG_VERSION");

    // ── Fetch latest release from GitHub ──────────────────────────────────────
    let client = match reqwest::blocking::Client::builder()
        .user_agent(format!("tsx-cli/{current}"))
        .build()
    {
        Ok(c) => c,
        Err(e) => return CommandResult::err("upgrade cli", format!("Failed to build HTTP client: {e}")),
    };

    let release: serde_json::Value = match client.get(GITHUB_RELEASES_API).send()
        .and_then(|r| r.json())
    {
        Ok(v) => v,
        Err(e) => return CommandResult::err(
            "upgrade cli",
            format!("Failed to fetch release info from GitHub: {e}"),
        ),
    };

    let latest_tag = match release.get("tag_name").and_then(|v| v.as_str()) {
        Some(t) => t.trim_start_matches('v'),
        None => return CommandResult::err("upgrade cli", "Could not read tag_name from GitHub API response"),
    };

    // ── Version comparison ─────────────────────────────────────────────────────
    let (cur_ver, lat_ver) = match (
        semver::Version::parse(current),
        semver::Version::parse(latest_tag),
    ) {
        (Ok(c), Ok(l)) => (c, l),
        _ => return CommandResult::err(
            "upgrade cli",
            format!("Could not parse versions: current={current} latest={latest_tag}"),
        ),
    };

    if lat_ver <= cur_ver {
        let mut result = CommandResult::ok("upgrade cli", vec![]);
        result.next_steps = vec![format!("tsx is already up to date (v{current})")];
        return result;
    }

    if check_only {
        let mut result = CommandResult::ok("upgrade cli", vec![]);
        result.next_steps = vec![
            format!("New version available: v{latest_tag} (you have v{current})"),
            "Run `tsx upgrade cli` to install it.".to_string(),
        ];
        return result;
    }

    // ── Determine platform asset name ──────────────────────────────────────────
    let asset_name = platform_asset_name(latest_tag);

    let asset_url = match find_asset_url(&release, &asset_name) {
        Some(u) => u,
        None => return CommandResult::err(
            "upgrade cli",
            format!("No release asset found for this platform ({asset_name}). Visit https://github.com/ateeq1999/tsx/releases to download manually."),
        ),
    };

    // ── Download binary ────────────────────────────────────────────────────────
    let bytes = match client.get(&asset_url).send().and_then(|r| {
        r.error_for_status()?.bytes().map(|b| b.to_vec())
    }) {
        Ok(b) => b,
        Err(e) => return CommandResult::err("upgrade cli", format!("Download failed: {e}")),
    };

    // ── Replace running executable ─────────────────────────────────────────────
    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => return CommandResult::err("upgrade cli", format!("Cannot locate current executable: {e}")),
    };

    // Write to a temp file first, then atomically rename.
    let tmp_path = exe_path.with_extension("tmp_upgrade");
    if let Err(e) = std::fs::write(&tmp_path, &bytes) {
        return CommandResult::err("upgrade cli", format!("Failed to write temporary binary: {e}"));
    }

    // Make executable on Unix.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&tmp_path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(&tmp_path, perms);
        }
    }

    if let Err(e) = std::fs::rename(&tmp_path, &exe_path) {
        let _ = std::fs::remove_file(&tmp_path);
        return CommandResult::err(
            "upgrade cli",
            format!("Failed to replace binary (try running with elevated permissions): {e}"),
        );
    }

    let mut result = CommandResult::ok("upgrade cli", vec![format!("Updated tsx to v{latest_tag}")]);
    result.next_steps = vec![format!("tsx has been updated from v{current} to v{latest_tag}")];
    result
}

fn platform_asset_name(version: &str) -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let (os_str, arch_str, ext) = match (os, arch) {
        ("linux",   "x86_64")  => ("linux",   "x86_64",  "tar.gz"),
        ("linux",   "aarch64") => ("linux",   "aarch64", "tar.gz"),
        ("macos",   "x86_64")  => ("macos",   "x86_64",  "tar.gz"),
        ("macos",   "aarch64") => ("macos",   "aarch64", "tar.gz"),
        ("windows", "x86_64")  => ("windows", "x86_64",  "zip"),
        _ => (os, arch, "tar.gz"),
    };

    format!("tsx-v{version}-{arch_str}-{os_str}.{ext}")
}

fn find_asset_url(release: &serde_json::Value, name: &str) -> Option<String> {
    release
        .get("assets")?
        .as_array()?
        .iter()
        .find(|a| a.get("name").and_then(|n| n.as_str()) == Some(name))
        .and_then(|a| a.get("browser_download_url"))
        .and_then(|u| u.as_str())
        .map(String::from)
}
