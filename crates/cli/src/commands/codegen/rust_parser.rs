//! Lightweight, regex-free line-by-line parser for the common serde struct/enum
//! patterns `rust_to_ts` supports. Does NOT attempt to handle all Rust syntax.

use super::types::{RustEnum, RustField, RustItem, RustStruct, RustVariant};

pub(super) fn parse_rust_items(src: &str) -> Vec<RustItem> {
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
