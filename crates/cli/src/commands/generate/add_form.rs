use crate::execution::ExecutionContext;
use crate::output::CommandResult;
use crate::render::render_and_write;
use crate::schemas::AddFormArgs;
use crate::utils::format::format_tsx;
use crate::utils::paths::resolve_output_path;
use crate::utils::validate::{validate_field_names, validate_fields_non_empty, validate_identifier};

pub fn add_form(args: AddFormArgs, ctx: &ExecutionContext) -> CommandResult {
    if let Err(e) = validate_identifier(&args.name) {
        return CommandResult::err("add:form", format!("Invalid name: {}", e));
    }
    if let Err(e) = validate_fields_non_empty(&args.fields) {
        return CommandResult::err("add:form", format!("Invalid fields: {}", e));
    }
    if let Err(e) = validate_field_names(&args.fields) {
        return CommandResult::err("add:form", format!("Invalid fields: {}", e));
    }

    let name = args.name.clone();
    render_and_write(
        "add:form",
        "features/form.jinja",
        minijinja::context!(
            name => args.name,
            fields => args.fields
        ),
        move |root| resolve_output_path(root, &format!("components/{name}/{name}-form.tsx")),
        format_tsx,
        ctx.overwrite,
        ctx.dry_run,
        ctx.diff,
    )
}
