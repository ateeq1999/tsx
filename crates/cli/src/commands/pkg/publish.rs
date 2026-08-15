use crate::commands::manage::auth::load_credentials;
use crate::output::CommandResult;

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
