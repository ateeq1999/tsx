use crate::output::CommandResult;
use crate::render::engine::{build_engine, reset_import_collector};
use crate::schemas::AddSchemaArgs;
use crate::utils::format::format_typescript;
use crate::utils::paths::{find_project_root, resolve_output_path};
use crate::utils::write::{write_file, WriteOutcome};
use std::path::PathBuf;

pub fn add_schema(args: AddSchemaArgs, overwrite: bool, dry_run: bool) -> CommandResult {
    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:schema", e.to_string()),
    };

    let templates_dir = get_templates_dir(&root);
    let engine = build_engine(&templates_dir);

    reset_import_collector();

    let template = match engine.get_template("features/schema.jinja") {
        Ok(t) => t,
        Err(e) => return CommandResult::err("add:schema", format!("Template error: {}", e)),
    };

    let rendered = match template.render(minijinja::context!(
        name => args.name,
        fields => args.fields,
        timestamps => args.timestamps,
        soft_delete => args.soft_delete
    )) {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:schema", format!("Render error: {}", e)),
    };

    let formatted = match format_typescript(&rendered) {
        Ok(f) => f,
        Err(_) => rendered,
    };

    let output_path = resolve_output_path(&root, &format!("db/schema/{}.ts", args.name));

    let files_created = if dry_run {
        vec![output_path.to_string_lossy().to_string()]
    } else {
        let outcome = match write_file(&output_path, &formatted, overwrite) {
            Ok(o) => o,
            Err(e) => return CommandResult::err("add:schema", format!("Write error: {}", e)),
        };

        match outcome {
            WriteOutcome::Created | WriteOutcome::Overwritten => {
                vec![output_path.to_string_lossy().to_string()]
            }
            WriteOutcome::Skipped => vec![],
        }
    };

    CommandResult::ok("add:schema", files_created)
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
