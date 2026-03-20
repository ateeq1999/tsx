use crate::output::CommandResult;
use crate::render::render_and_write;
use crate::schemas::AddServerFnArgs;
use crate::utils::format::format_typescript;
use crate::utils::paths::resolve_output_path;
use crate::utils::validate::validate_identifier;

pub fn add_server_fn(args: AddServerFnArgs, overwrite: bool, dry_run: bool, diff_only: bool) -> CommandResult {
    if let Err(e) = validate_identifier(&args.name) {
        return CommandResult::err("add:server-fn", format!("Invalid name: {}", e));
    }
    if let Err(e) = validate_identifier(&args.table) {
        return CommandResult::err("add:server-fn", format!("Invalid table: {}", e));
    }
    let operations = vec![args.operation.clone()];
    let input = args.input.clone();

    render_and_write(
        "add:server-fn",
        "features/server_fn.jinja",
        minijinja::context!(
            name => args.name,
            table => args.table,
            operations => operations,
            auth => args.auth,
            input => input
        ),
        |root| resolve_output_path(root, &format!("server-functions/{}.ts", args.name)),
        format_typescript,
        overwrite,
        dry_run,
        diff_only,
    )
}
