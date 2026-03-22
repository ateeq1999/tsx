//! Custom Tera filters and functions for forge.
//!
//! Case conversion: snake_case, pascal_case, camel_case, kebab_case
//! Import hoisting: collect_import, collect_import_priority, render_imports
//! String utilities: slugify, truncate, indent
//! Data utilities: json_encode, json_decode, debug
//! Inflection: plural, singular
//! System: env
//! Generation: random_id (function)

use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tera::{Error as TeraError, Result as TeraResult, Value};

use crate::collector;

pub fn snake_case(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or("");
    Ok(Value::String(s.to_snake_case()))
}

pub fn pascal_case(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or("");
    Ok(Value::String(s.to_pascal_case()))
}

pub fn camel_case(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or("");
    Ok(Value::String(s.to_lower_camel_case()))
}

pub fn kebab_case(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or("");
    Ok(Value::String(s.to_kebab_case()))
}

/// Tera filter — adds an import string to the regular collector.
/// Returns an empty string so it can be embedded in template output without leaving a trace.
pub fn collect_import(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or("").to_string();
    Ok(Value::String(collector::collect(s)))
}

/// Tera filter — adds an import string to the priority queue (output before regular imports).
pub fn collect_import_priority(
    value: &Value,
    _args: &HashMap<String, Value>,
) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or("").to_string();
    Ok(Value::String(collector::collect_priority(s)))
}

/// Tera function — drains the import collector and returns the sorted import block.
pub fn render_imports_fn(_args: &HashMap<String, Value>) -> TeraResult<Value> {
    Ok(Value::String(collector::drain()))
}

// ---------------------------------------------------------------------------
// String utilities
// ---------------------------------------------------------------------------

/// Convert a string to a URL-safe slug: lowercase, non-alphanumeric chars → `-`,
/// consecutive dashes collapsed, leading/trailing dashes removed.
pub fn slugify(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or("");
    let slug: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    Ok(Value::String(slug))
}

/// Truncate a string to `length` characters (default 100), appending `...` if cut.
pub fn truncate(value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or("");
    let len = args
        .get("length")
        .and_then(|v| v.as_u64())
        .unwrap_or(100) as usize;
    if s.chars().count() <= len {
        return Ok(Value::String(s.to_string()));
    }
    let truncated: String = s.chars().take(len).collect();
    Ok(Value::String(format!("{truncated}...")))
}

/// Indent every line of a string by `width` spaces (default 2).
pub fn indent(value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or("");
    let width = args
        .get("width")
        .and_then(|v| v.as_u64())
        .unwrap_or(2) as usize;
    let pad = " ".repeat(width);
    let indented = s
        .lines()
        .map(|l| format!("{pad}{l}"))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(Value::String(indented))
}

// ---------------------------------------------------------------------------
// JSON utilities
// ---------------------------------------------------------------------------

/// Serialize the input value to a compact JSON string.
pub fn json_encode(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    serde_json::to_string(value)
        .map(Value::String)
        .map_err(|e| TeraError::msg(format!("json_encode: {e}")))
}

/// Parse a JSON string into a value. Input must be a string.
pub fn json_decode(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value
        .as_str()
        .ok_or_else(|| TeraError::msg("json_decode: input must be a string"))?;
    serde_json::from_str(s).map_err(|e| TeraError::msg(format!("json_decode: {e}")))
}

/// Pretty-print the value as formatted JSON (useful for debugging templates).
pub fn debug_filter(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    serde_json::to_string_pretty(value)
        .map(Value::String)
        .map_err(|e| TeraError::msg(format!("debug: {e}")))
}

// ---------------------------------------------------------------------------
// Inflection
// ---------------------------------------------------------------------------

/// Very simple English pluralization heuristic covering the most common cases.
pub fn plural(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or("");
    Ok(Value::String(pluralize(s)))
}

/// Simple English singularization heuristic.
pub fn singular(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or("");
    Ok(Value::String(singularize(s)))
}

fn pluralize(word: &str) -> String {
    if word.is_empty() {
        return word.to_string();
    }
    let lower = word.to_ascii_lowercase();
    // Already plural guard (naive: ends with 's' but not 'ss', 'us', 'is')
    let irregular: &[(&str, &str)] = &[
        ("person", "people"), ("child", "children"), ("man", "men"),
        ("woman", "women"), ("tooth", "teeth"), ("foot", "feet"),
        ("mouse", "mice"), ("goose", "geese"),
    ];
    for (sing, plur) in irregular {
        if lower == *sing { return plur.to_string(); }
    }
    if lower.ends_with("quiz") { return format!("{word}zes"); }
    if lower.ends_with("ch") || lower.ends_with("sh") || lower.ends_with("ss")
        || lower.ends_with('x') || lower.ends_with('s')
    {
        return format!("{word}es");
    }
    if lower.ends_with('y') && !matches!(lower.chars().rev().nth(1), Some('a'|'e'|'i'|'o'|'u')) {
        return format!("{}ies", &word[..word.len() - 1]);
    }
    if lower.ends_with('f') {
        return format!("{}ves", &word[..word.len() - 1]);
    }
    if lower.ends_with("fe") {
        return format!("{}ves", &word[..word.len() - 2]);
    }
    format!("{word}s")
}

fn singularize(word: &str) -> String {
    if word.is_empty() {
        return word.to_string();
    }
    let lower = word.to_ascii_lowercase();
    let irregular: &[(&str, &str)] = &[
        ("people", "person"), ("children", "child"), ("men", "man"),
        ("women", "woman"), ("teeth", "tooth"), ("feet", "foot"),
        ("mice", "mouse"), ("geese", "goose"),
    ];
    for (plur, sing) in irregular {
        if lower == *plur { return sing.to_string(); }
    }
    if lower.ends_with("ves") {
        return format!("{}f", &word[..word.len() - 3]);
    }
    if lower.ends_with("ies") {
        return format!("{}y", &word[..word.len() - 3]);
    }
    if lower.ends_with("zes") && lower.ends_with("quiz") {
        return word[..word.len() - 2].to_string();
    }
    if lower.ends_with("sses") || lower.ends_with("xes") || lower.ends_with("ches")
        || lower.ends_with("shes")
    {
        return word[..word.len() - 2].to_string();
    }
    if lower.ends_with('s') && !lower.ends_with("ss") {
        return word[..word.len() - 1].to_string();
    }
    word.to_string()
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Read an environment variable by name. Returns an empty string if not set.
pub fn env_filter(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let name = value
        .as_str()
        .ok_or_else(|| TeraError::msg("env: input must be a string (env var name)"))?;
    Ok(Value::String(std::env::var(name).unwrap_or_default()))
}

// ---------------------------------------------------------------------------
// Generation
// ---------------------------------------------------------------------------

/// Generate a short pseudo-random hex ID based on the current system time.
/// Not cryptographically secure — intended for template use only.
pub fn random_id_fn(_args: &HashMap<String, Value>) -> TeraResult<Value> {
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);
    Ok(Value::String(format!("{:016x}", hasher.finish())))
}
