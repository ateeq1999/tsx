/// Registry discovery and community sharing for tsx framework registries.
///
/// `tsx registry search <query>`  — search npm for tsx-framework-* packages
/// `tsx registry install <pkg>`   — install a community registry into .tsx/frameworks/
/// `tsx registry list`            — list installed community registries
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;
use crate::utils::paths::find_project_root;

/// Metadata stored in `.tsx/registries.json` tracking installed community registries.
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct InstalledRegistry {
    pub slug: String,
    pub package: String,
    pub version: String,
    pub source: String,
    pub installed_at: String,
}

fn registries_index_path(root: &Path) -> PathBuf {
    root.join(".tsx").join("registries.json")
}

fn load_registries_index(root: &Path) -> Vec<InstalledRegistry> {
    let path = registries_index_path(root);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_registries_index(root: &Path, registries: &[InstalledRegistry]) -> anyhow::Result<()> {
    let path = registries_index_path(root);
    std::fs::create_dir_all(path.parent().unwrap())?;
    let content = serde_json::to_string_pretty(registries)?;
    std::fs::write(&path, content)?;
    Ok(())
}

fn iso_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    let year = 1970 + days / 365;
    let doy = days % 365 + 1;
    let month = (doy / 30).min(11) + 1;
    let day = doy % 30 + 1;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, h, m, s)
}

/// Search npm for `tsx-framework-*` packages matching a query.
pub fn registry_search(query: String, verbose: bool) -> CommandResult {
    let start = Instant::now();

    // npm registry search API — returns up to 20 results
    let search_text = if query.trim().is_empty() {
        "tsx-framework".to_string()
    } else {
        format!("tsx-framework {}", query)
    };

    let url = format!(
        "https://registry.npmjs.org/-/v1/search?text={}&size=20",
        urlencoding(&search_text)
    );

    let result = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .and_then(|c| c.get(&url).header("Accept", "application/json").send())
        .and_then(|r| r.json::<serde_json::Value>());

    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(json) => {
            let objects = json
                .get("objects")
                .and_then(|o| o.as_array())
                .cloned()
                .unwrap_or_default();

            let packages: Vec<serde_json::Value> = objects
                .iter()
                .filter_map(|o| {
                    let pkg = o.get("package")?;
                    Some(serde_json::json!({
                        "name": pkg.get("name")?.as_str()?,
                        "version": pkg.get("version")?.as_str().unwrap_or("?"),
                        "description": pkg.get("description").and_then(|d| d.as_str()).unwrap_or(""),
                        "publisher": pkg.get("publisher").and_then(|p| p.get("username")).and_then(|u| u.as_str()).unwrap_or(""),
                    }))
                })
                .collect();

            let response = ResponseEnvelope::success(
                "registry:search",
                serde_json::json!({ "query": query, "results": packages }),
                duration_ms,
            );
            if verbose {
                let context = crate::json::response::Context {
                    project_root: std::env::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    tsx_version: env!("CARGO_PKG_VERSION").to_string(),
                };
                response.with_context(context).print();
            } else {
                response.print();
            }
            CommandResult::ok("registry:search", vec![])
        }
        Err(e) => {
            let error = ErrorResponse::new(
                ErrorCode::InternalError,
                format!("npm search failed: {}", e),
            );
            ResponseEnvelope::error("registry:search", error, duration_ms).print();
            CommandResult::err("registry:search", e.to_string())
        }
    }
}

/// Install a community registry from an npm package into `.tsx/frameworks/<slug>/`.
///
/// The package must:
/// 1. Exist on npm
/// 2. Have a `registry.json` in its package root (fetched via the npm `dist.tarball` URL)
///
/// The installed registry is tracked in `.tsx/registries.json`.
pub fn registry_install(package: String, verbose: bool) -> CommandResult {
    let start = Instant::now();

    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(ErrorCode::ProjectNotFound, e.to_string());
            ResponseEnvelope::error("registry:install", error, duration_ms).print();
            return CommandResult::err("registry:install", e.to_string());
        }
    };

    // Fetch package metadata from npm registry
    let npm_url = format!(
        "https://registry.npmjs.org/{}",
        package.replace("/", "%2F")
    );

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return CommandResult::err("registry:install", e.to_string());
        }
    };

    let pkg_meta: serde_json::Value = match client
        .get(&npm_url)
        .header("Accept", "application/json")
        .send()
        .and_then(|r| r.json())
    {
        Ok(v) => v,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(
                ErrorCode::InternalError,
                format!("Failed to fetch package '{}' from npm: {}", package, e),
            );
            ResponseEnvelope::error("registry:install", error, duration_ms).print();
            return CommandResult::err("registry:install", e.to_string());
        }
    };

    let latest = pkg_meta
        .get("dist-tags")
        .and_then(|t| t.get("latest"))
        .and_then(|v| v.as_str())
        .unwrap_or("latest");

    // Try the direct registry.json URL convention: unpkg.com/<pkg>/registry.json
    let registry_url = format!("https://unpkg.com/{}/registry.json", package);

    let registry_json: serde_json::Value = match client
        .get(&registry_url)
        .header("Accept", "application/json")
        .send()
        .and_then(|r| r.json())
    {
        Ok(v) => v,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(
                ErrorCode::InternalError,
                format!(
                    "Package '{}' does not expose a registry.json via unpkg: {}",
                    package, e
                ),
            );
            ResponseEnvelope::error("registry:install", error, duration_ms).print();
            return CommandResult::err("registry:install", e.to_string());
        }
    };

    // Extract slug from registry.json
    let slug = match registry_json.get("slug").and_then(|s| s.as_str()) {
        Some(s) => s.to_string(),
        None => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(
                ErrorCode::ValidationError,
                "registry.json is missing required 'slug' field",
            );
            ResponseEnvelope::error("registry:install", error, duration_ms).print();
            return CommandResult::err("registry:install", "missing slug".to_string());
        }
    };

    // Write registry.json to .tsx/frameworks/<slug>/
    let dest_dir = root.join(".tsx").join("frameworks").join(&slug);
    std::fs::create_dir_all(&dest_dir).ok();
    let dest_file = dest_dir.join("registry.json");

    let registry_content = serde_json::to_string_pretty(&registry_json)
        .unwrap_or_else(|_| "{}".to_string());

    if let Err(e) = std::fs::write(&dest_file, &registry_content) {
        let duration_ms = start.elapsed().as_millis() as u64;
        let error = ErrorResponse::new(
            ErrorCode::PermissionDenied,
            format!("Failed to write registry: {}", e),
        );
        ResponseEnvelope::error("registry:install", error, duration_ms).print();
        return CommandResult::err("registry:install", e.to_string());
    }

    // Update registries index
    let mut index = load_registries_index(&root);
    index.retain(|r| r.slug != slug); // remove old entry if re-installing
    index.push(InstalledRegistry {
        slug: slug.clone(),
        package: package.clone(),
        version: latest.to_string(),
        source: registry_url,
        installed_at: iso_now(),
    });
    let _ = save_registries_index(&root, &index);

    let duration_ms = start.elapsed().as_millis() as u64;
    let response = ResponseEnvelope::success(
        "registry:install",
        serde_json::json!({
            "installed": {
                "slug": slug,
                "package": package,
                "version": latest,
            }
        }),
        duration_ms,
    );

    if verbose {
        let context = crate::json::response::Context {
            project_root: root.to_string_lossy().to_string(),
            tsx_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        response.with_context(context).print();
    } else {
        response.print();
    }

    CommandResult::ok("registry:install", vec![dest_file.to_string_lossy().to_string()])
}

/// List community registries installed in `.tsx/registries.json`.
pub fn registry_list(verbose: bool) -> CommandResult {
    let start = Instant::now();

    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(ErrorCode::ProjectNotFound, e.to_string());
            ResponseEnvelope::error("registry:list", error, duration_ms).print();
            return CommandResult::err("registry:list", e.to_string());
        }
    };

    let registries = load_registries_index(&root);
    let duration_ms = start.elapsed().as_millis() as u64;

    let response = ResponseEnvelope::success(
        "registry:list",
        serde_json::json!({ "registries": registries }),
        duration_ms,
    );

    if verbose {
        let context = crate::json::response::Context {
            project_root: root.to_string_lossy().to_string(),
            tsx_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        response.with_context(context).print();
    } else {
        response.print();
    }

    CommandResult::ok("registry:list", vec![])
}

/// Generate a static HTML registry website listing all built-in and installed community registries.
///
/// `tsx registry website --output <dir>`
///
/// Produces `<dir>/index.html` — a self-contained, zero-dependency HTML page that
/// catalogs every framework registry: name, category, version, docs link, integrations,
/// and conventions. Suitable for committing to a `gh-pages` branch or opening locally.
pub fn registry_website(output_dir: String, verbose: bool) -> CommandResult {
    use crate::framework::loader::FrameworkLoader;
    use crate::framework::registry::FrameworkRegistry;

    let start = Instant::now();
    let out_path = std::path::Path::new(&output_dir);

    if let Err(e) = std::fs::create_dir_all(out_path) {
        return CommandResult::err("registry:website", format!("Cannot create output dir: {}", e));
    }

    // Load built-in registries
    let mut loader = FrameworkLoader::default();
    loader.load_builtin_frameworks();

    // Also scan frameworks/ directory directly for full registry data
    let frameworks_dir = std::path::Path::new("frameworks");
    let mut registries: Vec<FrameworkRegistry> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(frameworks_dir) {
        for entry in entries.flatten() {
            let reg_path = entry.path().join("registry.json");
            if let Ok(content) = std::fs::read_to_string(&reg_path) {
                if let Ok(reg) = serde_json::from_str::<FrameworkRegistry>(&content) {
                    registries.push(reg);
                }
            }
        }
    }

    // Also load community registries from .tsx/frameworks/ if in a project
    if let Ok(root) = find_project_root() {
        let community_dir = root.join(".tsx").join("frameworks");
        if let Ok(entries) = std::fs::read_dir(&community_dir) {
            for entry in entries.flatten() {
                let reg_path = entry.path().join("registry.json");
                if let Ok(content) = std::fs::read_to_string(&reg_path) {
                    if let Ok(reg) = serde_json::from_str::<FrameworkRegistry>(&content) {
                        // Only add if not already present
                        if !registries.iter().any(|r| r.slug == reg.slug) {
                            registries.push(reg);
                        }
                    }
                }
            }
        }
    }

    registries.sort_by(|a, b| a.slug.cmp(&b.slug));

    let html = generate_registry_html(&registries);
    let index_path = out_path.join("index.html");

    if let Err(e) = std::fs::write(&index_path, &html) {
        return CommandResult::err(
            "registry:website",
            format!("Cannot write index.html: {}", e),
        );
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let response = ResponseEnvelope::success(
        "registry:website",
        serde_json::json!({
            "output": index_path.to_string_lossy(),
            "registries": registries.len(),
        }),
        duration_ms,
    );

    if verbose {
        let context = crate::json::response::Context {
            project_root: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            tsx_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        response.with_context(context).print();
    } else {
        response.print();
    }

    CommandResult::ok(
        "registry:website",
        vec![index_path.to_string_lossy().to_string()],
    )
}

fn generate_registry_html(registries: &[crate::framework::registry::FrameworkRegistry]) -> String {
    use crate::framework::registry::FrameworkCategory;

    let category_badge = |cat: &FrameworkCategory| -> &'static str {
        match cat {
            FrameworkCategory::Framework => "framework",
            FrameworkCategory::Orm => "orm",
            FrameworkCategory::Auth => "auth",
            FrameworkCategory::Ui => "ui",
            FrameworkCategory::Tool => "tool",
        }
    };

    let cards: String = registries
        .iter()
        .map(|r| {
            let badge = category_badge(&r.category);
            let integrations_list: String = r
                .integrations
                .iter()
                .map(|i| {
                    let install = i.install.as_deref().unwrap_or("");
                    format!(
                        "<li><code>{}</code>{}</li>",
                        html_escape(&i.package),
                        if install.is_empty() {
                            String::new()
                        } else {
                            format!(" — <code>{}</code>", html_escape(install))
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            let patterns_list: String = r
                .conventions
                .patterns
                .iter()
                .map(|p| {
                    format!(
                        "<li><strong>{}</strong>{}</li>",
                        html_escape(&p.name),
                        p.description.as_deref().map(|d| format!(" — {}", html_escape(d))).unwrap_or_default(),
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            let github_link = r
                .github
                .as_deref()
                .map(|url| {
                    format!(
                        r#" <a class="gh-link" href="{}" target="_blank" rel="noopener">GitHub ↗</a>"#,
                        html_escape(url)
                    )
                })
                .unwrap_or_default();

            format!(
                r#"<article class="card" id="{slug}">
  <header>
    <h2>{name} <span class="badge badge-{badge}">{badge}</span></h2>
    <div class="meta">v{version} · <a href="{docs}" target="_blank" rel="noopener">Docs ↗</a>{github_link}</div>
  </header>
  {integrations_section}
  {patterns_section}
</article>"#,
                slug = html_escape(&r.slug),
                name = html_escape(&r.framework),
                badge = badge,
                version = html_escape(&r.version),
                docs = html_escape(&r.docs),
                github_link = github_link,
                integrations_section = if r.integrations.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<section><h3>Integrations</h3><ul>{}</ul></section>",
                        integrations_list
                    )
                },
                patterns_section = if r.conventions.patterns.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<section><h3>Patterns</h3><ul>{}</ul></section>",
                        patterns_list
                    )
                },
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let nav_links: String = registries
        .iter()
        .map(|r| {
            format!(
                r##"<a href="#{}">{}</a>"##,
                html_escape(&r.slug),
                html_escape(&r.framework)
            )
        })
        .collect::<Vec<_>>()
        .join(" · ");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>TSX Framework Registry</title>
<style>
  :root {{ --bg:#0f1117; --surface:#1a1d27; --border:#2d3148; --text:#e2e8f0; --muted:#94a3b8; --accent:#6366f1; --green:#22c55e; --yellow:#f59e0b; --red:#ef4444; }}
  * {{ box-sizing:border-box; margin:0; padding:0; }}
  body {{ background:var(--bg); color:var(--text); font:16px/1.6 system-ui,sans-serif; padding:2rem; }}
  header.site {{ max-width:900px; margin:0 auto 2rem; border-bottom:1px solid var(--border); padding-bottom:1.5rem; }}
  header.site h1 {{ font-size:2rem; color:var(--accent); }}
  header.site p {{ color:var(--muted); margin-top:.25rem; }}
  nav {{ margin-top:1rem; font-size:.875rem; color:var(--muted); line-height:2; }}
  nav a {{ color:var(--accent); text-decoration:none; }}
  nav a:hover {{ text-decoration:underline; }}
  main {{ max-width:900px; margin:0 auto; display:grid; gap:1.5rem; }}
  .card {{ background:var(--surface); border:1px solid var(--border); border-radius:.75rem; padding:1.5rem; }}
  .card header {{ margin-bottom:1rem; }}
  .card header h2 {{ font-size:1.25rem; display:flex; align-items:center; gap:.5rem; flex-wrap:wrap; }}
  .meta {{ font-size:.875rem; color:var(--muted); margin-top:.25rem; }}
  .meta a {{ color:var(--accent); text-decoration:none; }}
  .meta a:hover {{ text-decoration:underline; }}
  .badge {{ font-size:.7rem; font-weight:600; padding:.2em .5em; border-radius:.25rem; text-transform:uppercase; letter-spacing:.05em; }}
  .badge-framework {{ background:#312e81; color:#a5b4fc; }}
  .badge-orm {{ background:#064e3b; color:#6ee7b7; }}
  .badge-auth {{ background:#7f1d1d; color:#fca5a5; }}
  .badge-ui {{ background:#1e3a5f; color:#93c5fd; }}
  .badge-tool {{ background:#374151; color:#d1d5db; }}
  section {{ margin-top:1rem; }}
  section h3 {{ font-size:.875rem; font-weight:600; color:var(--muted); text-transform:uppercase; letter-spacing:.05em; margin-bottom:.5rem; }}
  ul {{ padding-left:1.25rem; }}
  li {{ margin:.25rem 0; font-size:.9rem; }}
  code {{ background:#1e2235; color:#93c5fd; padding:.1em .35em; border-radius:.25rem; font-size:.85em; }}
  .int-version {{ color:var(--muted); font-size:.8em; }}
  footer {{ max-width:900px; margin:3rem auto 0; text-align:center; font-size:.8rem; color:var(--muted); border-top:1px solid var(--border); padding-top:1.5rem; }}
</style>
</head>
<body>
<header class="site">
  <h1>TSX Framework Registry</h1>
  <p>All built-in and community framework registries for <code>tsx</code> — {count} registries</p>
  <nav>{nav_links}</nav>
</header>
<main>
{cards}
</main>
<footer>Generated by <code>tsx registry website</code> · <a href="https://github.com/Ateeg/tsx" style="color:var(--accent)">tsx CLI</a></footer>
</body>
</html>"#,
        count = registries.len(),
        nav_links = nav_links,
        cards = cards,
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Minimal percent-encoding for URL query parameters.
fn urlencoding(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                vec![c]
            }
            ' ' => vec!['+'],
            c => {
                let mut buf = [0u8; 4];
                let bytes = c.encode_utf8(&mut buf);
                bytes
                    .bytes()
                    .flat_map(|b| {
                        let hex: Vec<char> =
                            format!("%{:02X}", b).chars().collect();
                        hex
                    })
                    .collect()
            }
        })
        .collect()
}
