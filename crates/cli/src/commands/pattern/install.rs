use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

use super::utils::{
    chrono_now, collect_builtin_pack_paths, download_bytes, download_bytes_authed,
    extract_embedded_dir, read_registry_url, urlencoding_simple, BUILTIN_PACKS,
};

/// Install a pack from a local path or `github:user/repo[#subpath][@ref]`.
pub fn pattern_install(source: String, id_override: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if source.starts_with("github:") {
        pattern_install_github(&source, id_override, &cwd)
    } else if source.starts_with('@') {
        pattern_install_registry(&source, id_override, &cwd)
    } else if source.starts_with("builtin:") {
        pattern_install_builtin(source.trim_start_matches("builtin:"), id_override, &cwd)
    } else {
        pattern_install_local(PathBuf::from(&source), id_override, &cwd)
    }
}

pub(super) fn pattern_install_local(src: PathBuf, id_override: Option<String>, root: &Path) -> ResponseEnvelope {
    let Some(pack) = forge::PackManifest::load_from_dir(&src) else {
        return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(
                ErrorCode::ValidationError,
                format!("No valid pack.json found in {}", src.display()),
            ),
            0,
        );
    };

    let id = id_override.unwrap_or_else(|| pack.id.clone());
    let dest = forge::PackManifest::dir(root, &id);

    if let Err(e) = copy_dir_all(&src, &dest) {
        return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        );
    }

    let source_meta = forge::PackSource {
        kind: "local".to_string(),
        source: src.to_string_lossy().to_string(),
        ref_: String::new(),
        installed_at: chrono_now(),
    };
    let _ = source_meta.save(root, &id);

    ResponseEnvelope::success(
        "pattern install",
        serde_json::json!({
            "id": id,
            "version": pack.version,
            "source": "local",
            "path": dest.to_string_lossy(),
        }),
        0,
    )
}

pub(super) fn pattern_install_github(source: &str, id_override: Option<String>, root: &Path) -> ResponseEnvelope {
    // Parse: github:user/repo[#sub/path[@ref]]
    let spec = source.trim_start_matches("github:");

    // Split #subpath first
    let (repo_and_ref, subpath_raw) = if let Some(hash) = spec.find('#') {
        (&spec[..hash], Some(&spec[hash + 1..]))
    } else {
        (spec, None)
    };

    // Split @ref from repo
    let (repo, git_ref) = if let Some(at) = repo_and_ref.rfind('@') {
        (&repo_and_ref[..at], &repo_and_ref[at + 1..])
    } else {
        (repo_and_ref, "HEAD")
    };

    // Split @ref from subpath if present
    let (subpath, git_ref) = match subpath_raw {
        Some(s) => {
            if let Some(at) = s.rfind('@') {
                (Some(&s[..at]), &s[at + 1..])
            } else {
                (Some(s), git_ref)
            }
        }
        None => (None, git_ref),
    };

    let tarball_url = format!("https://api.github.com/repos/{}/tarball/{}", repo, git_ref);

    // Download tarball into a temp dir
    let tmp_dir = match tempfile_dir() {
        Ok(d) => d,
        Err(e) => return ResponseEnvelope::error("pattern install", ErrorResponse::new(ErrorCode::InternalError, e), 0),
    };

    let github_token = std::env::var("GITHUB_TOKEN").ok();
    let tarball_bytes = match download_bytes_authed(&tarball_url, github_token.as_deref()) {
        Ok(b) => b,
        Err(e) => return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::InternalError, format!("Download failed: {e}")),
            0,
        ),
    };

    // Extract tarball
    let gz = flate2::read::GzDecoder::new(std::io::Cursor::new(&tarball_bytes));
    let mut archive = tar::Archive::new(gz);
    if let Err(e) = archive.unpack(&tmp_dir) {
        return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::InternalError, format!("Extract failed: {e}")),
            0,
        );
    }

    // GitHub tarballs extract into a single top-level directory like `user-repo-<sha>/`
    let extracted_root = match std::fs::read_dir(&tmp_dir)
        .ok()
        .and_then(|mut e| e.next())
        .and_then(|e| e.ok())
        .map(|e| e.path())
    {
        Some(p) => p,
        None => return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::InternalError, "Tarball appears empty"),
            0,
        ),
    };

    let pack_src = match subpath {
        Some(s) => extracted_root.join(s),
        None => extracted_root,
    };

    pattern_install_local(pack_src, id_override, root)
}

/// Install a built-in pack (embedded in binary) into `.tsx/patterns/<id>/`.
///
/// The `pack_path` is the relative path within the embedded `patterns/` dir,
/// e.g. `"tanstack-start/todo-with-auth"`.
fn pattern_install_builtin(pack_path: &str, id_override: Option<String>, root: &Path) -> ResponseEnvelope {
    let norm = pack_path.replace('\\', "/");
    let pack_dir = BUILTIN_PACKS.get_dir(&norm);

    let Some(dir) = pack_dir else {
        let available = collect_builtin_pack_paths().join(", ");
        return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(
                ErrorCode::ProjectNotFound,
                format!("Built-in pack '{}' not found. Available: {}", norm, available),
            ),
            0,
        );
    };

    // Parse pack.json from embedded bytes
    let manifest_bytes = match dir.get_file(format!("{}/pack.json", norm))
        .or_else(|| dir.get_file("pack.json"))
    {
        Some(f) => f.contents(),
        None => return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::ValidationError, format!("Built-in pack '{}' has no pack.json", norm)),
            0,
        ),
    };
    let pack: forge::PackManifest = match serde_json::from_slice(manifest_bytes) {
        Ok(p) => p,
        Err(e) => return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::InternalError, format!("Malformed pack.json: {e}")),
            0,
        ),
    };

    let id = id_override.unwrap_or_else(|| pack.id.clone());
    let dest = forge::PackManifest::dir(root, &id);
    if let Err(e) = extract_embedded_dir(dir, &dest) {
        return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::InternalError, format!("Extract failed: {e}")),
            0,
        );
    }

    let source_meta = forge::PackSource {
        kind: "builtin".to_string(),
        source: format!("builtin:{}", norm),
        ref_: pack.version.clone(),
        installed_at: chrono_now(),
    };
    let _ = source_meta.save(root, &id);

    ResponseEnvelope::success(
        "pattern install",
        serde_json::json!({
            "id": id,
            "version": pack.version,
            "source": "builtin",
            "path": dest.to_string_lossy(),
        }),
        0,
    )
}

pub(super) fn pattern_install_registry(source: &str, id_override: Option<String>, root: &Path) -> ResponseEnvelope {
    // Parse: @scope/name[@version] or scope/name[@version]
    let spec = source.trim_start_matches('@');
    let (slug, version) = if let Some(at) = spec.rfind('@') {
        (spec[..at].to_string(), Some(spec[at + 1..].to_string()))
    } else {
        (spec.to_string(), None)
    };

    let registry_url = read_registry_url(root);

    // GET metadata
    let meta_url = format!("{}/v1/patterns/{}", registry_url.trim_end_matches('/'), urlencoding_simple(&slug));
    let meta_bytes = match download_bytes(&meta_url) {
        Ok(b) => b,
        Err(e) => return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::InternalError, format!("Registry error: {e}")),
            0,
        ),
    };
    let meta: serde_json::Value = match serde_json::from_slice(&meta_bytes) {
        Ok(v) => v,
        Err(e) => return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::InternalError, format!("Parse error: {e}")),
            0,
        ),
    };

    let ver = version
        .or_else(|| meta["version"].as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "latest".to_string());

    // Download tarball
    let tarball_url = format!(
        "{}/v1/patterns/{}/{}/tarball",
        registry_url.trim_end_matches('/'),
        urlencoding_simple(&slug),
        urlencoding_simple(&ver),
    );
    let tarball_bytes = match download_bytes(&tarball_url) {
        Ok(b) => b,
        Err(e) => return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::InternalError, format!("Download failed: {e}")),
            0,
        ),
    };

    // Verify SHA256 checksum if provided
    if let Some(expected) = meta["checksum"].as_str().filter(|s| !s.is_empty()) {
        let actual = format!("{:x}", Sha256::digest(&tarball_bytes));
        if actual != expected {
            return ResponseEnvelope::error(
                "pattern install",
                ErrorResponse::new(
                    ErrorCode::ValidationError,
                    format!("Checksum mismatch: expected {expected}, got {actual}"),
                ),
                0,
            );
        }
    }

    // Extract into temp dir
    let tmp_dir = match tempfile_dir() {
        Ok(d) => d,
        Err(e) => return ResponseEnvelope::error("pattern install", ErrorResponse::new(ErrorCode::InternalError, e), 0),
    };
    let gz = flate2::read::GzDecoder::new(std::io::Cursor::new(&tarball_bytes));
    let mut archive = tar::Archive::new(gz);
    if let Err(e) = archive.unpack(&tmp_dir) {
        return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::InternalError, format!("Extract failed: {e}")),
            0,
        );
    }

    // Descend into single top-level dir if present (registry tarballs vary)
    let pack_src = {
        let entries: Vec<_> = std::fs::read_dir(&tmp_dir)
            .ok().into_iter().flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();
        if entries.len() == 1 { entries[0].path() } else { tmp_dir.clone() }
    };

    let Some(pack) = forge::PackManifest::load_from_dir(&pack_src) else {
        return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::ValidationError, "No valid pack.json found in registry tarball"),
            0,
        );
    };

    let id = id_override.unwrap_or_else(|| pack.id.clone());
    let dest = forge::PackManifest::dir(root, &id);
    if let Err(e) = copy_dir_all(&pack_src, &dest) {
        return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        );
    }

    let source_meta = forge::PackSource {
        kind: "registry".to_string(),
        source: source.to_string(),
        ref_: ver.clone(),
        installed_at: chrono_now(),
    };
    let _ = source_meta.save(root, &id);

    ResponseEnvelope::success(
        "pattern install",
        serde_json::json!({
            "id": id,
            "version": ver,
            "source": "registry",
            "registry": registry_url,
            "path": dest.to_string_lossy(),
        }),
        0,
    )
}

/// Recursively copy `src` directory into `dst`.
fn copy_dir_all(src: &Path, dst: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in walkdir::WalkDir::new(src).min_depth(1) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(src)?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Create a unique temporary directory under the system temp path.
fn tempfile_dir() -> Result<PathBuf, String> {
    let base = std::env::temp_dir().join(format!("tsx-install-{}", chrono_now()));
    std::fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    Ok(base)
}
