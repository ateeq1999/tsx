use crate::output::CommandResult;
use crate::utils::paths::find_project_root;
use std::process::Command;

pub fn add_migration() -> CommandResult {
    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => return CommandResult::err("add:migration", e.to_string()),
    };

    let generate_output = Command::new("npx")
        .args(["drizzle-kit", "generate"])
        .current_dir(&root)
        .output();

    let mut warnings = Vec::new();

    if let Ok(output) = generate_output {
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warnings.push(format!("Generate failed: {}", stderr));
        }
    } else {
        warnings.push("Failed to run drizzle-kit generate".to_string());
    }

    let migrate_output = Command::new("npx")
        .args(["drizzle-kit", "migrate"])
        .current_dir(&root)
        .output();

    if let Ok(output) = migrate_output {
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warnings.push(format!("Migrate failed: {}", stderr));
        }
    } else {
        warnings.push("Failed to run drizzle-kit migrate".to_string());
    }

    let mut result = CommandResult::ok("add:migration", vec![]);
    result.warnings = warnings;
    result
}
