//! `tsx codegen` — code generation utilities.
//!
//! Currently supports:
//! - `rust-to-ts`: parse Rust struct/enum definitions and emit TypeScript interfaces + Zod schemas
//! - `openapi-to-zod`: convert an OpenAPI spec to Zod schemas (stub — emits instructions)
//! - `drizzle-to-zod`: run drizzle-zod across schema files (stub — emits instructions)

use std::path::PathBuf;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

// ---------------------------------------------------------------------------
// Public entrypoints
// ---------------------------------------------------------------------------

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

pub fn codegen_openapi_to_zod(spec: String, out: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let out_path = out.unwrap_or_else(|| "src/lib/api-schemas.ts".to_string());
    ResponseEnvelope::success(
        "codegen openapi-to-zod",
        serde_json::json!({
            "spec": spec,
            "output": out_path,
            "status": "To generate Zod schemas from your OpenAPI spec, run: npx openapi-zod-client@latest <spec> -o <output>",
            "recommended_tool": "openapi-zod-client",
            "install": "npm install -D openapi-zod-client",
        }),
        0,
    )
    .with_next_steps(vec![
        format!("npx openapi-zod-client {} -o {}", spec, out_path),
        "Add the generated file to your version control".to_string(),
    ])
}

pub fn codegen_drizzle_to_zod(_verbose: bool) -> ResponseEnvelope {
    ResponseEnvelope::success(
        "codegen drizzle-to-zod",
        serde_json::json!({
            "status": "drizzle-zod integration",
            "install": "npm install drizzle-zod",
            "usage": "import { createInsertSchema, createSelectSchema } from 'drizzle-zod'",
            "example": "export const insertUserSchema = createInsertSchema(usersTable)\nexport const selectUserSchema = createSelectSchema(usersTable)",
        }),
        0,
    )
    .with_next_steps(vec![
        "npm install drizzle-zod".to_string(),
        "Import createInsertSchema / createSelectSchema from 'drizzle-zod' in your schema files".to_string(),
    ])
}

// ---------------------------------------------------------------------------
// Rust source parser — lightweight, regex-free hand-written parser
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum RustItem {
    Struct(RustStruct),
    Enum(RustEnum),
}

#[derive(Debug, Clone)]
struct RustStruct {
    name: String,
    fields: Vec<RustField>,
}

#[derive(Debug, Clone)]
struct RustField {
    name: String,
    rust_type: String,
    serde_rename: Option<String>,
    is_optional: bool, // Option<T> or #[serde(skip_serializing_if = "Option::is_none")]
    has_default: bool, // #[serde(default)]
    is_flatten: bool,  // #[serde(flatten)]
}

#[derive(Debug, Clone)]
struct RustEnum {
    name: String,
    variants: Vec<RustVariant>,
    is_unit_only: bool,
}

#[derive(Debug, Clone)]
struct RustVariant {
    name: String,
    serde_rename: Option<String>,
    payload: Option<String>, // for tuple/struct variants
}

/// Very lightweight line-by-line parser that handles the common cases from the spec.
/// Does NOT attempt to handle all Rust syntax — only the patterns produced by typical
/// serde structs and simple enums.
fn parse_rust_items(src: &str) -> Vec<RustItem> {
    let mut items: Vec<RustItem> = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Only parse types with #[derive(Serialize, ...)] or #[derive(Deserialize, ...)]
        if line.starts_with("#[derive(") && (line.contains("Serialize") || line.contains("Deserialize")) {
            // Look ahead for the type definition
            let mut j = i + 1;
            // Skip more attribute lines
            while j < lines.len() && lines[j].trim().starts_with('#') {
                j += 1;
            }
            if j < lines.len() {
                let def_line = lines[j].trim();
                if def_line.starts_with("pub struct ") || def_line.starts_with("struct ") {
                    if let Some(item) = parse_struct(&lines, j) {
                        items.push(RustItem::Struct(item));
                    }
                } else if def_line.starts_with("pub enum ") || def_line.starts_with("enum ") {
                    if let Some(item) = parse_enum(&lines, j) {
                        items.push(RustItem::Enum(item));
                    }
                }
            }
        }
        i += 1;
    }

    items
}

fn parse_struct(lines: &[&str], start: usize) -> Option<RustStruct> {
    let def_line = lines[start].trim();
    let name = extract_type_name(def_line, "struct")?;

    // Find the opening brace
    let mut i = start;
    while i < lines.len() && !lines[i].contains('{') {
        i += 1;
    }
    i += 1; // move past opening brace

    let mut fields: Vec<RustField> = Vec::new();
    let mut serde_attrs: Vec<String> = Vec::new();

    while i < lines.len() {
        let line = lines[i].trim();

        if line == "}" || line == "}," {
            break;
        }

        if line.starts_with("#[serde(") {
            serde_attrs.push(line.to_string());
            i += 1;
            continue;
        }

        // Skip other attributes / doc comments
        if line.starts_with('#') || line.starts_with("//") {
            i += 1;
            continue;
        }

        // Parse field: `pub name: Type,` or `name: Type,`
        if let Some(field) = parse_field(line, &serde_attrs) {
            fields.push(field);
        }
        serde_attrs.clear();
        i += 1;
    }

    Some(RustStruct { name, fields })
}

fn parse_enum(lines: &[&str], start: usize) -> Option<RustEnum> {
    let def_line = lines[start].trim();
    let name = extract_type_name(def_line, "enum")?;

    // Find opening brace
    let mut i = start;
    while i < lines.len() && !lines[i].contains('{') {
        i += 1;
    }
    i += 1;

    let mut variants: Vec<RustVariant> = Vec::new();
    let mut serde_attrs: Vec<String> = Vec::new();
    let mut is_unit_only = true;

    while i < lines.len() {
        let line = lines[i].trim();

        if line == "}" || line == "}," {
            break;
        }

        if line.starts_with("#[serde(") {
            serde_attrs.push(line.to_string());
            i += 1;
            continue;
        }

        if line.starts_with('#') || line.starts_with("//") || line.is_empty() {
            i += 1;
            continue;
        }

        // Variant lines: `Name,` or `Name(Type),` or `Name { field: Type },`
        let variant_name = line
            .trim_end_matches(',')
            .split(|c| c == '(' || c == '{' || c == ' ')
            .next()
            .unwrap_or("")
            .to_string();

        if variant_name.is_empty() {
            i += 1;
            continue;
        }

        let has_payload = line.contains('(') || line.contains('{');
        if has_payload {
            is_unit_only = false;
        }

        let serde_rename = extract_serde_rename(&serde_attrs);
        variants.push(RustVariant {
            name: variant_name,
            serde_rename,
            payload: if has_payload {
                Some(line.to_string())
            } else {
                None
            },
        });
        serde_attrs.clear();
        i += 1;
    }

    Some(RustEnum { name, variants, is_unit_only })
}

fn parse_field(line: &str, serde_attrs: &[String]) -> Option<RustField> {
    // Strip `pub ` prefix
    let line = line.strip_prefix("pub ").unwrap_or(line);
    // Must contain `:` to be a field
    let colon_pos = line.find(':')?;
    let name_raw = line[..colon_pos].trim().to_string();
    if name_raw.is_empty() || name_raw.starts_with("//") {
        return None;
    }

    let type_raw = line[colon_pos + 1..]
        .trim()
        .trim_end_matches(',')
        .to_string();

    let is_optional_type = type_raw.starts_with("Option<");
    let has_default = serde_attrs.iter().any(|a| a.contains("default"));
    let is_flatten = serde_attrs.iter().any(|a| a.contains("flatten"));
    let skip_if_none = serde_attrs
        .iter()
        .any(|a| a.contains("skip_serializing_if"));
    let serde_rename = extract_serde_rename(serde_attrs);

    Some(RustField {
        name: name_raw,
        rust_type: type_raw,
        serde_rename,
        is_optional: is_optional_type || skip_if_none,
        has_default,
        is_flatten,
    })
}

fn extract_type_name(def_line: &str, keyword: &str) -> Option<String> {
    let keyword_with_space = format!("{} ", keyword);
    let after = def_line
        .strip_prefix("pub ")
        .unwrap_or(def_line)
        .strip_prefix(keyword_with_space.as_str())?;
    let name = after
        .split(|c: char| c.is_whitespace() || c == '<' || c == '{')
        .next()?
        .to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn extract_serde_rename(attrs: &[String]) -> Option<String> {
    for attr in attrs {
        if let Some(start) = attr.find("rename = \"") {
            let rest = &attr[start + 10..];
            if let Some(end) = rest.find('"') {
                return Some(rest[..end].to_string());
            }
        }
    }
    None
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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
