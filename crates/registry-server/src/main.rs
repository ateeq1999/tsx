//! registry.tsx.dev — hosted package registry for the tsx CLI
//!
//! # API
//!
//! | Method | Path                                    | Description              |
//! |--------|-----------------------------------------|--------------------------|
//! | GET    | /health                                 | Health check             |
//! | GET    | /v1/stats                               | Aggregate stats          |
//! | GET    | /v1/search?q=&lang=&size=               | Search packages          |
//! | GET    | /v1/packages?sort=recent&limit=N        | Recent packages          |
//! | GET    | /v1/packages/:name                      | Package metadata         |
//! | GET    | /v1/packages/:name/versions             | Version list             |
//! | GET    | /v1/packages/:name/:version/tarball     | Download tarball         |
//! | POST   | /v1/packages/publish                    | Publish a package        |
//!
//! # Configuration (environment variables)
//!
//! | Variable               | Default              | Description                          |
//! |------------------------|----------------------|--------------------------------------|
//! | `PORT`                 | `8080`               | TCP port to listen on                |
//! | `DATA_DIR`             | `./data`             | Directory for SQLite DB + tarballs   |
//! | `TSX_REGISTRY_API_KEY` | *(none — open)*      | Bearer token required for publish    |
//!
//! # Running
//!
//! ```bash
//! cargo run -p tsx-registry
//! # or with config:
//! PORT=9090 DATA_DIR=/var/tsx-registry TSX_REGISTRY_API_KEY=secret cargo run -p tsx-registry
//! ```

mod db;
mod models;
mod routes;

use axum::{
    routing::{get, post},
    Router,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

/// Shared application state passed to all route handlers via `State<Arc<AppState>>`.
pub struct AppState {
    /// Mutex-wrapped because `rusqlite::Connection` is not `Sync`.
    pub db: std::sync::Mutex<Db>,
    pub data_dir: PathBuf,
    /// Optional API key for the publish endpoint. `None` → endpoint is open (dev mode).
    pub api_key: Option<String>,
    /// Per-IP rate limiter for the publish endpoint: (request_count, window_start).
    pub rate_limiter: std::sync::Mutex<std::collections::HashMap<std::net::IpAddr, (u32, std::time::Instant)>>,
}

use db::Db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "tsx_registry=info,tower_http=debug".into()),
        )
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let data_dir = PathBuf::from(
        std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string()),
    );
    tokio::fs::create_dir_all(&data_dir).await?;
    tokio::fs::create_dir_all(data_dir.join("tarballs")).await?;

    let db_path = data_dir.join("registry.db");
    let db = Db::open(&db_path)?;

    let api_key = std::env::var("TSX_REGISTRY_API_KEY").ok();
    if api_key.is_none() {
        tracing::warn!(
            "TSX_REGISTRY_API_KEY is not set — publish endpoint is open to anyone"
        );
    }

    let state = Arc::new(AppState {
        db: std::sync::Mutex::new(db),
        data_dir,
        api_key,
        rate_limiter: std::sync::Mutex::new(std::collections::HashMap::new()),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // ── Health ──────────────────────────────────────────────────────────
        .route("/health", get(routes::health::health))
        // ── Stats ───────────────────────────────────────────────────────────
        .route("/v1/stats", get(routes::stats::get_stats))
        // ── Search ──────────────────────────────────────────────────────────
        .route("/v1/search", get(routes::search::search))
        // ── Packages ────────────────────────────────────────────────────────
        .route("/v1/packages", get(routes::packages::list_packages))
        .route(
            "/v1/packages/publish",
            post(routes::packages::publish),
        )
        .route(
            "/v1/packages/:name",
            get(routes::packages::get_package),
        )
        .route(
            "/v1/packages/:name/versions",
            get(routes::packages::get_package_versions),
        )
        .route(
            "/v1/packages/:name/:version/tarball",
            get(routes::packages::download_tarball),
        )
        .with_state(state)
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("registry.tsx.dev listening on http://{}", addr);
    info!("  GET  /health");
    info!("  GET  /v1/stats");
    info!("  GET  /v1/search?q=<query>&lang=<lang>");
    info!("  GET  /v1/packages?sort=recent&limit=N");
    info!("  GET  /v1/packages/:name");
    info!("  GET  /v1/packages/:name/versions");
    info!("  GET  /v1/packages/:name/:version/tarball");
    info!("  POST /v1/packages/publish  (multipart: name, version, manifest, tarball)");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}
