use std::path::Path;

use crate::plugin::PluginManifest;

/// Validation error returned when a plugin fails the security or schema checks.
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// Validate a plugin directory.
///
/// Checks:
/// 1. `plugin.json` is parseable and all required fields are present.
/// 2. `templates/` directory exists.
/// 3. Each override path declared in `plugin.json` points to a file that exists.
/// 4. Generator template paths exist.
/// 5. Package name follows safe npm naming (`[a-z0-9-]` with optional `@scope/`).
/// 6. No path traversal (`..`) in override or generator template paths.
pub fn validate_plugin(dir: &Path) -> Result<PluginManifest, Vec<ValidationError>> {
    let mut errors: Vec<ValidationError> = Vec::new();

    let manifest = match PluginManifest::load(dir) {
        Ok(m) => m,
        Err(e) => {
            return Err(vec![ValidationError {
                field: "plugin.json".to_string(),
                message: e.to_string(),
            }]);
        }
    };

    // Required fields
    if manifest.name.is_empty() {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Plugin name is required".to_string(),
        });
    }
    if manifest.version.is_empty() {
        errors.push(ValidationError {
            field: "version".to_string(),
            message: "Plugin version is required".to_string(),
        });
    }
    if manifest.package.is_empty() {
        errors.push(ValidationError {
            field: "package".to_string(),
            message: "Package name is required".to_string(),
        });
    }

    // Package name safety: allow @scope/name or plain name
    let pkg = manifest.package.trim_start_matches('@');
    let pkg_clean = pkg.split('/').last().unwrap_or(pkg);
    if !pkg_clean
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        errors.push(ValidationError {
            field: "package".to_string(),
            message: format!(
                "Package name '{}' contains unsafe characters",
                manifest.package
            ),
        });
    }

    // templates/ directory
    if !dir.join("templates").is_dir() {
        errors.push(ValidationError {
            field: "templates/".to_string(),
            message: "Plugin must contain a templates/ directory".to_string(),
        });
    }

    // Path traversal check + file existence for overrides
    for (generator_id, template_path) in &manifest.overrides {
        if template_path.contains("..") {
            errors.push(ValidationError {
                field: format!("overrides.{}", generator_id),
                message: format!("Path traversal ('..') not allowed in override path '{}'", template_path),
            });
        } else {
            let full = dir.join(template_path);
            if !full.is_file() {
                errors.push(ValidationError {
                    field: format!("overrides.{}", generator_id),
                    message: format!("Override template '{}' not found in plugin", template_path),
                });
            }
        }
    }

    // Validate generator template paths
    for gen in &manifest.generators {
        if gen.template.contains("..") {
            errors.push(ValidationError {
                field: format!("generators.{}.template", gen.id),
                message: format!("Path traversal not allowed in generator template '{}'", gen.template),
            });
        } else {
            let full = dir.join(&gen.template);
            if !full.is_file() {
                errors.push(ValidationError {
                    field: format!("generators.{}.template", gen.id),
                    message: format!("Generator template '{}' not found in plugin", gen.template),
                });
            }
        }
    }

    if errors.is_empty() {
        Ok(manifest)
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::manifest::{PluginGenerator, PluginManifest};
    use tempfile::TempDir;

    fn make_valid_plugin(dir: &Path) -> PluginManifest {
        std::fs::create_dir_all(dir.join("templates/features")).unwrap();
        std::fs::write(dir.join("templates/features/custom.jinja"), "").unwrap();

        let manifest = PluginManifest {
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            description: "desc".to_string(),
            package: "tsx-plugin-test".to_string(),
            tsx_version: ">=0.1.0".to_string(),
            overrides: std::collections::HashMap::new(),
            generators: vec![],
            peer_dependencies: vec![],
            author: "".to_string(),
            docs: "".to_string(),
        };
        manifest.save(dir).unwrap();
        manifest
    }

    #[test]
    fn valid_plugin_passes() {
        let dir = TempDir::new().unwrap();
        make_valid_plugin(dir.path());
        assert!(validate_plugin(dir.path()).is_ok());
    }

    #[test]
    fn path_traversal_is_rejected() {
        let dir = TempDir::new().unwrap();
        let mut m = make_valid_plugin(dir.path());
        m.overrides
            .insert("add:schema".to_string(), "../etc/passwd".to_string());
        m.save(dir.path()).unwrap();
        let errs = validate_plugin(dir.path()).unwrap_err();
        assert!(errs.iter().any(|e| e.message.contains("traversal")));
    }

    #[test]
    fn unsafe_package_name_rejected() {
        let dir = TempDir::new().unwrap();
        let mut m = make_valid_plugin(dir.path());
        m.package = "bad name!".to_string();
        m.save(dir.path()).unwrap();
        let errs = validate_plugin(dir.path()).unwrap_err();
        assert!(errs.iter().any(|e| e.field == "package"));
    }
}
