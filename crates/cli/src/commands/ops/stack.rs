use std::path::PathBuf;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;
use crate::stack::StackProfile;

pub fn stack_init(
    lang: Option<String>,
    packages: Option<String>,
    dry_run: bool,
    _verbose: bool,
) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let detected = StackProfile::detect(&cwd);

    let mut profile = StackProfile::default();

    // Prefer explicit lang arg, fall back to detection
    profile.lang = lang.unwrap_or_else(|| {
        if detected.lang.is_empty() {
            "typescript".to_string()
        } else {
            detected.lang.clone()
        }
    });
    profile.runtime = detected.runtime;

    // Prefer explicit package list, fall back to detection
    if let Some(pkgs) = packages {
        for p in pkgs.split(',').map(|s| s.trim()) {
            if !p.is_empty() {
                profile.add_package(p);
            }
        }
    } else {
        profile.packages = detected.packages;
    }

    let file_path = StackProfile::stack_file(&cwd);
    let profile_json = serde_json::to_value(&profile).unwrap_or_default();

    if dry_run {
        return ResponseEnvelope::success(
            "stack init",
            serde_json::json!({
                "dry_run": true,
                "would_write": file_path.to_string_lossy(),
                "profile": profile_json
            }),
            0,
        );
    }

    match profile.save(&cwd) {
        Ok(_) => ResponseEnvelope::success(
            "stack init",
            serde_json::json!({
                "written": file_path.to_string_lossy(),
                "profile": profile_json
            }),
            0,
        )
        .with_next_steps(vec![
            "Run `tsx stack show` to verify the profile".to_string(),
            "Run `tsx list` to see all available commands".to_string(),
            "Run `tsx context` to get your agent onboarding prompt".to_string(),
        ]),
        Err(e) => ResponseEnvelope::error(
            "stack init",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}

pub fn stack_show(_verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    match StackProfile::load(&cwd) {
        Some(profile) => ResponseEnvelope::success(
            "stack show",
            serde_json::to_value(&profile).unwrap_or_default(),
            0,
        ),
        None => ResponseEnvelope::error(
            "stack show",
            ErrorResponse::new(
                ErrorCode::ProjectNotFound,
                "No .tsx/stack.json found — run `tsx stack init` first",
            ),
            0,
        ),
    }
}

pub fn stack_add(package: String, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut profile = StackProfile::load(&cwd).unwrap_or_default();
    profile.add_package(&package);

    match profile.save(&cwd) {
        Ok(_) => ResponseEnvelope::success(
            "stack add",
            serde_json::json!({
                "added": package,
                "packages": profile.packages
            }),
            0,
        ),
        Err(e) => ResponseEnvelope::error(
            "stack add",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}

pub fn stack_remove(package: String, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut profile = StackProfile::load(&cwd).unwrap_or_else(|| {
        StackProfile::default()
    });
    let before = profile.packages.len();
    profile
        .packages
        .retain(|p| p.split('@').next().unwrap_or(p) != package.as_str());
    let removed = before - profile.packages.len();

    if removed == 0 {
        return ResponseEnvelope::error(
            "stack remove",
            ErrorResponse::new(
                ErrorCode::UnknownCommand,
                format!("Package '{}' not found in stack", package),
            ),
            0,
        );
    }

    match profile.save(&cwd) {
        Ok(_) => ResponseEnvelope::success(
            "stack remove",
            serde_json::json!({
                "removed": package,
                "packages": profile.packages
            }),
            0,
        ),
        Err(e) => ResponseEnvelope::error(
            "stack remove",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}

pub fn stack_detect(install: bool, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let detected = StackProfile::detect(&cwd);

    let existing = StackProfile::load(&cwd);
    let has_stack = existing.is_some();

    if !install {
        return ResponseEnvelope::success(
            "stack detect",
            serde_json::json!({
                "lang": detected.lang,
                "runtime": detected.runtime,
                "suggested_packages": detected.packages,
                "has_stack_json": has_stack,
            }),
            0,
        )
        .with_next_steps(vec![
            if has_stack {
                "Stack already configured. Run `tsx stack show` to see it.".to_string()
            } else {
                "Run `tsx stack init` to create .tsx/stack.json with the detected packages".to_string()
            },
        ]);
    }

    // --install: auto-install each detected package via registry_install
    let mut installed = vec![];
    let mut failed = vec![];
    for pkg in &detected.packages {
        // Strip version suffix if any (e.g. "drizzle-pg@1.0.0" → "drizzle-pg")
        let slug = pkg.split('@').next().unwrap_or(pkg.as_str());
        let npm_pkg = format!("@tsx-pkg/{}", slug);
        if crate::commands::registry::registry_install(npm_pkg.clone(), false).success {
            installed.push(npm_pkg);
        } else {
            failed.push(npm_pkg);
        }
    }

    ResponseEnvelope::success(
        "stack detect",
        serde_json::json!({
            "lang": detected.lang,
            "runtime": detected.runtime,
            "suggested_packages": detected.packages,
            "has_stack_json": has_stack,
            "installed": installed,
            "failed": failed,
        }),
        0,
    )
    .with_next_steps(vec![
        "Run `tsx stack init` to activate the installed packages".to_string(),
        "Run `tsx context` to get your agent onboarding prompt".to_string(),
    ])
}
