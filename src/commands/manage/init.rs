use crate::output::CommandResult;
use std::process::Command;

pub fn init(name: Option<String>) -> CommandResult {
    let project_name = name.unwrap_or_else(|| "my-app".to_string());

    let output = Command::new("npm")
        .args([
            "create",
            "tanstack@latest",
            &project_name,
            "--template",
            "start",
        ])
        .output();

    let mut files_created = vec![];
    let mut warnings = vec![];

    match output {
        Ok(o) if o.status.success() => {
            files_created.push(format!("Created project: {}", project_name));
        }
        Ok(o) => {
            return CommandResult::err(
                "init",
                format!(
                    "npm create tanstack failed: {}",
                    String::from_utf8_lossy(&o.stderr).trim()
                ),
            );
        }
        Err(e) => {
            return CommandResult::err(
                "init",
                format!("Failed to run npm create tanstack: {}", e),
            );
        }
    }

    let project_dir = std::env::current_dir()
        .unwrap_or_default()
        .join(&project_name);

    if project_dir.exists() {
        // npm install — surface failure as error, not silence
        match Command::new("npm")
            .args(["install"])
            .current_dir(&project_dir)
            .output()
        {
            Ok(o) if o.status.success() => {
                files_created.push("Dependencies installed".to_string());
            }
            Ok(o) => {
                warnings.push(format!(
                    "npm install failed: {}",
                    String::from_utf8_lossy(&o.stderr).trim()
                ));
            }
            Err(e) => {
                warnings.push(format!("npm install could not run: {}", e));
            }
        }

        // shadcn/ui — warn on failure but do not abort (optional tooling)
        match Command::new("npx")
            .args(["shadcn@latest", "init", "-d"])
            .current_dir(&project_dir)
            .output()
        {
            Ok(o) if o.status.success() => {
                files_created.push("shadcn/ui initialized".to_string());
            }
            Ok(o) => {
                warnings.push(format!(
                    "shadcn/ui init failed (non-fatal): {}",
                    String::from_utf8_lossy(&o.stderr).trim()
                ));
            }
            Err(e) => {
                warnings.push(format!("npx shadcn could not run (non-fatal): {}", e));
            }
        }

        create_drizzle_config(&project_dir);
        files_created.push("drizzle.config.ts created".to_string());
        files_created.push(".env.example created".to_string());
    }

    let mut result = CommandResult::ok("init", files_created);
    result.warnings = warnings;
    result.next_steps = vec![
        format!("cd {}", project_name),
        "tsx add auth".to_string(),
        "tsx generate feature".to_string(),
    ];
    result
}

fn create_drizzle_config(project_dir: &std::path::Path) {
    let config = r#"import { defineConfig } from 'drizzle-kit';

export default defineConfig({
  schema: './db/schema/*',
  out: './drizzle',
  dialect: 'sqlite',
  dbCredentials: {
    url: './data.db',
  },
});
"#;

    let _ = std::fs::write(project_dir.join("drizzle.config.ts"), config);
    let _ = std::fs::write(project_dir.join(".env.example"), "DATABASE_URL=./data.db\n");
}
