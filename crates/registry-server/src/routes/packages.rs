use axum::{
    body::Body,
    extract::{ConnectInfo, Multipart, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use std::net::SocketAddr;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{path::PathBuf, sync::Arc};

use crate::{
    db::{UpsertPkg, UpsertVersion},
    models::{ApiError, ApiResponse, PackageMeta, VersionMeta},
    AppState,
};

/// GET /v1/packages/:name
///
/// Returns full metadata for a package. `:name` can be the scoped npm name
/// (`@tsx-pkg/drizzle-pg`) or the bare slug (`drizzle-pg`).
pub async fn get_package(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> (StatusCode, Json<Value>) {
    let decoded = urlencoding_decode(&name);
    let db = state.db.lock().unwrap();

    match db.get_package(&decoded) {
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ApiError::new(e.to_string())).unwrap()),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(
                serde_json::to_value(ApiError::new(format!(
                    "Package '{}' not found",
                    decoded
                )))
                .unwrap(),
            ),
        ),
        Ok(Some(pkg)) => {
            let version_rows = db.get_versions(pkg.id).unwrap_or_default();
            let manifest: Option<serde_json::Value> = version_rows
                .first()
                .and_then(|v| serde_json::from_str(&v.manifest).ok());

            let versions = version_rows
                .into_iter()
                .map(|v| VersionMeta {
                    tarball_url: format!(
                        "/v1/packages/{}/{}/tarball",
                        urlencoding_encode(&pkg.name),
                        v.version
                    ),
                    version: v.version,
                    checksum: v.checksum,
                    size_bytes: v.size_bytes,
                    published_at: v.published_at,
                })
                .collect::<Vec<_>>();

            let latest = versions
                .first()
                .map(|v| v.version.clone())
                .unwrap_or_default();

            let lang = pkg.lang_vec();
            let runtime = pkg.runtime_vec();
            let provides = pkg.provides_vec();
            let integrates_with = pkg.integrates_vec();
            let meta = PackageMeta {
                name: pkg.name,
                description: pkg.description,
                latest_version: latest,
                lang,
                runtime,
                provides,
                integrates_with,
                downloads: pkg.downloads,
                published_at: pkg.published_at,
                updated_at: pkg.updated_at,
                versions,
                manifest,
            };

            (
                StatusCode::OK,
                Json(serde_json::to_value(ApiResponse::ok(meta)).unwrap()),
            )
        }
    }
}

/// GET /v1/packages/:name/:version/tarball
///
/// Streams the package tarball to the client and increments the download counter.
pub async fn download_tarball(
    State(state): State<Arc<AppState>>,
    Path((name, version)): Path<(String, String)>,
) -> Response {
    let decoded_name = urlencoding_decode(&name);

    let (pkg_slug, tarball_path) = {
        let db = state.db.lock().unwrap();
        let pkg = match db.get_package(&decoded_name) {
            Ok(Some(p)) => p,
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::to_value(ApiError::new("Package not found")).unwrap()),
                )
                    .into_response()
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::to_value(ApiError::new(e.to_string())).unwrap()),
                )
                    .into_response()
            }
        };

        let path = match db.get_tarball_path(pkg.id, &version) {
            Ok(Some(p)) => PathBuf::from(p),
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(
                        serde_json::to_value(ApiError::new(format!(
                            "Version {} not found for package {}",
                            version, decoded_name
                        )))
                        .unwrap(),
                    ),
                )
                    .into_response()
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::to_value(ApiError::new(e.to_string())).unwrap()),
                )
                    .into_response()
            }
        };
        (pkg.slug, path)
    };

    let tarball_path = tarball_path;
    let bytes = match tokio::fs::read(&tarball_path).await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::to_value(ApiError::new(format!("Could not read tarball: {}", e)))
                        .unwrap(),
                ),
            )
                .into_response()
        }
    };

    // Increment download counter (best-effort)
    let _ = state.db.lock().unwrap().increment_downloads(&pkg_slug);

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/gzip".parse().unwrap(),
    );
    headers.insert(
        "Content-Disposition",
        format!("attachment; filename=\"{}-{}.tar.gz\"", pkg_slug, version)
            .parse()
            .unwrap(),
    );
    headers.insert(
        "Content-Length",
        bytes.len().to_string().parse().unwrap(),
    );

    (StatusCode::OK, headers, Body::from(bytes)).into_response()
}

/// POST /v1/packages/publish
///
/// Accepts multipart form with fields:
///   - `name`     — scoped package name (e.g. @tsx-pkg/drizzle-pg)
///   - `version`  — semver string
///   - `manifest` — JSON-encoded manifest.json content
///   - `tarball`  — .tar.gz file bytes
///
/// Requires `Authorization: Bearer <API_KEY>` header (checked against
/// `TSX_REGISTRY_API_KEY` environment variable).
pub async fn publish(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> (StatusCode, Json<Value>) {
    // --- Rate limiting: 10 req/min per IP ---
    {
        let ip = addr.ip();
        let mut limiter = state.rate_limiter.lock().unwrap();
        let now = std::time::Instant::now();
        let entry = limiter.entry(ip).or_insert((0, now));
        if now.duration_since(entry.1) >= std::time::Duration::from_secs(60) {
            *entry = (0, now);
        }
        entry.0 += 1;
        if entry.0 > 10 {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(
                    serde_json::to_value(ApiError::new(
                        "Rate limit exceeded: max 10 publishes per minute per IP",
                    ))
                    .unwrap(),
                ),
            );
        }
    }

    // --- Auth ---
    if let Some(expected_key) = &state.api_key {
        let provided = headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        if provided.map(|k| k != expected_key.as_str()).unwrap_or(true) {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::to_value(ApiError::new("Invalid or missing API key")).unwrap()),
            );
        }
    }

    // --- Parse multipart fields ---
    let mut name = String::new();
    let mut version = String::new();
    let mut manifest_str = String::new();
    let mut tarball_bytes: Vec<u8> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("name") => {
                name = field.text().await.unwrap_or_default();
            }
            Some("version") => {
                version = field.text().await.unwrap_or_default();
            }
            Some("manifest") => {
                manifest_str = field.text().await.unwrap_or_default();
            }
            Some("tarball") => {
                tarball_bytes = field.bytes().await.unwrap_or_default().to_vec();
            }
            _ => {}
        }
    }

    // --- Validate ---
    if name.is_empty() || version.is_empty() || manifest_str.is_empty() || tarball_bytes.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::to_value(ApiError::new(
                    "Missing required fields: name, version, manifest, tarball",
                ))
                .unwrap(),
            ),
        );
    }

    if semver::Version::parse(&version).is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::to_value(ApiError::new(format!(
                    "Invalid semver: '{}'",
                    version
                )))
                .unwrap(),
            ),
        );
    }

    let manifest: serde_json::Value = match serde_json::from_str(&manifest_str) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    serde_json::to_value(ApiError::new(format!(
                        "Invalid manifest JSON: {}",
                        e
                    )))
                    .unwrap(),
                ),
            )
        }
    };

    // --- Derive slug from name (@tsx-pkg/drizzle-pg → drizzle-pg) ---
    let slug = name
        .split('/')
        .last()
        .unwrap_or(&name)
        .to_string();

    // --- Compute checksum ---
    let mut hasher = Sha256::new();
    hasher.update(&tarball_bytes);
    let checksum = hex::encode(hasher.finalize());

    // --- Store tarball ---
    let tarball_dir = state.data_dir.join("tarballs").join(&slug);
    if let Err(e) = tokio::fs::create_dir_all(&tarball_dir).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                serde_json::to_value(ApiError::new(format!(
                    "Failed to create tarball dir: {}",
                    e
                )))
                .unwrap(),
            ),
        );
    }
    let tarball_path = tarball_dir.join(format!("{}.tar.gz", version));
    if let Err(e) = tokio::fs::write(&tarball_path, &tarball_bytes).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                serde_json::to_value(ApiError::new(format!(
                    "Failed to write tarball: {}",
                    e
                )))
                .unwrap(),
            ),
        );
    }

    // --- Extract metadata from manifest ---
    let description = manifest
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let lang = json_string_vec(&manifest, "lang");
    let runtime = json_string_vec(&manifest, "runtime");
    let provides = json_string_vec(&manifest, "provides");
    let integrates: Vec<String> = manifest
        .get("integrates_with")
        .and_then(|v| v.as_object())
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();

    // --- Persist to DB ---
    let db = state.db.lock().unwrap();
    let pkg_id = match db.upsert_package(&UpsertPkg {
        name: name.clone(),
        slug: slug.clone(),
        description,
        lang,
        runtime,
        provides,
        integrates,
    }) {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::to_value(ApiError::new(e.to_string())).unwrap()),
            )
        }
    };

    if let Err(e) = db.upsert_version(
        pkg_id,
        &UpsertVersion {
            version: version.clone(),
            manifest: manifest_str,
            checksum: checksum.clone(),
            size_bytes: tarball_bytes.len() as u64,
            tarball_path: tarball_path.to_string_lossy().to_string(),
        },
    ) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ApiError::new(e.to_string())).unwrap()),
        );
    }

    (
        StatusCode::CREATED,
        Json(
            serde_json::to_value(ApiResponse::ok(serde_json::json!({
                "name":     name,
                "version":  version,
                "checksum": checksum,
                "tarball_url": format!("/v1/packages/{}/{}/tarball",
                    urlencoding_encode(&name), version),
            })))
            .unwrap(),
        ),
    )
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn json_string_vec(v: &serde_json::Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn urlencoding_decode(s: &str) -> String {
    // Minimal percent-decode for @ and /
    s.replace("%40", "@").replace("%2F", "/")
}

fn urlencoding_encode(s: &str) -> String {
    s.replace('@', "%40").replace('/', "%2F")
}
