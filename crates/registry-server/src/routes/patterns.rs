use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{io::Read, sync::Arc};

use crate::{
    db::{self, patterns as pdb},
    models::ApiError,
    AppState,
};

// ── Constants ─────────────────────────────────────────────────────────────────

const MAX_LIST_LIMIT: i64 = 50;
const DEFAULT_LIST_LIMIT: i64 = 12;
const MAX_TARBALL_BYTES: usize = 50 * 1024 * 1024; // 50 MB

// ── Query types ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListPatternsQuery {
    pub limit: Option<i64>,
    pub framework: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_search_limit")]
    pub limit: i64,
}
fn default_search_limit() -> i64 { 20 }

// ── Response helpers ──────────────────────────────────────────────────────────

fn pattern_row_to_json(p: &pdb::PatternRow) -> Value {
    json!({
        "slug":           p.slug,
        "name":           p.name,
        "version":        p.version,
        "description":    p.description,
        "author":         p.author_name,
        "framework":      p.framework,
        "tags":           p.tags,
        "download_count": p.download_count,
        "published_at":   p.published_at.to_rfc3339(),
        "updated_at":     p.updated_at.to_rfc3339(),
    })
}

fn err(status: StatusCode, msg: impl ToString) -> (StatusCode, Json<Value>) {
    (status, Json(json!({ "error": msg.to_string() })))
}

// ── GET /v1/patterns ──────────────────────────────────────────────────────────

pub async fn list_patterns(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListPatternsQuery>,
) -> (StatusCode, Json<Value>) {
    let limit = params.limit.unwrap_or(DEFAULT_LIST_LIMIT).min(MAX_LIST_LIMIT).max(1);
    match pdb::list_patterns(&state.pool, limit, params.framework.as_deref()).await {
        Ok(rows) => {
            let items: Vec<Value> = rows.iter().map(pattern_row_to_json).collect();
            (StatusCode::OK, Json(json!({ "count": items.len(), "patterns": items })))
        }
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

// ── GET /v1/patterns/search?q=<query> ─────────────────────────────────────────

pub async fn search_patterns(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> (StatusCode, Json<Value>) {
    match pdb::search_patterns(&state.pool, &params.q, params.limit).await {
        Ok(rows) => {
            let items: Vec<Value> = rows.iter().map(pattern_row_to_json).collect();
            (StatusCode::OK, Json(json!({ "count": items.len(), "patterns": items })))
        }
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

// ── GET /v1/patterns/:slug ────────────────────────────────────────────────────

pub async fn get_pattern(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> (StatusCode, Json<Value>) {
    let decoded = urlencoding::decode(&slug).unwrap_or(std::borrow::Cow::Borrowed(&slug)).into_owned();
    match pdb::get_pattern(&state.pool, &decoded).await {
        Ok(Some(p)) => {
            let versions = pdb::list_pattern_versions(&state.pool, p.id).await.unwrap_or_default();
            let mut val = pattern_row_to_json(&p);
            val["versions"] = json!(versions.iter().map(|v| json!({
                "version":      v.version,
                "size_bytes":   v.size_bytes,
                "published_at": v.published_at.to_rfc3339(),
            })).collect::<Vec<_>>());
            val["readme"] = json!(p.readme);
            (StatusCode::OK, Json(val))
        }
        Ok(None) => err(StatusCode::NOT_FOUND, format!("Pattern '{}' not found", decoded)),
        Err(e)   => err(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

// ── GET /v1/patterns/:slug/:version/tarball ───────────────────────────────────

pub async fn download_tarball(
    State(state): State<Arc<AppState>>,
    Path((slug, version)): Path<(String, String)>,
) -> Response {
    let decoded = urlencoding::decode(&slug).unwrap_or(std::borrow::Cow::Borrowed(&slug)).into_owned();
    let pattern = match pdb::get_pattern(&state.pool, &decoded).await {
        Ok(Some(p)) => p,
        Ok(None)    => return (StatusCode::NOT_FOUND, Json(json!({"error": "Pattern not found"}))).into_response(),
        Err(e)      => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    let ver = match pdb::get_pattern_version(&state.pool, pattern.id, &version).await {
        Ok(Some(v)) => v,
        Ok(None)    => return (StatusCode::NOT_FOUND, Json(json!({"error": "Version not found"}))).into_response(),
        Err(e)      => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    let tarball_path = state.data_dir.join("pattern-tarballs").join(&ver.tarball_path);
    match tokio::fs::read(&tarball_path).await {
        Ok(bytes) => {
            pdb::increment_pattern_downloads(&state.pool, pattern.id).await;
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/gzip")
                .header("Content-Disposition", format!("attachment; filename=\"{}-{}.tar.gz\"", decoded.replace('/', "-"), version))
                .body(Body::from(bytes))
                .unwrap()
        }
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({"error": "Tarball file not found on disk"}))).into_response(),
    }
}

// ── POST /v1/patterns/publish ─────────────────────────────────────────────────

pub async fn publish_pattern(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> (StatusCode, Json<Value>) {
    // Optional API-key auth
    if let Some(required) = &state.api_key {
        let token = headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "));
        if token != Some(required.as_str()) {
            return err(StatusCode::UNAUTHORIZED, "Invalid or missing API key");
        }
    }

    let mut tarball_bytes: Option<Vec<u8>> = None;
    let mut manifest_json: Option<Value> = None;
    let mut author_name = String::new();
    let mut readme: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("tarball") => {
                let bytes = field.bytes().await.unwrap_or_default();
                if bytes.len() > MAX_TARBALL_BYTES {
                    return err(StatusCode::PAYLOAD_TOO_LARGE, "Tarball exceeds 50 MB limit");
                }
                tarball_bytes = Some(bytes.to_vec());
            }
            Some("manifest") => {
                let text = field.text().await.unwrap_or_default();
                manifest_json = serde_json::from_str(&text).ok();
            }
            Some("author") => {
                author_name = field.text().await.unwrap_or_default();
            }
            Some("readme") => {
                readme = Some(field.text().await.unwrap_or_default());
            }
            _ => {}
        }
    }

    let tarball = match tarball_bytes {
        Some(b) => b,
        None => return err(StatusCode::BAD_REQUEST, "Missing 'tarball' field"),
    };
    let manifest = match manifest_json {
        Some(m) => m,
        None => return err(StatusCode::BAD_REQUEST, "Missing or invalid 'manifest' field"),
    };

    // Extract required fields from manifest
    let pack_id = manifest["id"].as_str().unwrap_or("").to_string();
    let pack_name = manifest["name"].as_str().unwrap_or(&pack_id).to_string();
    let pack_version = manifest["version"].as_str().unwrap_or("1.0.0").to_string();
    let pack_desc = manifest["description"].as_str().unwrap_or("").to_string();
    let pack_framework = manifest["framework"].as_str().unwrap_or("").to_string();
    let pack_tags: Vec<String> = manifest["tags"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let pack_author = if author_name.is_empty() {
        manifest["author"].as_str().unwrap_or("anonymous").to_string()
    } else {
        author_name
    };

    if pack_id.is_empty() {
        return err(StatusCode::BAD_REQUEST, "Manifest missing 'id' field");
    }

    // Build slug: "author/id"
    let slug = if pack_id.contains('/') {
        pack_id.clone()
    } else {
        format!("{}/{}", pack_author, pack_id)
    };

    // Compute checksum
    let checksum = format!("{:x}", Sha256::digest(&tarball));
    let size_bytes = tarball.len() as i64;

    // Validate tarball is a valid gzip
    {
        let gz = GzDecoder::new(std::io::Cursor::new(&tarball));
        let mut archive = tar::Archive::new(gz);
        if archive.entries().is_err() {
            return err(StatusCode::BAD_REQUEST, "Tarball is not a valid .tar.gz");
        }
    }

    // Save tarball to disk
    let tarball_dir = state.data_dir.join("pattern-tarballs");
    if let Err(e) = tokio::fs::create_dir_all(&tarball_dir).await {
        return err(StatusCode::INTERNAL_SERVER_ERROR, format!("Storage error: {e}"));
    }
    let tarball_filename = format!("{}-{}.tar.gz", slug.replace('/', "-"), pack_version);
    let tarball_path = tarball_dir.join(&tarball_filename);
    if let Err(e) = tokio::fs::write(&tarball_path, &tarball).await {
        return err(StatusCode::INTERNAL_SERVER_ERROR, format!("Write error: {e}"));
    }

    // Upsert pattern row
    let pattern = match pdb::upsert_pattern(&state.pool, pdb::UpsertPattern {
        slug: slug.clone(),
        author_id: None,
        author_name: pack_author.clone(),
        name: pack_name,
        version: pack_version.clone(),
        description: pack_desc,
        framework: pack_framework,
        tags: pack_tags,
        tarball_path: tarball_filename.clone(),
        checksum: checksum.clone(),
        readme,
    }).await {
        Ok(p) => p,
        Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, e),
    };

    // Upsert version row
    let _ = pdb::upsert_pattern_version(&state.pool, pattern.id, pdb::UpsertPatternVersion {
        version: pack_version.clone(),
        tarball_path: tarball_filename,
        checksum,
        size_bytes,
        manifest,
    }).await;

    (StatusCode::OK, Json(json!({
        "slug":    slug,
        "version": pack_version,
        "status":  "published",
    })))
}
