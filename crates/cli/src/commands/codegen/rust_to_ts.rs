use std::path::PathBuf;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

use super::rust_parser::parse_rust_items;
use super::types::{RustEnum, RustItem, RustStruct};

pub fn codegen_rust_to_ts(
    input: Option<String>,
    out: Option<String>,
    watch: bool,
    verbose: bool,
) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Resolve input path
    let input_path = match &input {
        Some(p) => PathBuf::from(p),
        None => {
            // Try common defaults: crates/shared/src/lib.rs or src/lib.rs
            let candidates = [
                cwd.join("crates/shared/src/lib.rs"),
                cwd.join("src/lib.rs"),
            ];
            match candidates.into_iter().find(|p| p.exists()) {
                Some(p) => p,
                None => {
                    return ResponseEnvelope::error(
                        "codegen rust-to-ts",
                        ErrorResponse::new(
                            ErrorCode::ProjectNotFound,
                            "No input file specified and could not auto-detect a Rust source file. Use --input <path>",
                        ),
                        0,
                    )
                }
            }
        }
    };

    if !input_path.exists() {
        return ResponseEnvelope::error(
            "codegen rust-to-ts",
            ErrorResponse::new(
                ErrorCode::ProjectNotFound,
                format!("Input file not found: {}", input_path.display()),
            ),
            0,
        );
    }

    let source = match std::fs::read_to_string(&input_path) {
        Ok(s) => s,
        Err(e) => {
            return ResponseEnvelope::error(
                "codegen rust-to-ts",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Failed to read {}: {}", input_path.display(), e),
                ),
                0,
            )
        }
    };

    // Parse & generate
    let items = parse_rust_items(&source);

    if items.is_empty() {
        return ResponseEnvelope::error(
            "codegen rust-to-ts",
            ErrorResponse::new(
                ErrorCode::ValidationError,
                format!(
                    "No serializable structs or enums found in {}. \
                     Make sure types are annotated with #[derive(Serialize, Deserialize)].",
                    input_path.display()
                ),
            ),
            0,
        );
    }

    let generated = generate_ts_output(&items, &input_path, verbose);

    // Determine output path
    let out_path = if let Some(o) = &out {
        PathBuf::from(o)
    } else {
        let stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("types");
        cwd.join("generated").join(format!("{}.ts", stem))
    };

    // Write output
    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    match std::fs::write(&out_path, &generated) {
        Ok(_) => {
            let result = serde_json::json!({
                "input": input_path.to_string_lossy(),
                "output": out_path.to_string_lossy(),
                "items_generated": items.len(),
                "watch": watch,
                "preview": generated.lines().take(20).collect::<Vec<_>>().join("\n"),
            });
            ResponseEnvelope::success("codegen rust-to-ts", result, 0).with_next_steps(vec![
                format!("Generated {} types in {}", items.len(), out_path.display()),
                "Import the Zod schemas for runtime validation".to_string(),
                if watch {
                    "Watch mode enabled — re-runs on file change (not yet implemented, re-run manually)".to_string()
                } else {
                    format!("Re-run with --watch to regenerate automatically on changes to {}", input_path.display())
                },
            ])
        }
        Err(e) => ResponseEnvelope::error(
            "codegen rust-to-ts",
            ErrorResponse::new(
                ErrorCode::InternalError,
                format!("Failed to write {}: {}", out_path.display(), e),
            ),
            0,
        ),
    }
}

// ---------------------------------------------------------------------------
// TypeScript / Zod output generator
// ---------------------------------------------------------------------------

fn generate_ts_output(items: &[RustItem], input_path: &std::path::Path, _verbose: bool) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "// AUTO-GENERATED from {}\n// Do not edit manually — re-run `tsx codegen rust-to-ts`\n\n",
        input_path.display()
    ));
    out.push_str("import { z } from \"zod\"\n\n");

    for item in items {
        match item {
            RustItem::Struct(s) => {
                out.push_str(&generate_struct_zod(s));
                out.push('\n');
            }
            RustItem::Enum(e) => {
                out.push_str(&generate_enum_zod(e));
                out.push('\n');
            }
        }
    }

    out
}

fn generate_struct_zod(s: &RustStruct) -> String {
    let mut out = String::new();
    let schema_name = format!("{}Schema", s.name);

    out.push_str(&format!("export const {} = z.object({{\n", schema_name));

    for field in &s.fields {
        let key = field.serde_rename.as_deref().unwrap_or(&field.name);
        let zod_type = rust_type_to_zod(&field.rust_type);
        let mut zod_expr = if field.is_optional {
            format!("{}.optional()", zod_type)
        } else {
            zod_type
        };
        if field.has_default {
            zod_expr = format!("{}.default(undefined)", zod_expr);
        }
        if field.is_flatten {
            // For flattened fields, add a comment — proper spread would require manual editing
            out.push_str(&format!(
                "  // {} (flattened from {} — merge manually if needed)\n",
                key, field.rust_type
            ));
            continue;
        }
        out.push_str(&format!("  {}: {},\n", key, zod_expr));
    }

    out.push_str("})\n\n");
    out.push_str(&format!(
        "export type {} = z.infer<typeof {}>\n",
        s.name, schema_name
    ));

    out
}

fn generate_enum_zod(e: &RustEnum) -> String {
    let mut out = String::new();
    let schema_name = format!("{}Schema", e.name);

    if e.is_unit_only {
        // Simple string enum
        let variants: Vec<String> = e
            .variants
            .iter()
            .map(|v| {
                let name = v.serde_rename.as_deref().unwrap_or(&v.name);
                format!("\"{}\"", name)
            })
            .collect();
        out.push_str(&format!(
            "export const {} = z.enum([{}])\n\n",
            schema_name,
            variants.join(", ")
        ));
        out.push_str(&format!(
            "export type {} = z.infer<typeof {}>\n",
            e.name, schema_name
        ));
    } else {
        // Tagged union — emit a discriminated union if all variants have payloads,
        // otherwise fall back to z.union([...])
        let schemas: Vec<String> = e
            .variants
            .iter()
            .map(|v| {
                let tag = v.serde_rename.as_deref().unwrap_or(&v.name);
                if v.payload.is_some() {
                    format!(
                        "z.object({{ tag: z.literal(\"{}\"), data: z.unknown() }})",
                        tag
                    )
                } else {
                    format!("z.literal(\"{}\")", tag)
                }
            })
            .collect();

        out.push_str(&format!(
            "export const {} = z.union([\n  {}\n])\n\n",
            schema_name,
            schemas.join(",\n  ")
        ));
        out.push_str(&format!(
            "export type {} = z.infer<typeof {}>\n",
            e.name, schema_name
        ));
    }

    out
}

/// Map a Rust type string to a Zod expression.
fn rust_type_to_zod(rust_type: &str) -> String {
    let t = rust_type.trim();

    // Option<T> → inner T (caller handles .optional())
    if let Some(inner) = strip_generic(t, "Option") {
        return rust_type_to_zod(inner);
    }

    // Vec<T> → z.array(T)
    if let Some(inner) = strip_generic(t, "Vec") {
        return format!("z.array({})", rust_type_to_zod(inner));
    }

    // HashMap<K, V> / BTreeMap<K, V> → z.record(K, V)
    if let Some(inner) = strip_generic(t, "HashMap").or_else(|| strip_generic(t, "BTreeMap")) {
        if let Some(comma_pos) = inner.find(',') {
            let _k = rust_type_to_zod(inner[..comma_pos].trim());
            let v = rust_type_to_zod(inner[comma_pos + 1..].trim());
            return format!("z.record({})", v);
        }
    }

    // Box<T> / Arc<T> / Rc<T> → unwrap
    for wrapper in &["Box", "Arc", "Rc", "Cow"] {
        if let Some(inner) = strip_generic(t, wrapper) {
            return rust_type_to_zod(inner);
        }
    }

    match t {
        "String" | "&str" | "&'static str" => "z.string()".to_string(),
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
        | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => "z.number().int()".to_string(),
        "f32" | "f64" => "z.number()".to_string(),
        "bool" => "z.boolean()".to_string(),
        "()" => "z.null()".to_string(),
        "serde_json::Value" | "Value" => "z.unknown()".to_string(),
        "chrono::DateTime<Utc>" | "DateTime<Utc>" | "DateTime<Local>" => {
            "z.string().datetime()".to_string()
        }
        "NaiveDate" | "chrono::NaiveDate" => "z.string()".to_string(),
        "Uuid" | "uuid::Uuid" => "z.string().uuid()".to_string(),
        "Decimal" | "rust_decimal::Decimal" => "z.string()".to_string(), // serialized as string
        _ => {
            // Assume it's a named type with its own Schema
            let clean = t.split("::").last().unwrap_or(t);
            format!("{}Schema", clean)
        }
    }
}

/// Strip a generic wrapper: `Vec<i32>` with wrapper=`Vec` → `Some("i32")`.
fn strip_generic<'a>(t: &'a str, wrapper: &str) -> Option<&'a str> {
    let prefix = format!("{}<", wrapper);
    if t.starts_with(prefix.as_str()) && t.ends_with('>') {
        Some(&t[prefix.len()..t.len() - 1])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
#[derive(Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub download_count: i64,
    #[serde(default)]
    pub star_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated_message: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub enum Status {
    Active,
    Deprecated,
    Archived,
}
"#;

    #[test]
    fn parse_detects_struct_and_enum() {
        let items = parse_rust_items(SAMPLE);
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn generates_zod_schema() {
        let items = parse_rust_items(SAMPLE);
        let path = std::path::Path::new("test.rs");
        let ts = generate_ts_output(&items, path, false);
        assert!(ts.contains("PackageSchema"), "got: {}", ts);
        assert!(ts.contains("z.string()"), "got: {}", ts);
        assert!(ts.contains("z.number().int()"), "got: {}", ts);
        assert!(ts.contains("StatusSchema"), "got: {}", ts);
        assert!(ts.contains("z.enum("), "got: {}", ts);
    }

    #[test]
    fn optional_field_gets_optional() {
        let items = parse_rust_items(SAMPLE);
        let ts = generate_ts_output(&items, std::path::Path::new("x.rs"), false);
        assert!(ts.contains("z.string().optional()"), "got: {}", ts);
    }

    #[test]
    fn vec_maps_to_array() {
        assert_eq!(rust_type_to_zod("Vec<String>"), "z.array(z.string())");
    }

    #[test]
    fn option_unwraps() {
        assert_eq!(rust_type_to_zod("Option<bool>"), "z.boolean()");
    }

    #[test]
    fn hashmap_maps_to_record() {
        assert!(rust_type_to_zod("HashMap<String, i32>").contains("z.record("));
    }
}
