use std::path::PathBuf;

use crate::json::response::ResponseEnvelope;
use crate::stack::StackProfile;

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
