use crate::output::CommandResult;
use crate::utils::paths::find_project_root;
use std::process::Command;

pub fn add_migration() -> CommandResult {
    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:migration", e.to_string()),
    };

    // Step 1: drizzle-kit generate — hard failure
    let generate_output = Command::new("npx")
        .args(["drizzle-kit", "generate"])
        .current_dir(&root)
        .output();

    match generate_output {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            return CommandResult::err(
                "add:migration",
                format!(
                    "drizzle-kit generate failed:\n{}",
                    String::from_utf8_lossy(&o.stderr).trim()
                ),
            );
        }
        Err(e) => {
            return CommandResult::err(
                "add:migration",
                format!("Failed to run drizzle-kit generate: {}", e),
            );
        }
    }

    // Step 2: drizzle-kit migrate — hard failure
    let migrate_output = Command::new("npx")
        .args(["drizzle-kit", "migrate"])
        .current_dir(&root)
        .output();

    match migrate_output {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            return CommandResult::err(
                "add:migration",
                format!(
                    "drizzle-kit migrate failed:\n{}",
                    String::from_utf8_lossy(&o.stderr).trim()
                ),
            );
        }
        Err(e) => {
            return CommandResult::err(
                "add:migration",
                format!("Failed to run drizzle-kit migrate: {}", e),
            );
        }
    }

    let mut result = CommandResult::ok("add:migration", vec![]);
    result.next_steps = vec!["Migration applied. Run tsx generate feature to scaffold a new resource.".to_string()];
    result
}
