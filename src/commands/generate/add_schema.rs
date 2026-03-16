use crate::output::CommandResult;
use crate::render::render_and_write;
use crate::schemas::AddSchemaArgs;
use crate::utils::format::format_typescript;
use crate::utils::paths::resolve_output_path;
use crate::utils::validate::{validate_field_names, validate_fields_non_empty, validate_identifier};

pub fn add_schema(args: AddSchemaArgs, overwrite: bool, dry_run: bool) -> CommandResult {
    if let Err(e) = validate_identifier(&args.name) {
        return CommandResult::err("add:schema", format!("Invalid name: {}", e));
    }
    if let Err(e) = validate_fields_non_empty(&args.fields) {
        return CommandResult::err("add:schema", format!("Invalid fields: {}", e));
    }
    if let Err(e) = validate_field_names(&args.fields) {
        return CommandResult::err("add:schema", format!("Invalid fields: {}", e));
    }

    render_and_write(
        "add:schema",
        "features/schema.jinja",
        minijinja::context!(
            name => args.name,
            fields => args.fields,
            timestamps => args.timestamps,
            soft_delete => args.soft_delete
        ),
        |root| resolve_output_path(root, &format!("db/schema/{}.ts", args.name)),
        format_typescript,
        overwrite,
        dry_run,
    )
}
