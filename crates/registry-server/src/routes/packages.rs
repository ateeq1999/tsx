use axum::{
    body::Body,
    extract::{ConnectInfo, Multipart, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use crate::{
    db::{self, AuthUser, UpsertPkg, UpsertVersion},
    models::ApiError,
    AppState,
};

// ── Query / body types ────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListPackagesQuery {
    pub sort: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct DownloadStatsQuery {
    /// Number of days of history (default 7, max 90)
    #[serde(default = "default_days")]
    pub days: i64,
}
fn default_days() -> i64 { 7 }

#[derive(Deserialize)]
pub struct UpdatePackageBody {
    pub description: Option<String>,
}

// ── GET /v1/packages?sort=recent&limit=N ─────────────────────────────────────

pub async fn list_packages(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListPackagesQuery>,
) -> (StatusCode, Json<Value>) {
    let limit = params.limit.unwrap_or(12).min(50).max(1);
    match db::get_recent(&state.pool, limit).await {
        Ok(rows) => {
            let packages: Vec<crate::models::Package> = rows
                .into_iter()
                .map(|(pkg, latest)| pkg.into_package(latest))
                .collect();
            (StatusCode::OK, Json(serde_json::to_value(packages).unwrap()))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ApiError::new(e.to_string())).unwrap()),
        ),
    }
}

// ── GET /v1/packages/:name ────────────────────────────────────────────────────

pub async fn get_package(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> (StatusCode, Json<Value>) {
    let decoded = url_decode(&name);
    match db::get_package(&state.pool, &decoded).await {
        Err(e) => err500(e.to_string()),
        Ok(None) => err404(format!("Package '{}' not found", decoded)),
        Ok(Some(pkg)) => {
            let versions = db::get_versions(&state.pool, pkg.id).await.unwrap_or_default();
            let latest = versions.first().map(|v| v.version.clone()).unwrap_or_default();
            (StatusCode::OK, Json(serde_json::to_value(pkg.into_package(latest)).unwrap()))
        }
    }
}

// ── GET /v1/packages/:name/versions ──────────────────────────────────────────

pub async fn get_package_versions(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> (StatusCode, Json<Value>) {
    let decoded = url_decode(&name);
    match db::get_package(&state.pool, &decoded).await {
        Err(e) => err500(e.to_string()),
        Ok(None) => err404(format!("Package '{}' not found", decoded)),
        Ok(Some(pkg)) => {
            match db::get_versions(&state.pool, pkg.id).await {
                Err(e) => err500(e.to_string()),
                Ok(rows) => {
                    let versions: Vec<crate::models::PackageVersion> = rows
                        .into_iter()
                        .map(|v| crate::models::PackageVersion {
                            version: v.version,
                            published_at: v.published_at.to_rfc3339(),
                            download_count: v.download_count,
                        })
                        .collect();
                    (StatusCode::OK, Json(serde_json::to_value(versions).unwrap()))
                }
            }
        }
    }
}

// ── GET /v1/packages/:name/readme ─────────────────────────────────────────────

pub async fn get_readme(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Response {
    let decoded = url_decode(&name);
    match db::get_package(&state.pool, &decoded).await {
        Err(e) => err500(e.to_string()).into_response(),
        Ok(None) => err404(format!("Package '{}' not found", decoded)).into_response(),
        Ok(Some(pkg)) => {
            match pkg.readme {
                Some(readme) => {
                    let mut headers = HeaderMap::new();
                    headers.insert("Content-Type", "text/markdown; charset=utf-8".parse().unwrap());
                    (StatusCode::OK, headers, readme).into_response()
                }
                None => (StatusCode::NO_CONTENT, Body::empty()).into_response(),
            }
        }
    }
}

// ── PUT /v1/packages/:name/readme ─────────────────────────────────────────────

pub async fn update_readme(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    headers: HeaderMap,
    body: String,
) -> (StatusCode, Json<Value>) {
    let decoded = url_decode(&name);
    let auth = match extract_auth(&state, &headers).await {
        Ok(a) => a,
        Err(e) => return e,
    };

    match db::get_package(&state.pool, &decoded).await {
        Err(e) => err500(e.to_string()),
        Ok(None) => err404(format!("Package '{}' not found", decoded)),
        Ok(Some(pkg)) => {
            // Only the package author may update the readme
            if let Some(ref uid) = pkg.author_id {
                if auth.as_ref().map(|u| &u.user_id) != Some(uid) {
                    return err403("Only the package author may update the README");
                }
            }
            match db::update_readme(&state.pool, pkg.id, &body).await {
                Ok(_) => {
                    let _ = db::insert_audit(&state.pool, "update_readme", &decoded, None,
                        auth.as_ref().map(|u| u.user_id.as_str()),
                        auth.as_ref().map(|u| u.name.as_str()),
                        None, None).await;
                    (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
                }
                Err(e) => err500(e.to_string()),
            }
        }
    }
}

// ── PUT /v1/packages/:name ────────────────────────────────────────────────────

pub async fn update_package(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    headers: HeaderMap,
    Json(body): Json<UpdatePackageBody>,
) -> (StatusCode, Json<Value>) {
    let decoded = url_decode(&name);
    let auth = match extract_auth(&state, &headers).await {
        Ok(a) => a,
        Err(e) => return e,
    };

    match db::get_package(&state.pool, &decoded).await {
        Err(e) => err500(e.to_string()),
        Ok(None) => err404(format!("Package '{}' not found", decoded)),
        Ok(Some(pkg)) => {
            if let Some(ref uid) = pkg.author_id {
                if auth.as_ref().map(|u| &u.user_id) != Some(uid) {
                    return err403("Only the package author may update this package");
                }
            }
            if let Some(description) = body.description {
                if let Err(e) = db::update_description(&state.pool, pkg.id, &description).await {
                    return err500(e.to_string());
                }
                let _ = db::insert_audit(&state.pool, "update_meta", &decoded, None,
                    auth.as_ref().map(|u| u.user_id.as_str()),
                    auth.as_ref().map(|u| u.name.as_str()),
                    None, None).await;
            }
            (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
        }
    }
}

// ── DELETE /v1/packages/:name/versions/:version ───────────────────────────────

pub async fn yank_version(
    State(state): State<Arc<AppState>>,
    Path((name, version)): Path<(String, String)>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    let decoded = url_decode(&name);
    let auth = match extract_auth(&state, &headers).await {
        Ok(a) => a,
        Err(e) => return e,
    };

    match db::get_package(&state.pool, &decoded).await {
        Err(e) => err500(e.to_string()),
        Ok(None) => err404(format!("Package '{}' not found", decoded)),
        Ok(Some(pkg)) => {
            if let Some(ref uid) = pkg.author_id {
                if auth.as_ref().map(|u| &u.user_id) != Some(uid) {
                    return err403("Only the package author may yank versions");
                }
            }
            match db::yank_version(&state.pool, pkg.id, &version).await {
                Ok(true) => {
                    let _ = db::insert_audit(&state.pool, "yank", &decoded, Some(&version),
                        auth.as_ref().map(|u| u.user_id.as_str()),
                        auth.as_ref().map(|u| u.name.as_str()),
                        None, None).await;
                    (StatusCode::OK, Json(serde_json::json!({ "ok": true, "yanked": version })))
                }
                Ok(false) => err404(format!("Version {} not found for {}", version, decoded)),
                Err(e) => err500(e.to_string()),
            }
        }
    }
}

// ── DELETE /v1/packages/:name ─────────────────────────────────────────────────

pub async fn delete_package(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    let decoded = url_decode(&name);
    let auth = match extract_auth(&state, &headers).await {
        Ok(a) => a,
        Err(e) => return e,
    };

    match db::get_package(&state.pool, &decoded).await {
        Err(e) => err500(e.to_string()),
        Ok(None) => err404(format!("Package '{}' not found", decoded)),
        Ok(Some(pkg)) => {
            if let Some(ref uid) = pkg.author_id {
                if auth.as_ref().map(|u| &u.user_id) != Some(uid) {
                    return err403("Only the package author may delete this package");
                }
            }
            match db::delete_package(&state.pool, pkg.id).await {
                Ok(_) => {
                    let _ = db::insert_audit(&state.pool, "delete", &decoded, None,
                        auth.as_ref().map(|u| u.user_id.as_str()),
                        auth.as_ref().map(|u| u.name.as_str()),
                        None, None).await;
                    (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
                }
                Err(e) => err500(e.to_string()),
            }
        }
    }
}

// ── GET /v1/packages/:name/stats/downloads ────────────────────────────────────

pub async fn get_download_stats(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<DownloadStatsQuery>,
) -> (StatusCode, Json<Value>) {
    let decoded = url_decode(&name);
    let days = params.days.min(90).max(1);

    match db::get_package(&state.pool, &decoded).await {
        Err(e) => err500(e.to_string()),
        Ok(None) => err404(format!("Package '{}' not found", decoded)),
        Ok(Some(pkg)) => {
            match db::get_download_stats(&state.pool, pkg.id, days).await {
                Ok(stats) => (StatusCode::OK, Json(serde_json::to_value(stats).unwrap())),
                Err(e) => err500(e.to_string()),
            }
        }
    }
}

// ── GET /v1/packages/:name/:version/tarball ───────────────────────────────────

pub async fn download_tarball(
    State(state): State<Arc<AppState>>,
    Path((name, version)): Path<(String, String)>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    let decoded_name = url_decode(&name);
    let ua = headers.get("User-Agent").and_then(|v| v.to_str().ok()).map(String::from);
    let ip = addr.ip().to_string();

    let (pkg_id, pkg_slug, tarball_path) = match db::get_package(&state.pool, &decoded_name).await {
        Ok(Some(p)) => {
            match db::get_tarball_path(&state.pool, p.id, &version).await {
                Ok(Some((_, path))) => (p.id, p.slug, PathBuf::from(path)),
                Ok(None) => return err404(format!("Version {} not found for {}", version, decoded_name)).into_response(),
                Err(e) => return err500(e.to_string()).into_response(),
            }
        }
        Ok(None) => return err404(format!("Package '{}' not found", decoded_name)).into_response(),
        Err(e) => return err500(e.to_string()).into_response(),
    };

    let bytes = match tokio::fs::read(&tarball_path).await {
        Ok(b) => b,
        Err(e) => return err500(format!("Could not read tarball: {}", e)).into_response(),
    };

    // Increment counters (best-effort)
    if let Ok(Some((version_id, _))) = db::get_tarball_path(&state.pool, pkg_id, &version).await {
        let _ = db::increment_downloads(&state.pool, pkg_id, version_id, Some(&ip), ua.as_deref()).await;
    }

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert("Content-Type", "application/gzip".parse().unwrap());
    resp_headers.insert(
        "Content-Disposition",
        format!("attachment; filename=\"{}-{}.tar.gz\"", pkg_slug, version)
            .parse()
            .unwrap(),
    );
    resp_headers.insert("Content-Length", bytes.len().to_string().parse().unwrap());

    (StatusCode::OK, resp_headers, Body::from(bytes)).into_response()
}

// ── POST /v1/packages/publish ─────────────────────────────────────────────────

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
                Json(serde_json::to_value(ApiError::new("Rate limit exceeded: max 10 publishes per minute per IP")).unwrap()),
            );
        }
    }

    // --- Auth: static key OR better-auth session token ---
    let auth_user = match authenticate_publish(&state, &headers).await {
        Ok(u) => u,
        Err(e) => return e,
    };

    // --- Parse multipart fields ---
    let mut name = String::new();
    let mut version = String::new();
    let mut manifest_str = String::new();
    let mut tarball_bytes: Vec<u8> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("name")     => name = field.text().await.unwrap_or_default(),
            Some("version")  => version = field.text().await.unwrap_or_default(),
            Some("manifest") => manifest_str = field.text().await.unwrap_or_default(),
            Some("tarball")  => tarball_bytes = field.bytes().await.unwrap_or_default().to_vec(),
            _ => {}
        }
    }

    // --- Validate ---
    if name.is_empty() || version.is_empty() || manifest_str.is_empty() || tarball_bytes.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::to_value(ApiError::new("Missing required fields: name, version, manifest, tarball")).unwrap()),
        );
    }
    if semver::Version::parse(&version).is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::to_value(ApiError::new(format!("Invalid semver: '{}'", version))).unwrap()),
        );
    }
    let manifest: serde_json::Value = match serde_json::from_str(&manifest_str) {
        Ok(v) => v,
        Err(e) => return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::to_value(ApiError::new(format!("Invalid manifest JSON: {}", e))).unwrap()),
        ),
    };

    let slug = name.split('/').last().unwrap_or(&name).to_string();

    // --- Checksum ---
    let mut hasher = Sha256::new();
    hasher.update(&tarball_bytes);
    let checksum = hex::encode(hasher.finalize());

    // --- Store tarball ---
    let tarball_dir = state.data_dir.join("tarballs").join(&slug);
    if let Err(e) = tokio::fs::create_dir_all(&tarball_dir).await {
        return err500(format!("Failed to create tarball dir: {}", e));
    }
    let tarball_path = tarball_dir.join(format!("{}.tar.gz", version));
    if let Err(e) = tokio::fs::write(&tarball_path, &tarball_bytes).await {
        return err500(format!("Failed to write tarball: {}", e));
    }

    // --- Extract metadata ---
    let description = str_field(&manifest, "description");
    let license     = str_field(&manifest, "license").unwrap_or_else(|| "MIT".to_string());
    let tsx_min     = str_field(&manifest, "tsx_min").unwrap_or_else(|| "0.1.0".to_string());
    let lang   = arr_field(&manifest, "lang");
    let runtime= arr_field(&manifest, "runtime");
    let provides = arr_field(&manifest, "provides");
    let integrates: Vec<String> = manifest.get("integrates_with")
        .and_then(|v| v.as_object())
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();
    let tags = arr_field(&manifest, "tags");

    let author_id   = auth_user.as_ref().map(|u| u.user_id.clone());
    let author_name = auth_user.as_ref().map(|u| u.name.clone()).unwrap_or_default();

    // --- Persist ---
    let pkg_id = match db::upsert_package(&state.pool, &UpsertPkg {
        name: name.clone(), slug: slug.clone(),
        description: description.unwrap_or_default(),
        author_id, author_name: author_name.clone(),
        license, tsx_min, tags, lang, runtime, provides, integrates,
    }).await {
        Ok(id) => id,
        Err(e) => return err500(e.to_string()),
    };

    if let Err(e) = db::upsert_version(&state.pool, pkg_id, &UpsertVersion {
        version: version.clone(),
        manifest,
        checksum: checksum.clone(),
        size_bytes: tarball_bytes.len() as i64,
        tarball_path: tarball_path.to_string_lossy().to_string(),
    }).await {
        return err500(e.to_string());
    }

    let ip = addr.ip().to_string();
    let _ = db::insert_audit(&state.pool, "publish", &name, Some(&version),
        auth_user.as_ref().map(|u| u.user_id.as_str()),
        Some(&author_name), Some(&ip), None).await;

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "name": name,
            "version": version,
            "checksum": checksum,
            "tarball_url": format!("/v1/packages/{}/{}/tarball", url_encode(&name), version),
        })),
    )
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Authenticate a request for write operations.
/// Accepts either:
///   1. Static TSX_REGISTRY_API_KEY
///   2. better-auth session token (validated against PostgreSQL session table)
/// Returns Ok(None) when the endpoint is open (no API key configured).
async fn authenticate_publish(
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<Option<AuthUser>, (StatusCode, Json<Value>)> {
    let provided = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(String::from);

    // If no API key is configured, allow open access
    if state.api_key.is_none() {
        // Still try to look up session for author attribution
        if let Some(ref token) = provided {
            if let Ok(Some(user)) = db::validate_session_token(&state.pool, token).await {
                return Ok(Some(user));
            }
        }
        return Ok(None);
    }

    let token = provided.ok_or_else(|| (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::to_value(ApiError::new("Authorization header required")).unwrap()),
    ))?;

    // Check static API key first
    if let Some(ref key) = state.api_key {
        if token == *key {
            return Ok(None); // static key — no user identity
        }
    }

    // Try better-auth session token
    match db::validate_session_token(&state.pool, &token).await {
        Ok(Some(user)) => Ok(Some(user)),
        Ok(None) => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::to_value(ApiError::new("Invalid or expired token")).unwrap()),
        )),
        Err(e) => Err(err500(e.to_string())),
    }
}

/// Like authenticate_publish but also works for PUT/DELETE (same logic).
async fn extract_auth(
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<Option<AuthUser>, (StatusCode, Json<Value>)> {
    authenticate_publish(state, headers).await
}

fn str_field(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key).and_then(|v| v.as_str()).map(String::from)
}

fn arr_field(v: &serde_json::Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(|a| a.as_array())
        .map(|a| a.iter().filter_map(|e| e.as_str()).map(String::from).collect())
        .unwrap_or_default()
}

fn url_decode(s: &str) -> String {
    s.replace("%40", "@").replace("%2F", "/")
}

fn url_encode(s: &str) -> String {
    s.replace('@', "%40").replace('/', "%2F")
}

fn err500(msg: impl Into<String>) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::to_value(ApiError::new(msg)).unwrap()),
    )
}

fn err404(msg: impl Into<String>) -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::to_value(ApiError::new(msg)).unwrap()),
    )
}

fn err403(msg: impl Into<String>) -> (StatusCode, Json<Value>) {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::to_value(ApiError::new(msg)).unwrap()),
    )
}
