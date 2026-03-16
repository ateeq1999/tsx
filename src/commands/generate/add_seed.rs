use crate::output::CommandResult;
use crate::render::render_and_write;
use crate::schemas::AddSeedArgs;
use crate::utils::format::format_typescript;
use crate::utils::paths::resolve_output_path;

pub fn add_seed(args: AddSeedArgs, overwrite: bool, dry_run: bool) -> CommandResult {
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
        overwrite,
        dry_run,
    )
}
