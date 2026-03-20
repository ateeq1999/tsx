use crate::output::CommandResult;
use crate::render::render_and_write;
use crate::schemas::AddTableArgs;
use crate::utils::format::format_tsx;
use crate::utils::paths::resolve_output_path;
use crate::utils::validate::{validate_field_names, validate_fields_non_empty, validate_identifier};

pub fn add_table(args: AddTableArgs, overwrite: bool, dry_run: bool, diff_only: bool) -> CommandResult {
    if let Err(e) = validate_identifier(&args.name) {
        return CommandResult::err("add:table", format!("Invalid name: {}", e));
    }
    if let Err(e) = validate_fields_non_empty(&args.fields) {
        return CommandResult::err("add:table", format!("Invalid fields: {}", e));
    }
    if let Err(e) = validate_field_names(&args.fields) {
        return CommandResult::err("add:table", format!("Invalid fields: {}", e));
    }
    let name = args.name.clone();
    render_and_write(
        "add:table",
        "features/table.jinja",
        minijinja::context!(
            name => args.name,
            fields => args.fields,
            query_fn => args.query_fn,
            paginated => args.paginated,
            sortable => args.sortable,
        ),
        move |root| resolve_output_path(root, &format!("components/{name}/{name}-table.tsx")),
        format_tsx,
        overwrite,
        dry_run,
        diff_only,
    )
}
