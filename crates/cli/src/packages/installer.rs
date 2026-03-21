//! Package installer — downloads a tarball from the registry and extracts it
//! into the global package cache.

use std::path::PathBuf;

use tsx_shared::PackageSummary;

use crate::utils::paths::get_global_packages_dir;

/// Download and install a package from the registry server.
///
/// `registry_url` — base URL of the registry (e.g. `https://registry.tsx.dev`)
/// `package_id`   — slug, e.g. `tanstack-start`
/// `version`      — optional semver; `None` downloads the latest
pub fn install(
    registry_url: &str,
    package_id: &str,
    version: Option<&str>,
) -> anyhow::Result<PackageSummary> {
    let version_segment = version.unwrap_or("latest");
    let url = if version_segment == "latest" {
        format!("{}/v1/packages/{}/download", registry_url, package_id)
    } else {
        format!("{}/v1/packages/{}/{}/download", registry_url, package_id, version_segment)
    };

    // Download tarball bytes
    let bytes = fetch_bytes(&url)?;

    // Extract into a temp dir then move to global cache
    let tmp = std::env::temp_dir().join(format!("tsx-install-{}", package_id));
    if tmp.exists() { std::fs::remove_dir_all(&tmp)?; }
    std::fs::create_dir_all(&tmp)?;

    extract_tgz(&bytes, &tmp)?;

    // The tarball should contain a single top-level directory — find it.
    let pkg_src = find_package_root(&tmp)?;

    let summary = crate::packages::store::PackageStore::install_from_dir(&pkg_src)?;
    let _ = std::fs::remove_dir_all(&tmp);
    Ok(summary)
}

/// Install a package from a local directory (dev workflow).
pub fn install_local(src_dir: &std::path::Path) -> anyhow::Result<PackageSummary> {
    crate::packages::store::PackageStore::install_from_dir(src_dir)
}

/// Pack a package directory into a `.tgz` tarball.
pub fn pack(pkg_dir: &std::path::Path, out_path: &std::path::Path) -> anyhow::Result<()> {
    use std::io::BufWriter;
    let file = std::fs::File::create(out_path)?;
    let enc = flate2::write::GzEncoder::new(BufWriter::new(file), flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all("package", pkg_dir)?;
    tar.finish()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fetch_bytes(url: &str) -> anyhow::Result<Vec<u8>> {
    // Use ureq for a synchronous HTTP request (no async runtime needed in the CLI).
    let response = ureq::get(url)
        .call()
        .map_err(|e| anyhow::anyhow!("Failed to download {}: {}", url, e))?;

    let mut bytes = Vec::new();
    use std::io::Read;
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))?;
    Ok(bytes)
}

fn extract_tgz(bytes: &[u8], dest: &std::path::Path) -> anyhow::Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;
    let gz = GzDecoder::new(bytes);
    let mut archive = Archive::new(gz);
    archive.unpack(dest)?;
    Ok(())
}

fn find_package_root(extracted: &std::path::Path) -> anyhow::Result<PathBuf> {
    // If manifest.json is directly in extracted/, return that.
    if extracted.join("manifest.json").exists() {
        return Ok(extracted.to_path_buf());
    }
    // Otherwise look for a single subdirectory containing manifest.json.
    if let Ok(entries) = std::fs::read_dir(extracted) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() && p.join("manifest.json").exists() {
                return Ok(p);
            }
        }
    }
    anyhow::bail!(
        "Could not find manifest.json in extracted package at {}",
        extracted.display()
    )
}

/// Returns the install path for a given package id.
pub fn install_path(package_id: &str) -> PathBuf {
    get_global_packages_dir().join(package_id)
}
