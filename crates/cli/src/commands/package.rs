//! `tsx package` — author tools for creating and publishing registry packages.
//!
//! tsx package new <id>          scaffold a new package directory
//! tsx package validate          validate manifest.json and template refs
//! tsx package pack              create a .tgz locally
//! tsx package publish [--token] publish to the registry
//! tsx package install <id>      install a package from registry (alias for tsx registry install)

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;
use tsx_shared::PackageManifest;

// ---------------------------------------------------------------------------
// package new
// ---------------------------------------------------------------------------

pub fn package_new(id: String, out_dir: Option<String>) -> ResponseEnvelope {
    let dir = std::path::PathBuf::from(out_dir.unwrap_or_else(|| id.clone()));

    if dir.exists() {
        return ResponseEnvelope::error(
            "package new",
            ErrorResponse::new(ErrorCode::InternalError, format!("Directory already exists: {}", dir.display())),
            0,
        );
    }

    let templates_dir = dir.join("templates");
    let generators_dir = dir.join("generators");
    let knowledge_dir = dir.join("knowledge");

    for d in &[&templates_dir, &generators_dir, &knowledge_dir] {
        if let Err(e) = std::fs::create_dir_all(d) {
            return ResponseEnvelope::error(
                "package new",
                ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
                0,
            );
        }
    }

    let manifest = serde_json::json!({
        "id": id,
        "name": id,
        "version": "0.1.0",
        "description": "",
        "category": "framework",
        "author": "",
        "license": "MIT",
        "docs": "",
        "npm_packages": [],
        "commands": [],
        "stacks": {},
        "peer_packages": [],
        "tags": []
    });

    if let Err(e) = std::fs::write(
        dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    ) {
        return ResponseEnvelope::error(
            "package new",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        );
    }

    // Scaffold a sample generator spec
    let sample_gen = serde_json::json!({
        "id": "add:example",
        "command": "add:example",
        "description": "Example generator",
        "token_estimate": 100,
        "template": "example.jinja",
        "output_paths": ["{{name}}.ts"],
        "schema": {
            "type": "object",
            "required": ["name"],
            "properties": {
                "name": { "type": "string", "description": "Output file name" }
            }
        },
        "next_steps": []
    });
    let _ = std::fs::write(
        generators_dir.join("add-example.json"),
        serde_json::to_string_pretty(&sample_gen).unwrap(),
    );

    // Scaffold a sample template
    let _ = std::fs::write(
        templates_dir.join("example.jinja"),
        "// Generated: {{ name }}\n",
    );

    // Scaffold overview.md
    let _ = std::fs::write(
        knowledge_dir.join("overview.md"),
        format!("# {}\n\nDescribe your package here.\n", id),
    );

    ResponseEnvelope::success(
        "package new",
        serde_json::json!({
            "created": dir.to_string_lossy(),
            "files": [
                "manifest.json",
                "templates/example.jinja",
                "generators/add-example.json",
                "knowledge/overview.md"
            ]
        }),
        0,
    )
    .with_next_steps(vec![
        format!("Edit {}/manifest.json to fill in package details", dir.display()),
        format!("Run `tsx package validate` inside {} to check your package", dir.display()),
        "Run `tsx package publish` to publish to the registry".to_string(),
    ])
}

// ---------------------------------------------------------------------------
// package validate
// ---------------------------------------------------------------------------

pub fn package_validate(pkg_dir: Option<String>) -> ResponseEnvelope {
    let dir = std::path::PathBuf::from(pkg_dir.unwrap_or_else(|| ".".to_string()));
    let manifest_path = dir.join("manifest.json");

    let content = match std::fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(_) => {
            return ResponseEnvelope::error(
                "package validate",
                ErrorResponse::new(
                    ErrorCode::ProjectNotFound,
                    format!("No manifest.json found in {}", dir.display()),
                ),
                0,
            );
        }
    };

    let manifest: PackageManifest = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(e) => {
            return ResponseEnvelope::error(
                "package validate",
                ErrorResponse::new(ErrorCode::InternalError, format!("Invalid manifest.json: {}", e)),
                0,
            );
        }
    };

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Required fields
    if manifest.id.is_empty() { errors.push("manifest.id is required".to_string()); }
    if manifest.name.is_empty() { errors.push("manifest.name is required".to_string()); }
    if manifest.version.is_empty() { errors.push("manifest.version is required".to_string()); }
    if manifest.description.is_empty() { warnings.push("manifest.description is empty".to_string()); }
    if manifest.npm_packages.is_empty() { warnings.push("manifest.npm_packages is empty — package won't be auto-discovered".to_string()); }

    // Check template refs from commands
    for cmd in &manifest.commands {
        if !cmd.template.is_empty() {
            let tpl_path = dir.join("templates").join(&cmd.template);
            if !tpl_path.exists() {
                errors.push(format!("command '{}' references missing template: {}", cmd.id, cmd.template));
            }
        }
    }

    // Check generator specs
    let gen_dir = dir.join("generators");
    if gen_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&gen_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    match std::fs::read_to_string(&path)
                        .ok()
                        .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
                    {
                        Some(spec) => {
                            if let Some(tpl) = spec.get("template").and_then(|v| v.as_str()) {
                                if !tpl.is_empty() && !dir.join("templates").join(tpl).exists() {
                                    errors.push(format!(
                                        "generator '{}' references missing template: {}",
                                        path.file_name().unwrap_or_default().to_string_lossy(),
                                        tpl
                                    ));
                                }
                            }
                        }
                        None => errors.push(format!(
                            "invalid JSON in generator: {}",
                            path.display()
                        )),
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        let mut resp = ResponseEnvelope::success(
            "package validate",
            serde_json::json!({
                "valid": true,
                "package": manifest.id,
                "version": manifest.version,
                "warnings": warnings
            }),
            0,
        );
        if !warnings.is_empty() {
            resp = resp.with_next_steps(warnings.iter().map(|w| format!("Warning: {}", w)).collect());
        }
        resp
    } else {
        ResponseEnvelope::error(
            "package validate",
            ErrorResponse::new(ErrorCode::InternalError, errors.join("; ")),
            0,
        )
    }
}

// ---------------------------------------------------------------------------
// package pack
// ---------------------------------------------------------------------------

pub fn package_pack(pkg_dir: Option<String>, out: Option<String>) -> ResponseEnvelope {
    let dir = std::path::PathBuf::from(pkg_dir.unwrap_or_else(|| ".".to_string()));

    let manifest_path = dir.join("manifest.json");
    let content = match std::fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(_) => {
            return ResponseEnvelope::error(
                "package pack",
                ErrorResponse::new(ErrorCode::ProjectNotFound, "No manifest.json found"),
                0,
            );
        }
    };
    let manifest: PackageManifest = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(e) => {
            return ResponseEnvelope::error(
                "package pack",
                ErrorResponse::new(ErrorCode::InternalError, format!("Invalid manifest.json: {}", e)),
                0,
            );
        }
    };

    let tgz_name = out.unwrap_or_else(|| format!("{}-{}.tgz", manifest.id, manifest.version));
    let tgz_path = std::path::PathBuf::from(&tgz_name);

    match crate::packages::installer::pack(&dir, &tgz_path) {
        Ok(()) => ResponseEnvelope::success(
            "package pack",
            serde_json::json!({
                "package": manifest.id,
                "version": manifest.version,
                "tarball": tgz_name
            }),
            0,
        ),
        Err(e) => ResponseEnvelope::error(
            "package pack",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}

// ---------------------------------------------------------------------------
// package publish
// ---------------------------------------------------------------------------

pub fn package_publish(
    pkg_dir: Option<String>,
    registry_url: Option<String>,
    token: Option<String>,
) -> ResponseEnvelope {
    let dir = std::path::PathBuf::from(pkg_dir.unwrap_or_else(|| ".".to_string()));
    let registry = registry_url.unwrap_or_else(|| {
        std::env::var("TSX_REGISTRY_URL").unwrap_or_else(|_| "https://registry.tsx.dev".to_string())
    });

    // Validate first
    let validate_resp = package_validate(Some(dir.to_string_lossy().to_string()));
    if !validate_resp.success {
        return ResponseEnvelope::error(
            "package publish",
            ErrorResponse::new(ErrorCode::InternalError, "Package validation failed — run `tsx package validate` for details"),
            0,
        );
    }

    // Pack to a temp tarball
    let manifest_content = match std::fs::read_to_string(dir.join("manifest.json")) {
        Ok(c) => c,
        Err(e) => return ResponseEnvelope::error("package publish", ErrorResponse::new(ErrorCode::InternalError, e.to_string()), 0),
    };
    let manifest: PackageManifest = match serde_json::from_str(&manifest_content) {
        Ok(m) => m,
        Err(e) => return ResponseEnvelope::error("package publish", ErrorResponse::new(ErrorCode::InternalError, e.to_string()), 0),
    };

    let tgz_path = std::env::temp_dir().join(format!("{}-{}.tgz", manifest.id, manifest.version));
    if let Err(e) = crate::packages::installer::pack(&dir, &tgz_path) {
        return ResponseEnvelope::error("package publish", ErrorResponse::new(ErrorCode::InternalError, e.to_string()), 0);
    }

    // Upload
    let url = format!("{}/v1/packages/publish", registry);
    let tgz_bytes = match std::fs::read(&tgz_path) {
        Ok(b) => b,
        Err(e) => return ResponseEnvelope::error("package publish", ErrorResponse::new(ErrorCode::InternalError, e.to_string()), 0),
    };
    let _ = std::fs::remove_file(&tgz_path);

    let mut req = ureq::post(&url);
    if let Some(tok) = &token {
        req = req.set("Authorization", &format!("Bearer {}", tok));
    } else if let Ok(tok) = std::env::var("TSX_TOKEN") {
        req = req.set("Authorization", &format!("Bearer {}", tok));
    }

    match req
        .set("Content-Type", "application/octet-stream")
        .set("X-Package-Id", &manifest.id)
        .set("X-Package-Version", &manifest.version)
        .send_bytes(&tgz_bytes)
    {
        Ok(resp) => {
            let body: serde_json::Value = serde_json::from_reader(resp.into_reader())
                .unwrap_or_default();
            ResponseEnvelope::success("package publish", body, 0)
                .with_next_steps(vec![
                    format!("Package {} v{} published to {}", manifest.id, manifest.version, registry),
                    "Run `tsx registry install <id>` to install it in a project".to_string(),
                ])
        }
        Err(e) => ResponseEnvelope::error(
            "package publish",
            ErrorResponse::new(ErrorCode::InternalError, format!("Upload failed: {}", e)),
            0,
        ),
    }
}

// ---------------------------------------------------------------------------
// package install (wrapper around registry install)
// ---------------------------------------------------------------------------

pub fn package_install(id: String, registry_url: Option<String>) -> ResponseEnvelope {
    let registry = registry_url.unwrap_or_else(|| {
        std::env::var("TSX_REGISTRY_URL").unwrap_or_else(|_| "https://registry.tsx.dev".to_string())
    });

    match crate::packages::installer::install(&registry, &id, None) {
        Ok(summary) => ResponseEnvelope::success(
            "package install",
            serde_json::json!({
                "installed": summary.id,
                "version": summary.version,
                "path": summary.install_path,
                "commands": summary.commands
            }),
            0,
        )
        .with_next_steps(vec![
            format!("Run `tsx run --list` to see commands from {}", id),
        ]),
        Err(e) => ResponseEnvelope::error(
            "package install",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}
