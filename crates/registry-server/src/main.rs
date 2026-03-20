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
//! | GET    | /v1/packages/{name}                           | Package metadata               |
//! | GET    | /v1/packages/{name}/versions                  | Version list                   |
//! | GET    | /v1/packages/{name}/readme                    | README markdown                |
//! | GET    | /v1/packages/{name}/stats/downloads           | Per-day download stats         |
//! | GET    | /v1/packages/{name}/{version}/tarball         | Download tarball               |
//! | POST   | /v1/packages/publish                          | Publish a package              |
//! | PUT    | /v1/packages/{name}                           | Update description/metadata    |
//! | PUT    | /v1/packages/{name}/readme                    | Update README                  |
//! | DELETE | /v1/packages/{name}/versions/{version}        | Yank a version                 |
//! | DELETE | /v1/packages/{name}                           | Delete a package               |
//! | GET    | /v1/admin/audit-log                           | Publish audit log              |
//! | GET    | /v1/admin/rate-limits                         | Rate limit status per IP       |
//! | GET    | /api-docs/openapi.json                        | OpenAPI 3.1 spec (JSON)        |
//! | POST   | /v1/webhooks                                  | Register a webhook             |
//! | GET    | /v1/webhooks                                  | List your webhooks             |
//! | DELETE | /v1/webhooks/{id}                             | Delete a webhook               |
//!
//! # Environment variables
//!
//! | Variable               | Required | Description                                      |
//! |------------------------|----------|--------------------------------------------------|
//! | `DATABASE_URL`         | yes      | Neon PostgreSQL connection string                |
//! | `TSX_REGISTRY_API_KEY` | no       | Bearer token for admin + publish (open if unset) |
//! | `DATA_DIR`             | no       | Tarball storage path (default `./data`)          |
//! | `PORT`                 | no       | Listen port (default 8080, Railway sets this)    |

mod db;
mod routes;

// Re-export shared API types so route handlers can use `crate::models::*`
// without breaking existing import paths.
pub use tsx_shared as models;

// ── OpenAPI spec ──────────────────────────────────────────────────────────────

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "tsx Registry API",
        version = "0.1.0",
        description = "Hosted package registry for the tsx CLI — browse, publish, and install reusable code patterns.",
        license(name = "MIT"),
    ),
    paths(
        routes::health::health,
        routes::auth_route::whoami,
        routes::stats::get_stats,
        routes::search::search,
        routes::packages::list_packages,
        routes::packages::get_package,
        routes::packages::get_package_versions,
        routes::packages::get_readme,
        routes::packages::update_readme,
        routes::packages::get_download_stats,
        routes::packages::download_tarball,
        routes::packages::publish,
        routes::packages::update_package,
        routes::packages::yank_version,
        routes::packages::delete_package,
        routes::users::get_user_packages,
        routes::admin::get_audit_log,
        routes::admin::get_rate_limits,
        routes::webhooks::create_webhook,
        routes::webhooks::list_webhooks,
        routes::webhooks::delete_webhook,
    ),
    components(schemas(
        models::Package,
        models::PackageVersion,
        models::SearchResult,
        models::RegistryStats,
        models::DailyDownloads,
        models::AuditEntry,
        models::RateLimitEntry,
        models::PublishResult,
        models::ApiError,
        db::Webhook,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "meta",     description = "Health check and registry statistics"),
        (name = "packages", description = "Browse, search, publish, and manage packages"),
        (name = "admin",    description = "Admin endpoints (require API key)"),
        (name = "webhooks", description = "Webhook subscriptions for package events"),
    )
)]
struct ApiDoc;

struct SecurityAddon;
impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = &mut openapi.components {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .build(),
                ),
            );
        }
    }
}

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use utoipa::OpenApi as _;
use sqlx::postgres::PgPoolOptions;
use std::{
    path::PathBuf,
    sync::Arc,
};
use tokio::net::TcpListener;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

/// Shared application state passed to all route handlers.
pub struct AppState {
    /// PostgreSQL connection pool (Neon)
    pub pool: sqlx::PgPool,
    pub data_dir: PathBuf,
    /// Optional static API key for admin endpoints. `None` → open (dev mode).
    pub api_key: Option<String>,
}

#[tokio::main]
async fn main() {
    // ── Logging ────────────────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tsx_registry=info,tower_http=info".into()),
        )
        .json()
        .init();

    // ── Config from environment ────────────────────────────────────────────
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let data_dir = PathBuf::from(
        std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string()),
    );

    let api_key = std::env::var("TSX_REGISTRY_API_KEY").ok();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    // ── Storage directories ────────────────────────────────────────────────
    tokio::fs::create_dir_all(&data_dir).await
        .expect("Failed to create DATA_DIR");
    tokio::fs::create_dir_all(data_dir.join("tarballs")).await
        .expect("Failed to create DATA_DIR/tarballs");

    // ── Database pool ──────────────────────────────────────────────────────
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    info!("Connected to PostgreSQL (Neon)");

    // Run forward-only SQL migrations at startup
    db::run_migrations(&pool).await
        .expect("Failed to apply database migrations");

    info!("Database migrations applied");

    if api_key.is_none() {
        tracing::warn!("TSX_REGISTRY_API_KEY is not set — publish endpoint is open to anyone");
    }

    // ── Application state ──────────────────────────────────────────────────
    let state = Arc::new(AppState {
        pool,
        data_dir,
        api_key,
    });

    // ── Background: download-log aggregation ───────────────────────────────
    // Runs every 24 hours. Prunes download_logs older than 90 days to keep
    // the table from growing unboundedly.  The aggregated counts are already
    // stored on the packages and versions rows so no data is lost.
    {
        let bg_pool = state.pool.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(24 * 3600));
            loop {
                interval.tick().await;
                match sqlx::query(
                    "DELETE FROM download_logs WHERE downloaded_at < NOW() - INTERVAL '90 days'",
                )
                .execute(&bg_pool)
                .await
                {
                    Ok(r) => {
                        if r.rows_affected() > 0 {
                            tracing::info!(
                                rows = r.rows_affected(),
                                "Pruned old download_logs rows"
                            );
                        }
                    }
                    Err(e) => tracing::warn!(error = %e, "download_logs prune failed"),
                }
            }
        });
    }

    // ── Middleware ─────────────────────────────────────────────────────────
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // ── Router ─────────────────────────────────────────────────────────────
    let app = Router::new()
        // Health
        .route("/health", get(routes::health::health))
        // Stats
        .route("/v1/stats", get(routes::stats::get_stats))
        // Search
        .route("/v1/search", get(routes::search::search))
        // Packages
        .route("/v1/packages",              get(routes::packages::list_packages))
        .route("/v1/packages/publish",      post(routes::packages::publish))
        .route("/v1/packages/{name}",        get(routes::packages::get_package))
        .route("/v1/packages/{name}",        put(routes::packages::update_package))
        .route("/v1/packages/{name}",        delete(routes::packages::delete_package))
        .route("/v1/packages/{name}/versions",
            get(routes::packages::get_package_versions))
        .route("/v1/packages/{name}/readme",
            get(routes::packages::get_readme).put(routes::packages::update_readme))
        .route("/v1/packages/{name}/stats/downloads",
            get(routes::packages::get_download_stats))
        .route("/v1/packages/{name}/{version}/tarball",
            get(routes::packages::download_tarball))
        .route("/v1/packages/{name}/versions/{version}",
            delete(routes::packages::yank_version))
        // Webhooks
        .route("/v1/webhooks",      post(routes::webhooks::create_webhook))
        .route("/v1/webhooks",       get(routes::webhooks::list_webhooks))
        .route("/v1/webhooks/{id}", delete(routes::webhooks::delete_webhook))
        // Auth helpers (used by CLI login)
        .route("/v1/auth/whoami", get(routes::auth_route::whoami))
        // User profiles
        .route("/v1/users/{name}/packages", get(routes::users::get_user_packages))
        // Admin
        .route("/v1/admin/audit-log",   get(routes::admin::get_audit_log))
        .route("/v1/admin/rate-limits", get(routes::admin::get_rate_limits))
        // OpenAPI spec
        .route("/api-docs/openapi.json", get(|| async {
            axum::response::Json(ApiDoc::openapi())
        }))
        .with_state(state)
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    // ── Listener ───────────────────────────────────────────────────────────
    // Bind to IPv6 all-interfaces ([::]) which also accepts IPv4 on Railway.
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await
        .expect("Failed to bind TCP listener");

    info!(%addr, "Registry server listening");
    axum::serve(listener, app).await
        .expect("Server error");
}
