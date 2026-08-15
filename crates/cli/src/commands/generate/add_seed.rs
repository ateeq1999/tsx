use crate::execution::ExecutionContext;
use crate::output::CommandResult;
use crate::render::render_and_write;
use crate::schemas::AddSeedArgs;
use crate::utils::format::format_typescript;
use crate::utils::paths::resolve_output_path;
use crate::utils::validate::validate_identifier;

pub fn add_seed(args: AddSeedArgs, ctx: &ExecutionContext) -> CommandResult {
    if let Err(e) = validate_identifier(&args.name) {
        return CommandResult::err("add:seed", format!("Invalid name: {}", e));
    }

    render_and_write(
        "add:seed",
        "features/seed.jinja",
        minijinja::context!(
            name => args.name,
            count => args.count,
            fields => Vec::<serde_json::Value>::new()
        ),
        |root| resolve_output_path(root, &format!("db/seeds/{}.ts", args.name)),
        format_typescript,
        ctx.overwrite,
        ctx.dry_run,
        ctx.diff,
    )
}
