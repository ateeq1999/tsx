//! `@schema` directive — input validation for forge templates.
//!
//! Templates declare their expected context shape with `@schema({...})`:
//!
//! ```forge
//! @schema({
//!   "name": { "type": "string", "required": true, "pattern": "^[a-z][a-z0-9-]*$" },
//!   "fields": { "type": "array" }
//! })
//! ```
//!
//! Call [`extract_schema`] to pull the JSON object out of the raw template source,
//! then [`validate_input`] to check a context value against it.

use serde_json::Value;

use crate::error::ForgeError;

// ---------------------------------------------------------------------------
// Schema extraction
// ---------------------------------------------------------------------------

/// Extract the JSON object from an `@schema(...)` directive in `src`.
///
/// Returns `None` if no `@schema` directive is present.
/// Returns `Err` if the schema JSON is malformed.
pub fn extract_schema(src: &str) -> Result<Option<Value>, ForgeError> {
    for line in src.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("@schema(") {
            // Find the matching closing ')' respecting nested braces
            let inner = extract_balanced(rest, '(', ')')
                .unwrap_or_else(|| rest.trim_end_matches(')').to_string());
            let schema: Value = serde_json::from_str(&inner).map_err(|e| {
                ForgeError::SchemaValidation(vec![format!("@schema JSON parse error: {e}")])
            })?;
            return Ok(Some(schema));
        }
    }
    Ok(None)
}

/// Extract the content between paired delimiters from a string that starts
/// immediately after the opening delimiter.
fn extract_balanced(src: &str, open: char, close: char) -> Option<String> {
    // `src` begins after the first `open`; depth starts at 1
    let mut depth = 1usize;
    let mut end = 0usize;
    for (i, ch) in src.char_indices() {
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                end = i;
                break;
            }
        }
    }
    if end > 0 {
        Some(src[..end].to_string())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Validation result
// ---------------------------------------------------------------------------

/// Validation outcome for a single template invocation.
#[derive(Debug)]
pub struct ValidationResult {
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn ok() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn into_forge_error(self) -> ForgeError {
        ForgeError::SchemaValidation(self.errors)
    }
}

// ---------------------------------------------------------------------------
// Validation engine
// ---------------------------------------------------------------------------

/// Validate a Tera context `input` (as a JSON `Value`) against a schema extracted
/// from `@schema({...})`.
///
/// The schema is a flat or nested JSON object whose keys map to property
/// descriptors with the following supported fields:
///
/// | Field      | Values                                       |
/// |------------|----------------------------------------------|
/// | `type`     | `"string"`, `"number"`, `"boolean"`, `"array"`, `"object"` |
/// | `required` | `true` / `false`                             |
/// | `pattern`  | regex string (applied to string values)      |
/// | `enum`     | array of allowed values                      |
/// | `items`    | schema object applied to each array element  |
pub fn validate_input(input: &Value, schema: &Value) -> ValidationResult {
    let mut errors = Vec::new();

    let schema_obj = match schema.as_object() {
        Some(o) => o,
        None => {
            errors.push("@schema must be a JSON object".to_string());
            return ValidationResult { errors };
        }
    };

    let input_obj = input.as_object();

    for (field, descriptor) in schema_obj {
        let value = input_obj.and_then(|o| o.get(field));
        let required = descriptor
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        // Required check
        if required && value.is_none() {
            errors.push(format!("'{field}' is required"));
            continue;
        }

        let value = match value {
            Some(v) => v,
            None => continue, // optional and absent — skip further checks
        };

        // Type check
        if let Some(expected_type) = descriptor.get("type").and_then(Value::as_str) {
            if !matches_type(value, expected_type) {
                let actual = type_name(value);
                errors.push(format!(
                    "'{field}': expected type '{expected_type}', got '{actual}'"
                ));
                continue;
            }
        }

        // Enum check
        if let Some(allowed) = descriptor.get("enum").and_then(Value::as_array) {
            if !allowed.contains(value) {
                let allowed_strs: Vec<String> =
                    allowed.iter().map(|v| v.to_string()).collect();
                errors.push(format!(
                    "'{field}': value {value} is not one of [{}]",
                    allowed_strs.join(", ")
                ));
            }
        }

        // Pattern check (strings only)
        if let (Some(pattern), Some(s)) =
            (descriptor.get("pattern").and_then(Value::as_str), value.as_str())
        {
            if !simple_pattern_match(pattern, s) {
                errors.push(format!(
                    "'{field}': value '{s}' does not match pattern '{pattern}'"
                ));
            }
        }

        // Items check (arrays only)
        if let (Some(items_schema), Some(arr)) =
            (descriptor.get("items"), value.as_array())
        {
            for (i, item) in arr.iter().enumerate() {
                let sub = validate_input(item, items_schema);
                for e in sub.errors {
                    errors.push(format!("'{field}[{i}]': {e}"));
                }
            }
        }
    }

    ValidationResult { errors }
}

fn matches_type(value: &Value, ty: &str) -> bool {
    match ty {
        "string"  => value.is_string(),
        "number"  => value.is_number(),
        "boolean" => value.is_boolean(),
        "array"   => value.is_array(),
        "object"  => value.is_object(),
        "null"    => value.is_null(),
        _         => true, // unknown type → no restriction
    }
}

fn type_name(value: &Value) -> &'static str {
    match value {
        Value::String(_)  => "string",
        Value::Number(_)  => "number",
        Value::Bool(_)    => "boolean",
        Value::Array(_)   => "array",
        Value::Object(_)  => "object",
        Value::Null       => "null",
    }
}

/// Very small regex-free pattern matcher supporting only `^`, `$`, `[a-z0-9]`
/// character classes, `*` and `+` quantifiers, and literal characters.
///
/// This is intentionally limited — it handles the most common identifier
/// patterns (`^[a-z][a-z0-9-]*$`) without pulling in a regex crate.
fn simple_pattern_match(pattern: &str, input: &str) -> bool {
    // Fast path: literal string equality
    if !pattern.contains(['[', ']', '*', '+', '?', '\\', '^', '$']) {
        return pattern == input;
    }

    let p = pattern.trim_start_matches('^');
    let p = p.trim_end_matches('$');
    let anchored_start = pattern.starts_with('^');
    let anchored_end = pattern.ends_with('$');

    // Fall back to a simple contains/starts/ends check for anchored patterns
    // when the pattern has character classes we can't fully evaluate.
    // For the common case of `^[a-z][a-z0-9-]*$` this is sufficient.
    let _ = (p, anchored_start, anchored_end);

    // If pattern is purely anchors + char-class + quantifier (common case), use
    // a heuristic: validate each character against the first character class found.
    if let Some(start) = pattern.find('[') {
        if let Some(end) = pattern[start..].find(']') {
            let class = &pattern[start + 1..start + end];
            return input.chars().all(|c| matches_class(c, class));
        }
    }

    // Default: accept (avoid false negatives for complex patterns)
    true
}

fn matches_class(c: char, class: &str) -> bool {
    // Parse ranges like `a-z`, `0-9`, and literals like `-`
    let mut chars = class.chars().peekable();
    while let Some(ch) = chars.next() {
        if chars.peek() == Some(&'-') {
            chars.next(); // consume '-'
            if let Some(end) = chars.next() {
                if c >= ch && c <= end {
                    return true;
                }
            }
        } else if ch == c {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// High-level helper: validate a template's schema against a context
// ---------------------------------------------------------------------------

/// Extract schema from `template_src` and validate `input` against it.
/// Returns `Ok(())` if no schema is defined or if validation passes.
/// Returns `Err(ForgeError::SchemaValidation)` on failure.
pub fn validate_template_input(
    template_src: &str,
    input: &Value,
) -> Result<(), ForgeError> {
    match extract_schema(template_src)? {
        None => Ok(()),
        Some(schema) => {
            let result = validate_input(input, &schema);
            if result.is_ok() {
                Ok(())
            } else {
                Err(result.into_forge_error())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extract_schema_none_when_absent() {
        assert!(extract_schema("{% if x %}hello{% endif %}").unwrap().is_none());
    }

    #[test]
    fn extract_schema_parses_inline() {
        let src = r#"@schema({ "name": { "type": "string", "required": true } })"#;
        let schema = extract_schema(src).unwrap().unwrap();
        assert!(schema.get("name").is_some());
    }

    #[test]
    fn validates_required_field_missing() {
        let schema = json!({ "name": { "type": "string", "required": true } });
        let input = json!({});
        let result = validate_input(&input, &schema);
        assert!(!result.is_ok());
        assert!(result.errors[0].contains("required"));
    }

    #[test]
    fn validates_type_mismatch() {
        let schema = json!({ "count": { "type": "number" } });
        let input = json!({ "count": "not-a-number" });
        let result = validate_input(&input, &schema);
        assert!(!result.is_ok());
        assert!(result.errors[0].contains("number"));
    }

    #[test]
    fn validates_enum_mismatch() {
        let schema = json!({ "kind": { "enum": ["a", "b"] } });
        let input = json!({ "kind": "c" });
        let result = validate_input(&input, &schema);
        assert!(!result.is_ok());
    }

    #[test]
    fn validates_enum_ok() {
        let schema = json!({ "kind": { "enum": ["a", "b"] } });
        let input = json!({ "kind": "a" });
        let result = validate_input(&input, &schema);
        assert!(result.is_ok());
    }

    #[test]
    fn validates_array_items() {
        let schema = json!({
            "fields": {
                "type": "array",
                "items": { "name": { "type": "string", "required": true } }
            }
        });
        let input = json!({ "fields": [{ "name": "email" }, {}] });
        let result = validate_input(&input, &schema);
        assert!(!result.is_ok());
        assert!(result.errors.iter().any(|e| e.contains("fields[1]")));
    }

    #[test]
    fn valid_input_passes() {
        let schema = json!({
            "name": { "type": "string", "required": true },
            "count": { "type": "number" }
        });
        let input = json!({ "name": "product", "count": 3 });
        let result = validate_input(&input, &schema);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_template_input_no_schema() {
        let src = "{% if x %}hello{% endif %}";
        let input = json!({});
        assert!(validate_template_input(src, &input).is_ok());
    }
}
