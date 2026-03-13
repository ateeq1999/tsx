use crate::output::CommandResult;
use crate::render::engine::{build_engine, reset_import_collector};
use crate::schemas::AddFormArgs;
use crate::utils::format::format_tsx;
use crate::utils::paths::{find_project_root, resolve_output_path};
use crate::utils::write::{write_file, WriteOutcome};
use std::path::PathBuf;

pub fn add_form(args: AddFormArgs, overwrite: bool, dry_run: bool) -> CommandResult {
    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:form", e.to_string()),
    };

    let templates_dir = get_templates_dir(&root);
    let engine = build_engine(&templates_dir);

    reset_import_collector();

    let template = match engine.get_template("features/form.jinja") {
        Ok(t) => t,
        Err(e) => return CommandResult::err("add:form", format!("Template error: {}", e)),
    };

    let rendered = match template.render(minijinja::context!(
        name => args.name,
        fields => args.fields
    )) {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:form", format!("Render error: {}", e)),
    };

    let formatted = match format_tsx(&rendered) {
        Ok(f) => f,
        Err(_) => rendered,
    };

    let output_path = resolve_output_path(
        &root,
        &format!("components/{}/{}-form.tsx", args.name, args.name),
    );

    let files_created = if dry_run {
        vec![output_path.to_string_lossy().to_string()]
    } else {
        let outcome = match write_file(&output_path, &formatted, overwrite) {
            Ok(o) => o,
            Err(e) => return CommandResult::err("add:form", format!("Write error: {}", e)),
        };

        match outcome {
            WriteOutcome::Created | WriteOutcome::Overwritten => {
                vec![output_path.to_string_lossy().to_string()]
            }
            WriteOutcome::Skipped => vec![],
        }
    };

    CommandResult::ok("add:form", files_created)
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
