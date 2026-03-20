use crate::output::CommandResult;
use crate::render::render_and_write;
use crate::schemas::AddPageArgs;
use crate::utils::format::format_tsx;
use crate::utils::paths::resolve_output_path;
use crate::utils::validate::validate_route_path;

pub fn add_page(args: AddPageArgs, overwrite: bool, dry_run: bool, diff_only: bool) -> CommandResult {
    if let Err(e) = validate_route_path(&args.path) {
        return CommandResult::err("add:page", format!("Invalid path: {}", e));
    }
    let path_parts: Vec<&str> = args.path.trim_start_matches('/').split('/').collect();
    let name = path_parts.last().unwrap_or(&"page").to_string();
    let route_path = args.path.trim_start_matches('/').replace('/', "-");
    let route_display = args.path.trim_start_matches('/').to_string();

    render_and_write(
        "add:page",
        "features/page.jinja",
        minijinja::context!(
            name => name,
            route_path => route_display
        ),
        move |root| resolve_output_path(root, &format!("routes/{route_path}.tsx")),
        format_tsx,
        overwrite,
        dry_run,
        diff_only,
    )
}
