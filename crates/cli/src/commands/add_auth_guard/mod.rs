use crate::output::CommandResult;
use crate::schemas::AddAuthGuardArgs;
use crate::utils::paths::find_project_root;
use std::fs;
use std::path::PathBuf;

pub fn add_auth_guard(args: AddAuthGuardArgs, _overwrite: bool, dry_run: bool, _diff_only: bool) -> CommandResult {
    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:auth-guard", e.to_string()),
    };

    let route_file = resolve_route_file(&root, &args.route_path);

    if !route_file.exists() {
        return CommandResult::err(
            "add:auth-guard",
            format!("Route file not found: {}", route_file.display()),
        );
    }

    let content = match fs::read_to_string(&route_file) {
        Ok(c) => c,
        Err(e) => return CommandResult::err("add:auth-guard", format!("Read error: {}", e)),
    };

    // Guard against double-injection
    if content.contains("beforeLoad") {
        return CommandResult::err(
            "add:auth-guard",
            format!(
                "Route '{}' already has a beforeLoad guard",
                route_file.display()
            ),
        );
    }

    // The guard block to inject
    let guard_code = format!(
        concat!(
            "  beforeLoad: async ({{ context, redirect }}) => {{\n",
            "    const session = context.session ?? await getSession();\n",
            "    if (!session) {{\n",
            "      throw redirect({{ to: '{}' }});\n",
            "    }}\n",
            "  }},\n",
        ),
        args.redirect_to
    );

    // Inject into the createFileRoute('...')({{ ... }}) options object.
    //
    // Patterns handled:
    //   createFileRoute('/path')({ component: Foo })
    //   createFileRoute('/path')({
    //     component: Foo,
    //   })
    //
    // Strategy: find the opening brace of the options object (the `({` after
    // the route path argument) and insert the beforeLoad entry right after it.
    let modified = inject_before_load(&content, &guard_code);

    if modified.is_none() {
        return CommandResult::err(
            "add:auth-guard",
            format!(
                "Could not locate route options object in '{}'. \
                 Expected pattern: createFileRoute('...')(<options>)",
                route_file.display()
            ),
        );
    }

    let modified = modified.unwrap();

    // Verify injection is syntactically sound — beforeLoad must now be present
    if !modified.contains("beforeLoad") {
        return CommandResult::err(
            "add:auth-guard",
            "Guard injection produced invalid output — aborting write".to_string(),
        );
    }

    if dry_run {
        return CommandResult::ok(
            "add:auth-guard",
            vec![route_file.to_string_lossy().to_string()],
        );
    }

    match fs::write(&route_file, &modified) {
        Ok(_) => {
            let mut result = CommandResult::ok(
                "add:auth-guard",
                vec![route_file.to_string_lossy().to_string()],
            );
            result.next_steps = vec![
                "Import getSession from your auth client at the top of the route file.".to_string(),
            ];
            result
        }
        Err(e) => CommandResult::err("add:auth-guard", format!("Write error: {}", e)),
    }
}

/// Find the options object passed to `createFileRoute('...')({...})` and
/// insert `beforeLoad` as the first entry.
///
/// Handles both single-line and multi-line options objects.
fn inject_before_load(source: &str, guard_code: &str) -> Option<String> {
    // Find `createFileRoute(` — locate the options object that follows
    let create_pos = source.find("createFileRoute(")?;

    // Skip past the route path argument: find the closing `)` after the path string
    let after_create = &source[create_pos..];

    // Find the second `(` which opens the options object — pattern: `createFileRoute('...')(  {`
    let first_close = after_create.find(')')?;
    let after_path = &after_create[first_close + 1..];

    // Find the `{` that opens the options object (may have whitespace before it)
    let brace_offset = after_path.find('{')?;

    // Absolute position of the `{` in the original source
    let insert_at = create_pos + first_close + 1 + brace_offset + 1; // +1 to insert AFTER `{`

    if insert_at > source.len() {
        return None;
    }

    // Build the modified source: everything before `{`, then `{\n<guard>`, then rest
    let (before, after) = source.split_at(insert_at);
    Some(format!("{}\n{}{}", before, guard_code, after))
}

fn resolve_route_file(root: &PathBuf, route_path: &str) -> PathBuf {
    // Normalise: strip leading slash, preserve the path as-is (don't replace `-`)
    let path = route_path.trim_start_matches('/');
    root.join("routes").join(format!("{}.tsx", path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_single_line() {
        let src = "export const Route = createFileRoute('/dashboard')({ component: Dashboard });";
        let result = inject_before_load(src, "  beforeLoad: async () => {},\n").unwrap();
        assert!(result.contains("beforeLoad"));
        assert!(result.contains("component: Dashboard"));
    }

    #[test]
    fn inject_multiline() {
        let src = "export const Route = createFileRoute('/dashboard')({\n  component: Dashboard,\n});";
        let result = inject_before_load(src, "  beforeLoad: async () => {},\n").unwrap();
        assert!(result.contains("beforeLoad"));
        assert!(result.contains("component: Dashboard"));
    }

    #[test]
    fn no_create_file_route_returns_none() {
        let src = "export const x = 1;";
        assert!(inject_before_load(src, "  beforeLoad: async () => {},\n").is_none());
    }

    #[test]
    fn resolve_route_strips_leading_slash() {
        let root = PathBuf::from("/project");
        let path = resolve_route_file(&root, "/dashboard/index");
        assert_eq!(path, PathBuf::from("/project/routes/dashboard/index.tsx"));
    }
}
