use crate::output::CommandResult;
use crate::render::engine::{build_engine, reset_import_collector};
use crate::schemas::AddPageArgs;
use crate::utils::paths::{find_project_root, resolve_output_path};
use crate::utils::write::{write_file, WriteOutcome};
use std::path::PathBuf;

pub fn add_page(args: AddPageArgs, overwrite: bool) -> CommandResult {
    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:page", e.to_string()),
    };

    let templates_dir = get_templates_dir(&root);
    let mut engine = build_engine(&templates_dir);

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

    let route_path = args.path.trim_start_matches('/').replace('/', "-");
    let output_path = resolve_output_path(&root, &format!("routes/{}.tsx", route_path));

    let outcome = match write_file(&output_path, &rendered, overwrite) {
        Ok(o) => o,
        Err(e) => return CommandResult::err("add:page", format!("Write error: {}", e)),
    };

    let files_created = match outcome {
        WriteOutcome::Created | WriteOutcome::Overwritten => {
            vec![output_path.to_string_lossy().to_string()]
        }
        WriteOutcome::Skipped => vec![],
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
