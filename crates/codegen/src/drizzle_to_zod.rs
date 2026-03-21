//! Drizzle schema file → Zod insert/select schemas.
//!
//! Parses a TypeScript Drizzle schema file and emits:
//!   export const insertUserSchema = createInsertSchema(usersTable)
//!   export const selectUserSchema = createSelectSchema(usersTable)
//!   export type InsertUser = z.infer<typeof insertUserSchema>
//!   export type SelectUser = z.infer<typeof selectUserSchema>

use crate::types::{CodegenError, CodegenInput, CodegenOutput};

pub fn convert(input: CodegenInput, out_filename: Option<&str>) -> Result<CodegenOutput, CodegenError> {
    let src = input.read()?;
    let tables = extract_table_names(&src);

    if tables.is_empty() {
        return Err(CodegenError::ParseError(
            "No `pgTable`/`sqliteTable`/`mysqlTable` declarations found".into(),
        ));
    }

    let mut lines = vec![
        "// AUTO-GENERATED — Drizzle table → Zod schemas".to_string(),
        r#"import { z } from "zod""#.to_string(),
        r#"import { createInsertSchema, createSelectSchema } from "drizzle-zod""#.to_string(),
        String::new(),
        "// Import your table definitions".to_string(),
        format!(
            "import {{ {} }} from \"./schema\"",
            tables.iter().map(|t| t.var_name.as_str()).collect::<Vec<_>>().join(", ")
        ),
        String::new(),
    ];

    let mut exported_names = Vec::new();

    for table in &tables {
        let pascal = to_pascal(&table.table_name);

        let insert_name = format!("insert{}Schema", pascal);
        let select_name = format!("select{}Schema", pascal);
        let insert_type = format!("Insert{}", pascal);
        let select_type = format!("Select{}", pascal);

        lines.push(format!("export const {} = createInsertSchema({})", insert_name, table.var_name));
        lines.push(format!("export const {} = createSelectSchema({})", select_name, table.var_name));
        lines.push(String::new());
        lines.push(format!("export type {} = z.infer<typeof {}>", insert_type, insert_name));
        lines.push(format!("export type {} = z.infer<typeof {}>", select_type, select_name));
        lines.push(String::new());

        exported_names.extend([insert_name, select_name, insert_type, select_type]);
    }

    Ok(CodegenOutput {
        content: lines.join("\n"),
        filename: out_filename.unwrap_or("generated/drizzle-schemas.ts").to_string(),
        exported_names,
        warnings: Vec::new(),
    })
}

// ---------------------------------------------------------------------------
// Table name extraction
// ---------------------------------------------------------------------------

struct TableRef {
    var_name: String,
    table_name: String,
}

fn extract_table_names(src: &str) -> Vec<TableRef> {
    let mut tables = Vec::new();
    for line in src.lines() {
        let trimmed = line.trim();
        // Matches: `export const <varName> = pgTable("<tableName>",`
        for fn_name in &["pgTable", "sqliteTable", "mysqlTable"] {
            if let Some(pos) = trimmed.find(fn_name) {
                // Extract var name: const <varName> =
                if let Some(var_name) = extract_const_name(trimmed) {
                    // Extract table string name from the function call
                    let after_fn = &trimmed[pos + fn_name.len()..];
                    if let Some(tbl_name) = extract_string_arg(after_fn) {
                        tables.push(TableRef { var_name, table_name: tbl_name });
                    }
                }
            }
        }
    }
    tables
}

fn extract_const_name(line: &str) -> Option<String> {
    // "export const usersTable = " or "const usersTable ="
    let after_const = line.split("const ").nth(1)?;
    let name: String = after_const
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() { None } else { Some(name) }
}

fn extract_string_arg(s: &str) -> Option<String> {
    // Find first "..." or '...' after the opening paren
    let s = s.trim_start_matches('(').trim();
    let q = s.chars().find(|c| *c == '"' || *c == '\'')?;
    let inner = s.trim_start_matches(q);
    let end = inner.find(q)?;
    Some(inner[..end].to_string())
}

fn to_pascal(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pg_tables() {
        let src = r#"
export const usersTable = pgTable("users", { id: uuid("id").primaryKey() })
export const postsTable = pgTable("posts", { id: uuid("id").primaryKey() })
"#;
        let out = convert(CodegenInput::source(src), None).unwrap();
        assert!(out.content.contains("insertUsersSchema"));
        assert!(out.content.contains("selectPostsSchema"));
        assert!(out.content.contains("createInsertSchema(usersTable)"));
    }
}
