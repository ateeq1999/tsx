use crate::output::CommandResult;
use crate::render::engine::{build_engine, reset_import_collector};
use crate::schemas::AddAuthArgs;
use crate::utils::paths::{find_project_root, resolve_output_path};
use crate::utils::write::{write_file, WriteOutcome};
use std::path::PathBuf;
use std::process::Command;

pub fn add_auth(args: AddAuthArgs, overwrite: bool) -> CommandResult {
    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:auth", e.to_string()),
    };

    let templates_dir = get_templates_dir(&root);
    let engine = build_engine(&templates_dir);

    reset_import_collector();

    let template = match engine.get_template("features/auth_config.jinja") {
        Ok(t) => t,
        Err(e) => return CommandResult::err("add:auth", format!("Template error: {}", e)),
    };

    let rendered = match template.render(minijinja::context!(
        providers => args.providers,
        session_fields => args.session_fields,
        email_verification => args.email_verification
    )) {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:auth", format!("Render error: {}", e)),
    };

    let output_path = resolve_output_path(&root, "lib/auth.ts");

    let outcome = match write_file(&output_path, &rendered, overwrite) {
        Ok(o) => o,
        Err(e) => return CommandResult::err("add:auth", format!("Write error: {}", e)),
    };

    let mut files_created = match outcome {
        WriteOutcome::Created | WriteOutcome::Overwritten => {
            vec![output_path.to_string_lossy().to_string()]
        }
        WriteOutcome::Skipped => vec![],
    };

    let install_result = install_better_auth(&root);
    if !install_result.is_empty() {
        files_created.extend(install_result);
    }

    let mut result = CommandResult::ok("add:auth", files_created);
    result.next_steps = vec!["Configure your auth providers in .env".to_string()];
    result
}

fn install_better_auth(root: &PathBuf) -> Vec<String> {
    let mut files = Vec::new();

    let output = Command::new("npm")
        .args(["install", "better-auth", "@better-auth/react"])
        .current_dir(root)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            files.push("better-auth installed".to_string());
        }
    }

    files
}

fn get_templates_dir(root: &PathBuf) -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    if let Some(dir) = exe_dir {
        let templates = dir.join("templates");
        if templates.exists() {
            return templates;
        }
    }

    root.join("templates")
}
