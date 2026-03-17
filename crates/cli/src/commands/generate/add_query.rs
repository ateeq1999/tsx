use crate::output::CommandResult;
use crate::render::render_and_write;
use crate::schemas::AddQueryArgs;
use crate::utils::format::format_typescript;
use crate::utils::paths::resolve_output_path;
use crate::utils::validate::validate_identifier;

pub fn add_query(args: AddQueryArgs, overwrite: bool, dry_run: bool) -> CommandResult {
    if let Err(e) = validate_identifier(&args.name) {
        return CommandResult::err("add:query", format!("Invalid name: {}", e));
    }
    let operations: Vec<String> = if args.mutation {
        vec!["create".to_string()]
    } else {
        vec!["list".to_string()]
    };

    render_and_write(
        "add:query",
        "features/query.jinja",
        minijinja::context!(
            name => args.name,
            operations => operations
        ),
        |root| resolve_output_path(root, &format!("queries/{}.ts", args.name)),
        format_typescript,
        overwrite,
        dry_run,
    )
}
