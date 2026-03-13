use crate::output::CommandResult;
use crate::render::engine::{build_engine, reset_import_collector};
use crate::schemas::AddFormArgs;
use crate::utils::paths::{find_project_root, resolve_output_path};
use crate::utils::write::{write_file, WriteOutcome};
use std::path::PathBuf;

pub fn add_table(args: AddFormArgs, overwrite: bool) -> CommandResult {
    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:table", e.to_string()),
    };

    let templates_dir = get_templates_dir(&root);
    let engine = build_engine(&templates_dir);

    reset_import_collector();

    let template = match engine.get_template("features/table.jinja") {
        Ok(t) => t,
        Err(e) => return CommandResult::err("add:table", format!("Template error: {}", e)),
    };

    let rendered = match template.render(minijinja::context!(
        name => args.name,
        fields => args.fields
    )) {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:table", format!("Render error: {}", e)),
    };

    let output_path = resolve_output_path(
        &root,
        &format!("components/{}/{}-table.tsx", args.name, args.name),
    );

    let outcome = match write_file(&output_path, &rendered, overwrite) {
        Ok(o) => o,
        Err(e) => return CommandResult::err("add:table", format!("Write error: {}", e)),
    };

    let files_created = match outcome {
        WriteOutcome::Created | WriteOutcome::Overwritten => {
            vec![output_path.to_string_lossy().to_string()]
        }
        WriteOutcome::Skipped => vec![],
    };

    CommandResult::ok("add:table", files_created)
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
