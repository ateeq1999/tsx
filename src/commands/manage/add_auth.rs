use crate::output::CommandResult;
use crate::render::render_and_write;
use crate::schemas::AddAuthArgs;
use crate::utils::format::format_typescript;
use crate::utils::paths::resolve_output_path;
use std::path::PathBuf;
use std::process::Command;

pub fn add_auth(args: AddAuthArgs, overwrite: bool, dry_run: bool) -> CommandResult {
    let mut result = render_and_write(
        "add:auth",
        "features/auth_config.jinja",
        minijinja::context!(
            providers => args.providers,
            session_fields => args.session_fields,
            email_verification => args.email_verification
        ),
        |root| resolve_output_path(root, "lib/auth.ts"),
        format_typescript,
        overwrite,
        dry_run,
    );

    if result.success && !dry_run {
        result.files_created.extend(install_better_auth());
    }

    result.next_steps = vec!["Configure your auth providers in .env".to_string()];
    result
}

fn install_better_auth() -> Vec<String> {
    let root = match crate::utils::paths::find_project_root() {
        Ok(r) => r,
        Err(_) => return vec![],
    };
    install_better_auth_in(&root)
}

fn install_better_auth_in(root: &PathBuf) -> Vec<String> {
    let output = Command::new("npm")
        .args(["install", "better-auth", "@better-auth/react"])
        .current_dir(root)
        .output();

    match output {
        Ok(o) if o.status.success() => vec!["better-auth installed".to_string()],
        _ => vec![],
    }
}
