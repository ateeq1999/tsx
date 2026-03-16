use crate::output::CommandResult;
use crate::utils::paths::find_project_root;
use std::process::Command;

pub fn add_migration() -> CommandResult {
    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:migration", e.to_string()),
    };

    let mut warnings = Vec::new();

    let generate_output = Command::new("npx")
        .args(["drizzle-kit", "generate"])
        .current_dir(&root)
        .output();

    match generate_output {
        Ok(o) if !o.status.success() => {
            warnings.push(format!(
                "Generate failed: {}",
                String::from_utf8_lossy(&o.stderr)
            ));
        }
        Err(_) => warnings.push("Failed to run drizzle-kit generate".to_string()),
        _ => {}
    }

    let migrate_output = Command::new("npx")
        .args(["drizzle-kit", "migrate"])
        .current_dir(&root)
        .output();

    match migrate_output {
        Ok(o) if !o.status.success() => {
            warnings.push(format!(
                "Migrate failed: {}",
                String::from_utf8_lossy(&o.stderr)
            ));
        }
        Err(_) => warnings.push("Failed to run drizzle-kit migrate".to_string()),
        _ => {}
    }

    let mut result = CommandResult::ok("add:migration", vec![]);
    result.warnings = warnings;
    result
}
