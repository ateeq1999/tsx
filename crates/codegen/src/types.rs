//! Shared types across codegen pipelines.

use serde::{Deserialize, Serialize};

/// Input to any codegen pipeline.
#[derive(Debug, Clone)]
pub enum CodegenInput {
    /// Raw source text (Rust, OpenAPI JSON, Drizzle TS, etc.)
    Source(String),
    /// Path to a file on disk
    File(std::path::PathBuf),
}

impl CodegenInput {
    pub fn source(s: impl Into<String>) -> Self { Self::Source(s.into()) }
    pub fn file(p: impl Into<std::path::PathBuf>) -> Self { Self::File(p.into()) }

    pub fn read(&self) -> Result<String, CodegenError> {
        match self {
            Self::Source(s) => Ok(s.clone()),
            Self::File(p) => std::fs::read_to_string(p)
                .map_err(|e| CodegenError::Io(p.to_string_lossy().to_string(), e.to_string())),
        }
    }
}

/// Output from a codegen pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodegenOutput {
    /// Generated TypeScript / Zod source code
    pub content: String,
    /// Suggested output filename
    pub filename: String,
    /// Names of types/schemas that were emitted
    pub exported_names: Vec<String>,
    /// Any non-fatal warnings encountered during conversion
    pub warnings: Vec<String>,
}

/// Errors from codegen pipelines.
#[derive(Debug)]
pub enum CodegenError {
    ParseError(String),
    Io(String, String),
    UnsupportedConstruct(String),
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::ParseError(s) => write!(f, "Parse error: {}", s),
            CodegenError::Io(path, e) => write!(f, "IO error reading {}: {}", path, e),
            CodegenError::UnsupportedConstruct(s) => write!(f, "Unsupported: {}", s),
        }
    }
}

impl std::error::Error for CodegenError {}

// ---------------------------------------------------------------------------
// Rust type → Zod expression mapping (shared by rust_to_zod + rust_to_ts)
// ---------------------------------------------------------------------------

/// Convert a Rust type string to its Zod equivalent.
pub fn rust_type_to_zod(ty: &str) -> String {
    let ty = ty.trim();
    // Option<T> → z.optional(T) or .nullable()
    if let Some(inner) = strip_wrapper(ty, "Option") {
        return format!("{}.nullable()", rust_type_to_zod(inner));
    }
    // Vec<T> → z.array(T)
    if let Some(inner) = strip_wrapper(ty, "Vec") {
        return format!("z.array({})", rust_type_to_zod(inner));
    }
    // HashMap<K, V> → z.record(K, V)
    if let Some(inner) = strip_wrapper(ty, "HashMap") {
        if let Some((k, v)) = split_two(inner) {
            return format!("z.record({}, {})", rust_type_to_zod(k), rust_type_to_zod(v));
        }
    }
    // BTreeMap<K,V> same
    if let Some(inner) = strip_wrapper(ty, "BTreeMap") {
        if let Some((k, v)) = split_two(inner) {
            return format!("z.record({}, {})", rust_type_to_zod(k), rust_type_to_zod(v));
        }
    }
    match ty {
        "String" | "&str" | "&'static str" => "z.string()".into(),
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
        | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => "z.number().int()".into(),
        "f32" | "f64" => "z.number()".into(),
        "bool" => "z.boolean()".into(),
        "()" => "z.void()".into(),
        "serde_json::Value" | "Value" => "z.unknown()".into(),
        "chrono::DateTime<Utc>" | "DateTime<Utc>" | "NaiveDateTime" => "z.string().datetime()".into(),
        "Uuid" | "uuid::Uuid" => "z.string().uuid()".into(),
        _ => format!("{}Schema", ty), // assume a named schema exists
    }
}

/// Convert a Rust type string to its TypeScript equivalent.
pub fn rust_type_to_ts(ty: &str) -> String {
    let ty = ty.trim();
    if let Some(inner) = strip_wrapper(ty, "Option") {
        return format!("{} | undefined", rust_type_to_ts(inner));
    }
    if let Some(inner) = strip_wrapper(ty, "Vec") {
        return format!("{}[]", rust_type_to_ts(inner));
    }
    if let Some(inner) = strip_wrapper(ty, "HashMap").or_else(|| strip_wrapper(ty, "BTreeMap")) {
        if let Some((k, v)) = split_two(inner) {
            return format!("Record<{}, {}>", rust_type_to_ts(k), rust_type_to_ts(v));
        }
    }
    match ty {
        "String" | "&str" | "&'static str" => "string".into(),
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
        | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
        | "f32" | "f64" => "number".into(),
        "bool" => "boolean".into(),
        "()" => "void".into(),
        "serde_json::Value" | "Value" => "unknown".into(),
        "chrono::DateTime<Utc>" | "DateTime<Utc>" | "NaiveDateTime" => "string".into(),
        "Uuid" | "uuid::Uuid" => "string".into(),
        _ => ty.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

pub(crate) fn strip_wrapper<'a>(ty: &'a str, wrapper: &str) -> Option<&'a str> {
    let prefix = format!("{}<", wrapper);
    if ty.starts_with(&prefix) && ty.ends_with('>') {
        Some(&ty[prefix.len()..ty.len() - 1])
    } else {
        None
    }
}

pub(crate) fn split_two(s: &str) -> Option<(&str, &str)> {
    // Split "K, V" respecting nested angle brackets
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => return Some((&s[..i], s[i + 1..].trim())),
            _ => {}
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests — rust_type_to_zod and rust_type_to_ts mappings
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── Zod primitives ────────────────────────────────────────────────────

    #[test] fn zod_string()      { assert_eq!(rust_type_to_zod("String"),   "z.string()"); }
    #[test] fn zod_str_ref()     { assert_eq!(rust_type_to_zod("&str"),     "z.string()"); }
    #[test] fn zod_i32()         { assert_eq!(rust_type_to_zod("i32"),      "z.number().int()"); }
    #[test] fn zod_i64()         { assert_eq!(rust_type_to_zod("i64"),      "z.number().int()"); }
    #[test] fn zod_u32()         { assert_eq!(rust_type_to_zod("u32"),      "z.number().int()"); }
    #[test] fn zod_u64()         { assert_eq!(rust_type_to_zod("u64"),      "z.number().int()"); }
    #[test] fn zod_usize()       { assert_eq!(rust_type_to_zod("usize"),    "z.number().int()"); }
    #[test] fn zod_f32()         { assert_eq!(rust_type_to_zod("f32"),      "z.number()"); }
    #[test] fn zod_f64()         { assert_eq!(rust_type_to_zod("f64"),      "z.number()"); }
    #[test] fn zod_bool()        { assert_eq!(rust_type_to_zod("bool"),     "z.boolean()"); }
    #[test] fn zod_unit()        { assert_eq!(rust_type_to_zod("()"),       "z.void()"); }
    #[test] fn zod_json_value()  { assert_eq!(rust_type_to_zod("serde_json::Value"), "z.unknown()"); }
    #[test] fn zod_value_short() { assert_eq!(rust_type_to_zod("Value"),    "z.unknown()"); }
    #[test] fn zod_datetime()    { assert_eq!(rust_type_to_zod("chrono::DateTime<Utc>"), "z.string().datetime()"); }
    #[test] fn zod_datetime_short() { assert_eq!(rust_type_to_zod("DateTime<Utc>"), "z.string().datetime()"); }
    #[test] fn zod_naive_dt()    { assert_eq!(rust_type_to_zod("NaiveDateTime"), "z.string().datetime()"); }
    #[test] fn zod_uuid()        { assert_eq!(rust_type_to_zod("Uuid"),     "z.string().uuid()"); }
    #[test] fn zod_uuid_full()   { assert_eq!(rust_type_to_zod("uuid::Uuid"), "z.string().uuid()"); }

    // ── Zod generics ─────────────────────────────────────────────────────

    #[test]
    fn zod_option_string() {
        assert_eq!(rust_type_to_zod("Option<String>"), "z.string().nullable()");
    }

    #[test]
    fn zod_option_i32() {
        assert_eq!(rust_type_to_zod("Option<i32>"), "z.number().int().nullable()");
    }

    #[test]
    fn zod_vec_string() {
        assert_eq!(rust_type_to_zod("Vec<String>"), "z.array(z.string())");
    }

    #[test]
    fn zod_vec_i64() {
        assert_eq!(rust_type_to_zod("Vec<i64>"), "z.array(z.number().int())");
    }

    #[test]
    fn zod_vec_option() {
        assert_eq!(rust_type_to_zod("Vec<Option<String>>"), "z.array(z.string().nullable())");
    }

    #[test]
    fn zod_hashmap_string_i32() {
        assert_eq!(
            rust_type_to_zod("HashMap<String, i32>"),
            "z.record(z.string(), z.number().int())"
        );
    }

    #[test]
    fn zod_btreemap() {
        assert_eq!(
            rust_type_to_zod("BTreeMap<String, bool>"),
            "z.record(z.string(), z.boolean())"
        );
    }

    #[test]
    fn zod_unknown_named_type() {
        // Unknown type → assume a named schema exists
        assert_eq!(rust_type_to_zod("MyType"), "MyTypeSchema");
    }

    // ── TypeScript primitives ─────────────────────────────────────────────

    #[test] fn ts_string()     { assert_eq!(rust_type_to_ts("String"), "string"); }
    #[test] fn ts_str_ref()    { assert_eq!(rust_type_to_ts("&str"),   "string"); }
    #[test] fn ts_i32()        { assert_eq!(rust_type_to_ts("i32"),    "number"); }
    #[test] fn ts_i64()        { assert_eq!(rust_type_to_ts("i64"),    "number"); }
    #[test] fn ts_f64()        { assert_eq!(rust_type_to_ts("f64"),    "number"); }
    #[test] fn ts_bool()       { assert_eq!(rust_type_to_ts("bool"),   "boolean"); }
    #[test] fn ts_unit()       { assert_eq!(rust_type_to_ts("()"),     "void"); }
    #[test] fn ts_json_value() { assert_eq!(rust_type_to_ts("serde_json::Value"), "unknown"); }
    #[test] fn ts_datetime()   { assert_eq!(rust_type_to_ts("chrono::DateTime<Utc>"), "string"); }
    #[test] fn ts_uuid()       { assert_eq!(rust_type_to_ts("Uuid"), "string"); }

    // ── TypeScript generics ───────────────────────────────────────────────

    #[test]
    fn ts_option_string() {
        assert_eq!(rust_type_to_ts("Option<String>"), "string | undefined");
    }

    #[test]
    fn ts_vec_string() {
        assert_eq!(rust_type_to_ts("Vec<String>"), "string[]");
    }

    #[test]
    fn ts_vec_i32() {
        assert_eq!(rust_type_to_ts("Vec<i32>"), "number[]");
    }

    #[test]
    fn ts_hashmap() {
        assert_eq!(
            rust_type_to_ts("HashMap<String, bool>"),
            "Record<string, boolean>"
        );
    }

    #[test]
    fn ts_btreemap() {
        assert_eq!(
            rust_type_to_ts("BTreeMap<String, i64>"),
            "Record<string, number>"
        );
    }

    // ── Helpers ───────────────────────────────────────────────────────────

    #[test]
    fn strip_wrapper_option() {
        assert_eq!(strip_wrapper("Option<String>", "Option"), Some("String"));
    }

    #[test]
    fn strip_wrapper_nested() {
        assert_eq!(strip_wrapper("Vec<Option<i32>>", "Vec"), Some("Option<i32>"));
    }

    #[test]
    fn strip_wrapper_no_match() {
        assert_eq!(strip_wrapper("String", "Option"), None);
    }

    #[test]
    fn split_two_simple() {
        assert_eq!(split_two("String, i32"), Some(("String", "i32")));
    }

    #[test]
    fn split_two_nested() {
        assert_eq!(split_two("String, Vec<i32>"), Some(("String", "Vec<i32>")));
    }

    #[test]
    fn split_two_no_comma() {
        assert_eq!(split_two("String"), None);
    }
}
