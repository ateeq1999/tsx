use crate::output::CommandResult;
use crate::schemas::AddAuthGuardArgs;
use crate::utils::paths::find_project_root;
use std::fs;
use std::path::PathBuf;

pub fn add_auth_guard(args: AddAuthGuardArgs, overwrite: bool) -> CommandResult {
    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:auth-guard", e.to_string()),
    };

    let route_file = resolve_route_file(&root, &args.route_path);

    if !route_file.exists() {
        return CommandResult::err(
            "add:auth-guard",
            format!("Route file not found: {:?}", route_file),
        );
    }

    let content = match fs::read_to_string(&route_file) {
        Ok(c) => c,
        Err(e) => return CommandResult::err("add:auth-guard", format!("Read error: {}", e)),
    };

    if content.contains("beforeLoad") {
        return CommandResult::err("add:auth-guard", "Route already has a beforeLoad guard");
    }

    let guard_code = format!(
        r#"
  beforeLoad: async ({{ redirect }}) => {{
    const session = await getSession();
    if (!session && '{{}}' !== '/login') {{
      throw redirect({{ to: '{}' }});
    }}
  }}"#,
        args.redirect_to
    );

    let modified = content.replace(
        "export const Route = createFileRoute",
        &format!("export const Route = createFileRoute{}", guard_code),
    );

    match fs::write(&route_file, &modified) {
        Ok(_) => CommandResult::ok(
            "add:auth-guard",
            vec![route_file.to_string_lossy().to_string()],
        ),
        Err(e) => CommandResult::err("add:auth-guard", format!("Write error: {}", e)),
    }
}

fn resolve_route_file(root: &PathBuf, route_path: &str) -> PathBuf {
    let path = route_path.trim_start_matches('/').replace('-', "/");
    root.join("routes").join(format!("{}.tsx", path))
}
