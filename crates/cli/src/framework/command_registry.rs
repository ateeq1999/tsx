use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::utils::paths::get_frameworks_dir;

/// A generator specification loaded from a framework's `generators/<id>.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorSpec {
    pub id: String,
    pub command: String,
    pub description: String,
    #[serde(default)]
    pub token_estimate: Option<u32>,
    #[serde(default)]
    pub schema: Option<serde_json::Value>,
    /// Glob-style output path templates, e.g. `"db/schema/{{name}}.ts"`.
    #[serde(default)]
    pub output_paths: Vec<String>,
    /// Suggested follow-up steps shown after a successful run.
    #[serde(default)]
    pub next_steps: Vec<String>,
    /// Injected at load time — not present in the JSON file.
    #[serde(skip_deserializing, default)]
    pub framework: String,
}

/// Registry of all generator commands across all installed frameworks.
pub struct CommandRegistry {
    /// Keyed by `spec.id`.
    commands: HashMap<String, GeneratorSpec>,
}

impl CommandRegistry {
    /// Load all generators from the builtin frameworks directory and any user-installed ones.
    pub fn load_all() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };

        // Builtin frameworks shipped with the binary
        let builtin = get_frameworks_dir();
        registry.scan_dir(&builtin);

        if let Ok(cwd) = std::env::current_dir() {
            // Legacy: user-installed frameworks under .tsx/frameworks/
            let user_fw = cwd.join(".tsx").join("frameworks");
            if user_fw.is_dir() {
                registry.scan_dir(&user_fw);
            }

            // New: packages installed via `tsx registry install` under .tsx/packages/
            let user_pkgs = cwd.join(".tsx").join("packages");
            if user_pkgs.is_dir() {
                registry.scan_dir(&user_pkgs);
            }
        }

        registry
    }

    fn scan_dir(&mut self, dir: &std::path::Path) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let fw_path = entry.path();
            if fw_path.is_dir() {
                let fw_name = fw_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                self.load_framework_generators(&fw_path, &fw_name);
            }
        }
    }

    fn load_framework_generators(&mut self, fw_path: &std::path::Path, fw_name: &str) {
        let gen_dir = fw_path.join("generators");
        let Ok(entries) = std::fs::read_dir(&gen_dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(mut spec) = serde_json::from_str::<GeneratorSpec>(&content) {
                        spec.framework = fw_name.to_string();
                        self.commands.insert(spec.id.clone(), spec);
                    }
                }
            }
        }
    }

    /// Resolve by generator id (e.g. `"add-schema"`) or command name (e.g. `"add:schema"`).
    pub fn resolve(&self, id: &str) -> Option<&GeneratorSpec> {
        self.commands
            .get(id)
            .or_else(|| self.commands.values().find(|s| s.command == id))
    }

    /// All specs sorted by framework then id.
    pub fn all(&self) -> Vec<&GeneratorSpec> {
        let mut specs: Vec<&GeneratorSpec> = self.commands.values().collect();
        specs.sort_by(|a, b| a.framework.cmp(&b.framework).then(a.id.cmp(&b.id)));
        specs
    }

    /// All specs for a specific framework slug.
    pub fn for_framework(&self, slug: &str) -> Vec<&GeneratorSpec> {
        let mut specs: Vec<&GeneratorSpec> = self
            .commands
            .values()
            .filter(|s| s.framework == slug)
            .collect();
        specs.sort_by(|a, b| a.id.cmp(&b.id));
        specs
    }
}

/// Validate a JSON object against a simplified JSON Schema subset.
/// Returns human-readable error messages (empty list = valid).
pub fn validate_input(input: &serde_json::Value, schema: &serde_json::Value) -> Vec<String> {
    let mut errors = Vec::new();

    let Some(schema_obj) = schema.as_object() else {
        return errors;
    };

    let Some(input_obj) = input.as_object() else {
        errors.push("input must be a JSON object".to_string());
        return errors;
    };

    // Required field presence check
    if let Some(required) = schema_obj.get("required").and_then(|v| v.as_array()) {
        for req in required {
            if let Some(field) = req.as_str() {
                if !input_obj.contains_key(field) {
                    errors.push(format!("missing required field '{}'", field));
                }
            }
        }
    }

    // Per-property type and enum checks
    if let Some(props) = schema_obj.get("properties").and_then(|v| v.as_object()) {
        for (key, prop_schema) in props {
            let Some(value) = input_obj.get(key) else {
                continue;
            };

            if let Some(type_str) = prop_schema.get("type").and_then(|v| v.as_str()) {
                let ok = match type_str {
                    "string" => value.is_string(),
                    "number" | "integer" => value.is_number(),
                    "boolean" => value.is_boolean(),
                    "array" => value.is_array(),
                    "object" => value.is_object(),
                    "null" => value.is_null(),
                    _ => true,
                };
                if !ok {
                    errors.push(format!(
                        "field '{}': expected {}, got {}",
                        key,
                        type_str,
                        json_type_name(value)
                    ));
                }
            }

            if let Some(enum_values) = prop_schema.get("enum").and_then(|v| v.as_array()) {
                if !enum_values.contains(value) {
                    let allowed: Vec<String> = enum_values
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| format!("'{}'", s)))
                        .collect();
                    errors.push(format!(
                        "field '{}': must be one of [{}]",
                        key,
                        allowed.join(", ")
                    ));
                }
            }
        }
    }

    errors
}

/// Fill missing fields from `properties[*].default` in a JSON Schema.
pub fn apply_defaults(input: &mut serde_json::Value, schema: &serde_json::Value) {
    let Some(props) = schema.get("properties").and_then(|v| v.as_object()) else {
        return;
    };
    let Some(obj) = input.as_object_mut() else {
        return;
    };
    for (key, prop_schema) in props {
        if !obj.contains_key(key) {
            if let Some(default_val) = prop_schema.get("default") {
                obj.insert(key.clone(), default_val.clone());
            }
        }
    }
}

fn json_type_name(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::String(_) => "string",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
        serde_json::Value::Null => "null",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validate_missing_required_field() {
        let schema = json!({ "type": "object", "required": ["name"], "properties": { "name": { "type": "string" } } });
        let errors = validate_input(&json!({}), &schema);
        assert_eq!(errors, vec!["missing required field 'name'"]);
    }

    #[test]
    fn validate_type_mismatch() {
        let schema = json!({ "type": "object", "properties": { "count": { "type": "number" } } });
        let errors = validate_input(&json!({ "count": "five" }), &schema);
        assert!(errors[0].contains("expected number"), "{:?}", errors);
    }

    #[test]
    fn validate_enum_violation() {
        let schema = json!({
            "type": "object",
            "properties": { "kind": { "type": "string", "enum": ["a", "b"] } }
        });
        let errors = validate_input(&json!({ "kind": "c" }), &schema);
        assert!(errors[0].contains("must be one of"), "{:?}", errors);
    }

    #[test]
    fn validate_passes_for_valid_input() {
        let schema = json!({
            "type": "object",
            "required": ["name"],
            "properties": {
                "name": { "type": "string" },
                "timestamps": { "type": "boolean" }
            }
        });
        let errors = validate_input(&json!({ "name": "users", "timestamps": true }), &schema);
        assert!(errors.is_empty(), "{:?}", errors);
    }

    #[test]
    fn apply_defaults_fills_missing() {
        let schema = json!({ "type": "object", "properties": { "ts": { "type": "boolean", "default": true } } });
        let mut input = json!({ "name": "users" });
        apply_defaults(&mut input, &schema);
        assert_eq!(input["ts"], json!(true));
    }

    #[test]
    fn apply_defaults_does_not_overwrite_existing() {
        let schema = json!({ "type": "object", "properties": { "ts": { "type": "boolean", "default": true } } });
        let mut input = json!({ "ts": false });
        apply_defaults(&mut input, &schema);
        assert_eq!(input["ts"], json!(false));
    }
}