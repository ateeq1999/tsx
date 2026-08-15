//! Helpers for `run`: peer-package slot injection and output-path expansion.

use std::path::{Path, PathBuf};

use crate::utils::paths::get_frameworks_dir;

/// Locate the directory for a package slug by searching builtin, .tsx/frameworks, .tsx/packages.
pub(super) fn find_package_dir(slug: &str, cwd: &Path) -> Option<PathBuf> {
    // Builtin
    let builtin = get_frameworks_dir().join(slug);
    if builtin.is_dir() {
        return Some(builtin);
    }
    // User-installed FPF
    let fpf = cwd.join(".tsx").join("packages").join(slug);
    if fpf.is_dir() {
        return Some(fpf);
    }
    // Legacy
    let legacy = cwd.join(".tsx").join("frameworks").join(slug);
    if legacy.is_dir() {
        return Some(legacy);
    }
    None
}

/// Scan all installed packages and inject slot content into the input JSON.
///
/// For each peer package listed in `stack.packages`:
///   - Load its `manifest.json`
///   - If `integrates_with[current_framework]` exists, get the slot name
///   - Load and render `slots/<slot>.forge` with tsx-forge using `input` as context
///   - Set `input["slot_<name>"]` to the rendered string
pub(super) fn inject_slots(
    input: &mut serde_json::Value,
    current_framework: &str,
    stack: &crate::stack::StackProfile,
    cwd: &Path,
) {
    let package_names = stack.package_names();
    for pkg_name in &package_names {
        // Don't inject a package's slots into itself
        if *pkg_name == current_framework {
            continue;
        }
        let Some(pkg_dir) = find_package_dir(pkg_name, cwd) else {
            continue;
        };
        let manifest_path = pkg_dir.join("manifest.json");
        let Ok(manifest_str) = std::fs::read_to_string(&manifest_path) else {
            continue;
        };
        let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&manifest_str) else {
            continue;
        };
        let Some(integrates) = manifest
            .get("integrates_with")
            .and_then(|v| v.as_object())
        else {
            continue;
        };
        let Some(integration) = integrates.get(current_framework) else {
            continue;
        };
        let Some(slot_name) = integration.get("slot").and_then(|v| v.as_str()) else {
            continue;
        };

        // Load the .forge slot template
        let slot_path = pkg_dir.join("slots").join(format!("{slot_name}.forge"));
        let Ok(template_src) = std::fs::read_to_string(&slot_path) else {
            continue;
        };

        // Render with tsx-forge using the current generator input as context
        let rendered = {
            let mut engine = forge::Engine::new();
            let tpl_key = format!("slot_{slot_name}.forge");
            if engine.add_raw(&tpl_key, &template_src).is_err() {
                continue;
            }
            let mut ctx = forge::ForgeContext::new();
            if let Some(obj) = input.as_object() {
                for (k, v) in obj {
                    if let Some(s) = v.as_str() {
                        ctx.insert_mut(k, s);
                    } else if let Some(b) = v.as_bool() {
                        ctx.insert_mut(k, &b);
                    } else if let Some(n) = v.as_i64() {
                        ctx.insert_mut(k, &n);
                    }
                }
            }
            match engine.render(&tpl_key, &ctx) {
                Ok(s) => s,
                Err(_) => continue,
            }
        };

        // Inject as slot_<name> into the input object
        if let Some(obj) = input.as_object_mut() {
            let key = format!("slot_{slot_name}");
            obj.entry(key).or_insert_with(|| serde_json::json!(rendered));
        }
    }
}

/// Expand `{{field}}` placeholders in a path template using values from the JSON input.
/// If a `PathConfig` is provided, path prefix overrides from `.tsx/stack.json` are applied first.
pub(super) fn expand_path_template(
    template: &str,
    input: &serde_json::Value,
    paths: Option<&crate::stack::PathConfig>,
) -> String {
    // Apply path prefix overrides from stack.json
    let template = if let Some(cfg) = paths {
        apply_path_prefix(template, cfg)
    } else {
        template.to_string()
    };

    let Some(obj) = input.as_object() else {
        return template;
    };

    let mut result = template;
    for (key, value) in obj {
        // Skip internal __style_* vars in path expansion
        if key.starts_with("__") {
            continue;
        }
        let placeholder = format!("{{{{{}}}}}", key);
        if let Some(s) = value.as_str() {
            result = result.replace(&placeholder, s);
        }
    }
    result
}

/// Replace well-known path prefixes with overrides from `.tsx/stack.json`.
/// E.g. if `paths.components = "src/components"`, then `"components/Foo.tsx"` →
/// `"src/components/Foo.tsx"`.
fn apply_path_prefix(template: &str, cfg: &crate::stack::PathConfig) -> String {
    let overrides: &[(&str, Option<&str>)] = &[
        ("components/", Some(cfg.components.as_str())),
        ("routes/", Some(cfg.routes.as_str())),
        ("db/", Some(cfg.db.as_str())),
        ("server-functions/", Some(cfg.server_fns.as_str())),
        ("hooks/", Some(cfg.hooks.as_str())),
    ];
    for (default_prefix, override_dir) in overrides {
        if let Some(dir) = override_dir {
            if template.starts_with(default_prefix) {
                return format!(
                    "{}/{}",
                    dir.trim_end_matches('/'),
                    &template[default_prefix.len()..]
                );
            }
        }
    }
    template.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack::PathConfig;

    #[test]
    fn expand_path_template_replaces_placeholders() {
        let input = serde_json::json!({ "name": "users" });
        let result = expand_path_template("db/schema/{{name}}.ts", &input, None);
        assert_eq!(result, "db/schema/users.ts");
    }

    #[test]
    fn expand_path_template_no_match_unchanged() {
        let input = serde_json::json!({ "name": "users" });
        let result = expand_path_template("src/static.ts", &input, None);
        assert_eq!(result, "src/static.ts");
    }

    #[test]
    fn expand_path_template_handles_non_object_input() {
        let input = serde_json::json!("not-an-object");
        let result = expand_path_template("db/{{name}}.ts", &input, None);
        assert_eq!(result, "db/{{name}}.ts");
    }

    #[test]
    fn path_prefix_override_applied() {
        let cfg = PathConfig {
            components: "src/components".to_string(),
            ..Default::default()
        };
        let input = serde_json::json!({ "name": "Todo" });
        let result = expand_path_template("components/{{name}}Form.tsx", &input, Some(&cfg));
        assert_eq!(result, "src/components/TodoForm.tsx");
    }

    #[test]
    fn path_prefix_applies_default() {
        // PathConfig::default() has components = "app/components", so prefix is rewritten
        let cfg = PathConfig::default();
        let input = serde_json::json!({ "name": "Todo" });
        let result = expand_path_template("components/{{name}}Form.tsx", &input, Some(&cfg));
        assert_eq!(result, "app/components/TodoForm.tsx");
    }

    #[test]
    fn style_vars_not_expanded_in_paths() {
        let mut input = serde_json::json!({ "name": "todo" });
        input["__style_quotes"] = serde_json::json!("double");
        let result = expand_path_template("db/{{name}}.ts", &input, None);
        assert_eq!(result, "db/todo.ts");
    }
}
