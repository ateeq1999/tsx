use crate::output::CommandResult;
use crate::render::render_and_write;
use crate::schemas::AddSchemaArgs;
use crate::utils::format::format_typescript;
use crate::utils::paths::resolve_output_path;

pub fn add_schema(args: AddSchemaArgs, overwrite: bool, dry_run: bool) -> CommandResult {
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
