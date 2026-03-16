//! Custom Tera filters and functions for forge.
//!
//! Case conversion: snake_case, pascal_case, camel_case, kebab_case
//! Import hoisting: collect_import, collect_import_priority, render_imports

use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use std::collections::HashMap;
use tera::{Result as TeraResult, Value};

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
