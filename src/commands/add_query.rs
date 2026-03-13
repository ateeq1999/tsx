use crate::output::CommandResult;
use crate::render::engine::{build_engine, reset_import_collector};
use crate::schemas::AddQueryArgs;
use crate::utils::paths::{find_project_root, resolve_output_path};
use crate::utils::write::{write_file, WriteOutcome};
use std::path::PathBuf;

pub fn add_query(args: AddQueryArgs, overwrite: bool) -> CommandResult {
    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:query", e.to_string()),
    };

    let templates_dir = get_templates_dir(&root);
    let mut engine = build_engine(&templates_dir);

    reset_import_collector();

    let template = match engine.get_template("features/query.jinja") {
        Ok(t) => t,
        Err(e) => return CommandResult::err("add:query", format!("Template error: {}", e)),
    };

    let operations: Vec<String> = if args.mutation {
        vec!["create".to_string()]
    } else {
        vec!["list".to_string()]
    };

    let rendered = match template.render(minijinja::context!(
        name => args.name,
        operations => operations
    )) {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:query", format!("Render error: {}", e)),
    };

    let output_path = resolve_output_path(&root, &format!("queries/{}.ts", args.name));

    let outcome = match write_file(&output_path, &rendered, overwrite) {
        Ok(o) => o,
        Err(e) => return CommandResult::err("add:query", format!("Write error: {}", e)),
    };

    let files_created = match outcome {
        WriteOutcome::Created | WriteOutcome::Overwritten => {
            vec![output_path.to_string_lossy().to_string()]
        }
        WriteOutcome::Skipped => vec![],
    };

    CommandResult::ok("add:query", files_created)
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
