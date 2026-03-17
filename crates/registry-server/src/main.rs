//! registry.tsx.dev — hosted package registry for the tsx CLI
//!
//! # API
//!
//! | Method | Path                                          | Description                    |
//! |--------|-----------------------------------------------|--------------------------------|
//! | GET    | /health                                       | Health check                   |
//! | GET    | /v1/stats                                     | Aggregate stats                |
//! | GET    | /v1/search?q=&lang=&sort=&page=&size=         | Search packages (paginated)    |
//! | GET    | /v1/packages?sort=recent&limit=N              | Recent packages                |
//! | GET    | /v1/packages/:name                            | Package metadata               |
//! | GET    | /v1/packages/:name/versions                   | Version list                   |
//! | GET    | /v1/packages/:name/readme                     | README markdown                |
//! | GET    | /v1/packages/:name/stats/downloads            | Per-day download stats         |
//! | GET    | /v1/packages/:name/:version/tarball           | Download tarball               |
//! | POST   | /v1/packages/publish                          | Publish a package              |
//! | PUT    | /v1/packages/:name                            | Update description/metadata    |
//! | PUT    | /v1/packages/:name/readme                     | Update README                  |
//! | DELETE | /v1/packages/:name/versions/:version          | Yank a version                 |
//! | DELETE | /v1/packages/:name                            | Delete a package               |
//! | GET    | /v1/admin/audit-log                           | Publish audit log              |
//! | GET    | /v1/admin/rate-limits                         | Rate limit status per IP       |
//!
//! # Configuration (environment variables)
//!
//! | Variable               | Default                                      | Description                           |
//! |------------------------|----------------------------------------------|---------------------------------------|
//! | `DATABASE_URL`         | *(required)*                                 | PostgreSQL connection string          |
//! | `PORT`                 | `8080`                                       | TCP port to listen on                 |
//! | `DATA_DIR`             | `./data`                                     | Directory for tarball storage         |
//! | `TSX_REGISTRY_API_KEY` | *(none — open)*                              | Bearer token for publish (optional)   |

mod db;
mod routes;

// Re-export shared API types so route handlers can use `crate::models::*`
// without breaking existing import paths.
pub use tsx_shared as models;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
    time::Instant,
};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

/// Shared application state passed to all route handlers.
pub struct AppState {
    /// PostgreSQL connection pool
    pub pool: sqlx::PgPool,
    pub data_dir: PathBuf,
    /// Optional static API key for the publish endpoint. `None` → open (dev mode).
    pub api_key: Option<String>,
    /// Per-IP rate limiter: ip → (request_count, window_start)
    pub rate_limiter: std::sync::Mutex<HashMap<std::net::IpAddr, (u32, Instant)>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env from current dir or parent
    let _ = dotenvy::dotenv();

    let filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "tsx_registry=info,tower_http=debug".into());
    if std::env::var("LOG_FORMAT").as_deref() == Ok("json") {
        tracing_subscriber::fmt().json().with_env_filter(filter).init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (postgresql://user:pass@host/tsx_db)");

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let data_dir = PathBuf::from(
        std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string()),
    );
    tokio::fs::create_dir_all(&data_dir).await?;
    tokio::fs::create_dir_all(data_dir.join("tarballs")).await?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    info!("Connected to PostgreSQL at {}", &database_url);

    // Run schema migrations
    db::run_migrations(&pool).await?;
    info!("Database migrations applied");

    let api_key = std::env::var("TSX_REGISTRY_API_KEY").ok();
    if api_key.is_none() {
        tracing::warn!("TSX_REGISTRY_API_KEY is not set — publish endpoint is open to anyone");
    }

    let state = Arc::new(AppState {
        pool,
        data_dir,
        api_key,
        rate_limiter: std::sync::Mutex::new(HashMap::new()),
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
        .route("/v1/packages/publish", post(routes::packages::publish))
        .route("/v1/packages/:name", get(routes::packages::get_package))
        .route("/v1/packages/:name", put(routes::packages::update_package))
        .route("/v1/packages/:name", delete(routes::packages::delete_package))
        .route("/v1/packages/:name/versions", get(routes::packages::get_package_versions))
        .route("/v1/packages/:name/readme", get(routes::packages::get_readme))
        .route("/v1/packages/:name/readme", put(routes::packages::update_readme))
        .route("/v1/packages/:name/stats/downloads", get(routes::packages::get_download_stats))
        .route("/v1/packages/:name/:version/tarball", get(routes::packages::download_tarball))
        .route("/v1/packages/:name/versions/:version", delete(routes::packages::yank_version))
        // ── Admin ────────────────────────────────────────────────────────────
        .route("/v1/admin/audit-log", get(routes::admin::get_audit_log))
        .route("/v1/admin/rate-limits", get(routes::admin::get_rate_limits))
        .with_state(state)
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("registry.tsx.dev listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}
