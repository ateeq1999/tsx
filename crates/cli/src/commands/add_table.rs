use crate::output::CommandResult;
use crate::render::render_and_write;
use crate::schemas::AddFormArgs;
use crate::utils::format::format_tsx;
use crate::utils::paths::resolve_output_path;

pub fn add_table(args: AddFormArgs, overwrite: bool, dry_run: bool) -> CommandResult {
    let name = args.name.clone();
    render_and_write(
        "add:table",
        "features/table.jinja",
        minijinja::context!(
            name => args.name,
            fields => args.fields
        ),
        move |root| resolve_output_path(root, &format!("components/{name}/{name}-table.tsx")),
        format_tsx,
        overwrite,
        dry_run,
    )
}
