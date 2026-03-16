use crate::output::CommandResult;
use crate::render::render_and_write;
use crate::schemas::AddAuthArgs;
use crate::utils::format::format_typescript;
use crate::utils::paths::resolve_output_path;
use std::path::Path;
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
        match install_better_auth() {
            Ok(msg) => result.files_created.push(msg),
            Err(e) => result.warnings.push(format!("better-auth install failed: {}", e)),
        }
    }

    result.next_steps = vec!["Configure your auth providers in .env".to_string()];
    result
}

fn install_better_auth() -> Result<String, String> {
    let root = crate::utils::paths::find_project_root()
        .map_err(|e| format!("Could not find project root: {}", e))?;
    install_better_auth_in(&root)
}

fn install_better_auth_in(root: &Path) -> Result<String, String> {
    let output = Command::new("npm")
        .args(["install", "better-auth", "@better-auth/react"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("npm could not run: {}", e))?;

    if output.status.success() {
        Ok("better-auth installed".to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
