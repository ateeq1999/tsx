use crate::output::CommandResult;
use crate::render::engine::{build_engine, reset_import_collector};
use crate::schemas::AddPageArgs;
use crate::utils::format::format_tsx;
use crate::utils::paths::{find_project_root, resolve_output_path};
use crate::utils::write::{write_file, WriteOutcome};
use std::path::PathBuf;

pub fn add_page(args: AddPageArgs, overwrite: bool, dry_run: bool) -> CommandResult {
    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:page", e.to_string()),
    };

    let templates_dir = get_templates_dir(&root);
    let engine = build_engine(&templates_dir);

    reset_import_collector();

    let template = match engine.get_template("features/page.jinja") {
        Ok(t) => t,
        Err(e) => return CommandResult::err("add:page", format!("Template error: {}", e)),
    };

    let path_parts: Vec<&str> = args.path.trim_start_matches('/').split('/').collect();
    let name = path_parts.last().unwrap_or(&"page");

    let rendered = match template.render(minijinja::context!(
        name => name,
        route_path => args.path.trim_start_matches('/')
    )) {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:page", format!("Render error: {}", e)),
    };

    let formatted = match format_tsx(&rendered) {
        Ok(f) => f,
        Err(_) => rendered,
    };

    let route_path = args.path.trim_start_matches('/').replace('/', "-");
    let output_path = resolve_output_path(&root, &format!("routes/{}.tsx", route_path));

    let files_created = if dry_run {
        vec![output_path.to_string_lossy().to_string()]
    } else {
        let outcome = match write_file(&output_path, &formatted, overwrite) {
            Ok(o) => o,
            Err(e) => return CommandResult::err("add:page", format!("Write error: {}", e)),
        };

        match outcome {
            WriteOutcome::Created | WriteOutcome::Overwritten => {
                vec![output_path.to_string_lossy().to_string()]
            }
            WriteOutcome::Skipped => vec![],
        }
    };

    CommandResult::ok("add:page", files_created)
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
