use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use std::sync::Arc;

use crate::{db, AppState};

// ── GET /v1/feed.xml ──────────────────────────────────────────────────────────

#[utoipa::path(
    get, path = "/v1/feed.xml",
    responses(
        (status = 200, description = "RSS 2.0 feed of the 20 most recently published packages"),
    ),
    tag = "packages"
)]
pub async fn rss_feed(State(state): State<Arc<AppState>>) -> Response {
    let rows = match db::get_recent(&state.pool, 20).await {
        Ok(r) => r,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Body::empty()).into_response(),
    };

    let items: String = rows
        .into_iter()
        .map(|(pkg, version)| {
            let pub_date = pkg.published_at.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
            let escaped_desc = xml_escape(&pkg.description);
            let escaped_name = xml_escape(&pkg.name);
            format!(
                "  <item>\n    <title>{escaped_name} v{version}</title>\n    \
                 <link>https://tsx-tsnv.onrender.com/packages/{name}</link>\n    \
                 <guid>https://tsx-tsnv.onrender.com/packages/{name}@{version}</guid>\n    \
                 <description>{escaped_desc}</description>\n    \
                 <pubDate>{pub_date}</pubDate>\n    \
                 <author>{author}</author>\n  </item>",
                name = xml_escape(&pkg.name),
                author = xml_escape(&pkg.author_name),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>tsx Registry — New Packages</title>
    <link>https://tsx-tsnv.onrender.com</link>
    <description>The 20 most recently published packages on the tsx registry.</description>
    <language>en-us</language>
    <atom:link href="https://tsx-tsnv.onrender.com/v1/feed.xml" rel="self" type="application/rss+xml" />
{items}
  </channel>
</rss>"#
    );

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/rss+xml; charset=utf-8".parse().unwrap());
    headers.insert("Cache-Control", "public, max-age=300".parse().unwrap());
    (StatusCode::OK, headers, xml).into_response()
}

// ── GET /v1/packages/:name/badge.svg ─────────────────────────────────────────

#[utoipa::path(
    get, path = "/v1/packages/{name}/badge.svg",
    params(("name" = String, Path, description = "Package name")),
    responses(
        (status = 302, description = "Redirect to shields.io SVG badge with live download count"),
        (status = 404, description = "Package not found"),
    ),
    tag = "packages"
)]
pub async fn download_badge(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Response {
    let decoded = crate::routes::packages::url_decode_pub(&name);
    match db::get_package(&state.pool, &decoded).await {
        Ok(Some(pkg)) => {
            let count = pkg.downloads;
            let label = if count == 1 { "1 install".to_string() } else { format!("{count} installs") };
            let encoded = label.replace(' ', "%20");
            let url = format!("https://img.shields.io/badge/tsx-{encoded}-blue?logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI+PHBhdGggZmlsbD0id2hpdGUiIGQ9Ik0xMiAyQzYuNDggMiAyIDYuNDggMiAxMnM0LjQ4IDEwIDEwIDEwIDEwLTQuNDggMTAtMTBTMTcuNTIgMiAxMiAyem0tMiAxNFY4bDYgNHoiLz48L3N2Zz4=");
            Redirect::permanent(&url).into_response()
        }
        _ => (StatusCode::NOT_FOUND, Body::empty()).into_response(),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
