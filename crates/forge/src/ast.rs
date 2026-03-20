//! **ForgeNode AST builder** — build TypeScript output as a typed node tree in Rust (E2).
//!
//! Instead of writing template strings, callers compose a document from typed nodes
//! that render to formatted TypeScript/TSX source code. This eliminates template
//! syntax errors, makes output refactorable, and is fully testable at the type level.
//!
//! ## Quick start
//!
//! ```rust
//! use forge::ast::*;
//!
//! let doc = ForgeFile::new("schema.ts")
//!     .import("drizzle-orm/pg-core").named(["pgTable", "text", "uuid", "timestamp"])
//!     .body(
//!         pg_table("users_table")
//!             .col(uuid_pk("id"))
//!             .col(text_col("email").unique().not_null())
//!             .timestamps()
//!     )
//!     .export(["usersTable"]);
//!
//! let output = doc.render(&StyleConfig::default()).unwrap();
//! assert!(output.contains("pgTable"));
//! ```

use std::fmt::Write as FmtWrite;

// ---------------------------------------------------------------------------
// Style configuration
// ---------------------------------------------------------------------------

/// Controls how the AST is serialised to source text.
#[derive(Debug, Clone)]
pub struct StyleConfig {
    /// Number of spaces per indent level (default: 2)
    pub indent: usize,
    /// Quote style for string literals (default: double)
    pub quotes: QuoteStyle,
    /// Whether to emit semicolons (default: true)
    pub semicolons: bool,
    /// Whether to emit trailing commas in multi-line structures (default: true)
    pub trailing_commas: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuoteStyle { Single, Double }

impl Default for StyleConfig {
    fn default() -> Self {
        Self { indent: 2, quotes: QuoteStyle::Double, semicolons: true, trailing_commas: true }
    }
}

impl StyleConfig {
    pub fn quote(&self, s: &str) -> String {
        match self.quotes {
            QuoteStyle::Double => format!("\"{}\"", s),
            QuoteStyle::Single => format!("'{}'", s),
        }
    }
    pub fn semi(&self) -> &str {
        if self.semicolons { ";" } else { "" }
    }
    pub fn indent_str(&self) -> String {
        " ".repeat(self.indent)
    }
}

// ---------------------------------------------------------------------------
// Render trait
// ---------------------------------------------------------------------------

pub trait Render {
    /// Render this node at the given indent level.
    fn render_indented(&self, style: &StyleConfig, depth: usize) -> String;

    /// Render at top level (depth 0).
    fn render(&self, style: &StyleConfig) -> Result<String, String> {
        Ok(self.render_indented(style, 0))
    }
}

// ---------------------------------------------------------------------------
// Import statement
// ---------------------------------------------------------------------------

/// `import { A, B } from "module"` or `import Mod from "module"`
#[derive(Debug, Clone)]
pub struct ImportNode {
    pub module: String,
    pub named: Vec<String>,
    pub default: Option<String>,
    pub side_effect: bool, // import "module"
}

impl ImportNode {
    pub fn new(module: impl Into<String>) -> Self {
        Self { module: module.into(), named: Vec::new(), default: None, side_effect: false }
    }

    /// Add named imports, returning `self` for chaining.
    pub fn named<I, S>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.named.extend(items.into_iter().map(|s| s.into()));
        self
    }

    /// Set the default import.
    pub fn default_import(mut self, name: impl Into<String>) -> Self {
        self.default = Some(name.into());
        self
    }
}

impl Render for ImportNode {
    fn render_indented(&self, style: &StyleConfig, _depth: usize) -> String {
        if self.side_effect {
            return format!("import {}{}", style.quote(&self.module), style.semi());
        }
        let module_str = style.quote(&self.module);
        match (&self.default, self.named.is_empty()) {
            (Some(def), true) => format!("import {} from {}{}", def, module_str, style.semi()),
            (None, false) => {
                let names = self.named.join(", ");
                format!("import {{ {} }} from {}{}", names, module_str, style.semi())
            }
            (Some(def), false) => {
                let names = self.named.join(", ");
                format!("import {}, {{ {} }} from {}{}", def, names, module_str, style.semi())
            }
            (None, true) => format!("import {} from {}{}", style.quote(&self.module), module_str, style.semi()),
        }
    }
}

// ---------------------------------------------------------------------------
// Import builder — fluent API attached to ForgeFile
// ---------------------------------------------------------------------------

/// Builder returned by `ForgeFile::import()` that allows chaining `.named()`.
pub struct ImportBuilder<'a> {
    file: &'a mut ForgeFile,
    module: String,
}

impl<'a> ImportBuilder<'a> {
    /// Add named imports and return the file for further chaining.
    pub fn named<I, S>(self, items: I) -> &'a mut ForgeFile
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let node = ImportNode::new(self.module).named(items);
        self.file.imports.push(node);
        self.file
    }

    /// Set default import.
    pub fn default_import(self, name: impl Into<String>) -> &'a mut ForgeFile {
        let node = ImportNode::new(self.module).default_import(name);
        self.file.imports.push(node);
        self.file
    }
}

// ---------------------------------------------------------------------------
// Column / field nodes
// ---------------------------------------------------------------------------

/// A single column in a Drizzle table definition.
#[derive(Debug, Clone)]
pub struct ColumnNode {
    pub field_name: String,
    pub col_type: String,
    pub db_name: Option<String>,
    pub not_null: bool,
    pub unique: bool,
    pub primary_key: bool,
    pub default_random: bool,
    pub default_now: bool,
    pub default_val: Option<String>,
    pub nullable_explicit: bool,
}

impl ColumnNode {
    pub fn new(field_name: impl Into<String>, col_type: impl Into<String>) -> Self {
        Self {
            field_name: field_name.into(),
            col_type: col_type.into(),
            db_name: None,
            not_null: false,
            unique: false,
            primary_key: false,
            default_random: false,
            default_now: false,
            default_val: None,
            nullable_explicit: false,
        }
    }
    pub fn db_name(mut self, name: impl Into<String>) -> Self { self.db_name = Some(name.into()); self }
    pub fn not_null(mut self) -> Self { self.not_null = true; self }
    pub fn unique(mut self) -> Self { self.unique = true; self }
    pub fn primary_key(mut self) -> Self { self.primary_key = true; self }
    pub fn default_random(mut self) -> Self { self.default_random = true; self }
    pub fn default_now(mut self) -> Self { self.default_now = true; self }
    pub fn default_val(mut self, val: impl Into<String>) -> Self { self.default_val = Some(val.into()); self }
    pub fn nullable(mut self) -> Self { self.nullable_explicit = true; self }
}

impl Render for ColumnNode {
    fn render_indented(&self, style: &StyleConfig, depth: usize) -> String {
        let pad = style.indent_str().repeat(depth);
        let db = self.db_name.as_deref().unwrap_or(&self.field_name);
        let mut s = format!("{}{}: {}({})", pad, self.field_name, self.col_type, style.quote(db));
        if self.primary_key { s.push_str(".primaryKey()"); }
        if self.default_random { s.push_str(".defaultRandom()"); }
        if self.default_now { s.push_str(".defaultNow()"); }
        if let Some(val) = &self.default_val { s.push_str(&format!(".default({})", val)); }
        if self.not_null { s.push_str(".notNull()"); }
        if self.unique { s.push_str(".unique()"); }
        if self.nullable_explicit { s.push_str(".nullable()"); }
        s.push(',');
        s
    }
}

// ---------------------------------------------------------------------------
// Table node (Drizzle pgTable / sqliteTable / mysqlTable)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableKind { Pg, Sqlite, Mysql }

/// A complete Drizzle table definition.
#[derive(Debug, Clone)]
pub struct TableNode {
    pub var_name: String,
    pub table_name: String,
    pub kind: TableKind,
    pub columns: Vec<ColumnNode>,
    pub has_timestamps: bool,
    pub soft_delete: bool,
}

impl TableNode {
    pub fn new(var_name: impl Into<String>, table_name: impl Into<String>, kind: TableKind) -> Self {
        Self {
            var_name: var_name.into(),
            table_name: table_name.into(),
            kind,
            columns: Vec::new(),
            has_timestamps: false,
            soft_delete: false,
        }
    }

    pub fn col(mut self, col: ColumnNode) -> Self {
        self.columns.push(col);
        self
    }

    pub fn timestamps(mut self) -> Self {
        self.has_timestamps = true;
        self
    }

    pub fn soft_delete(mut self) -> Self {
        self.soft_delete = true;
        self
    }
}

impl Render for TableNode {
    fn render_indented(&self, style: &StyleConfig, _depth: usize) -> String {
        let fn_name = match self.kind {
            TableKind::Pg => "pgTable",
            TableKind::Sqlite => "sqliteTable",
            TableKind::Mysql => "mysqlTable",
        };
        let tc = if style.trailing_commas { "," } else { "" };
        let pad = style.indent_str();

        let mut s = format!(
            "export const {} = {}({}, {{\n",
            self.var_name,
            fn_name,
            style.quote(&self.table_name)
        );

        for col in &self.columns {
            s.push_str(&col.render_indented(style, 1));
            s.push('\n');
        }

        if self.has_timestamps {
            s.push_str(&format!(
                "{}createdAt: timestamp(\"created_at\").defaultNow().notNull(){},\n",
                pad, tc
            ));
            s.push_str(&format!(
                "{}updatedAt: timestamp(\"updated_at\").defaultNow().notNull(){},\n",
                pad, tc
            ));
        }

        if self.soft_delete {
            s.push_str(&format!(
                "{}deletedAt: timestamp(\"deleted_at\").nullable(){},\n",
                pad, tc
            ));
        }

        s.push_str("})");
        s.push_str(style.semi());
        s
    }
}

// ---------------------------------------------------------------------------
// Generic expression / raw node
// ---------------------------------------------------------------------------

/// A raw string that is emitted verbatim.
#[derive(Debug, Clone)]
pub struct RawNode(pub String);

impl Render for RawNode {
    fn render_indented(&self, _style: &StyleConfig, depth: usize) -> String {
        let pad = " ".repeat(depth * 2);
        self.0.lines().map(|l| format!("{}{}", pad, l)).collect::<Vec<_>>().join("\n")
    }
}

// ---------------------------------------------------------------------------
// Top-level body node enum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum BodyNode {
    Table(TableNode),
    Raw(RawNode),
}

impl Render for BodyNode {
    fn render_indented(&self, style: &StyleConfig, depth: usize) -> String {
        match self {
            BodyNode::Table(t) => t.render_indented(style, depth),
            BodyNode::Raw(r) => r.render_indented(style, depth),
        }
    }
}

// ---------------------------------------------------------------------------
// ForgeFile — the root document node
// ---------------------------------------------------------------------------

/// A complete TypeScript/TSX source file built from typed nodes.
pub struct ForgeFile {
    pub filename: String,
    pub imports: Vec<ImportNode>,
    pub body: Vec<BodyNode>,
    pub exports: Vec<String>,
}

impl ForgeFile {
    pub fn new(filename: impl Into<String>) -> Self {
        Self { filename: filename.into(), imports: Vec::new(), body: Vec::new(), exports: Vec::new() }
    }

    /// Start an import builder for `module`.
    pub fn import(&mut self, module: impl Into<String>) -> ImportBuilder<'_> {
        ImportBuilder { file: self, module: module.into() }
    }

    /// Add a body node.
    pub fn body(mut self, node: impl Into<BodyNode>) -> Self {
        self.body.push(node.into());
        self
    }

    /// Declare named re-exports (emitted at the end of the file).
    pub fn export<I, S>(mut self, names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.exports.extend(names.into_iter().map(|s| s.into()));
        self
    }

    /// Render to a string.
    pub fn render_to_string(&self, style: &StyleConfig) -> Result<String, String> {
        let mut out = String::new();

        // Imports
        for imp in &self.imports {
            writeln!(out, "{}", imp.render_indented(style, 0)).map_err(|e| e.to_string())?;
        }
        if !self.imports.is_empty() && !self.body.is_empty() {
            out.push('\n');
        }

        // Body
        for (i, node) in self.body.iter().enumerate() {
            write!(out, "{}", node.render_indented(style, 0)).map_err(|e| e.to_string())?;
            if i < self.body.len() - 1 { out.push_str("\n\n"); }
        }
        if !self.body.is_empty() { out.push('\n'); }

        // Re-exports
        if !self.exports.is_empty() {
            out.push('\n');
            let names = self.exports.join(", ");
            writeln!(out, "export {{ {} }}{}", names, style.semi()).map_err(|e| e.to_string())?;
        }

        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// Conversion impls
// ---------------------------------------------------------------------------

impl From<TableNode> for BodyNode {
    fn from(t: TableNode) -> Self { BodyNode::Table(t) }
}

impl From<RawNode> for BodyNode {
    fn from(r: RawNode) -> Self { BodyNode::Raw(r) }
}

// ---------------------------------------------------------------------------
// Convenience constructor functions (mirror the spec examples)
// ---------------------------------------------------------------------------

/// `pgTable("name", { ... })` — creates a PostgreSQL table builder.
pub fn pg_table(var_name: impl Into<String>) -> TableNode {
    let vn: String = var_name.into();
    let tbl_name = to_snake_case(&vn);
    TableNode::new(vn, tbl_name, TableKind::Pg)
}

/// `sqliteTable("name", { ... })`
pub fn sqlite_table(var_name: impl Into<String>) -> TableNode {
    let vn: String = var_name.into();
    let tbl_name = to_snake_case(&vn);
    TableNode::new(vn, tbl_name, TableKind::Sqlite)
}

/// UUID primary key with `.defaultRandom()`.
pub fn uuid_pk(field_name: impl Into<String>) -> ColumnNode {
    ColumnNode::new(field_name, "uuid").primary_key().default_random()
}

/// `text("name")` column.
pub fn text_col(field_name: impl Into<String>) -> ColumnNode {
    ColumnNode::new(field_name, "text")
}

/// `integer("name")` column.
pub fn int_col(field_name: impl Into<String>) -> ColumnNode {
    ColumnNode::new(field_name, "integer")
}

/// `boolean("name")` column.
pub fn bool_col(field_name: impl Into<String>) -> ColumnNode {
    ColumnNode::new(field_name, "boolean")
}

/// `timestamp("name")` column.
pub fn timestamp_col(field_name: impl Into<String>) -> ColumnNode {
    ColumnNode::new(field_name, "timestamp")
}

/// `real("name")` column.
pub fn real_col(field_name: impl Into<String>) -> ColumnNode {
    ColumnNode::new(field_name, "real")
}

/// A raw string body node.
pub fn raw(s: impl Into<String>) -> RawNode {
    RawNode(s.into())
}

// ---------------------------------------------------------------------------
// String utilities
// ---------------------------------------------------------------------------

pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap_or(c));
    }
    result
}

pub fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-' || c == ' ')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_named() {
        let style = StyleConfig::default();
        let imp = ImportNode::new("drizzle-orm/pg-core").named(["pgTable", "text"]);
        let rendered = imp.render_indented(&style, 0);
        assert_eq!(rendered, r#"import { pgTable, text } from "drizzle-orm/pg-core";"#);
    }

    #[test]
    fn import_default() {
        let style = StyleConfig::default();
        let imp = ImportNode::new("react").default_import("React");
        assert_eq!(imp.render_indented(&style, 0), r#"import React from "react";"#);
    }

    #[test]
    fn column_uuid_pk() {
        let style = StyleConfig::default();
        let col = uuid_pk("id");
        let out = col.render_indented(&style, 1);
        assert!(out.contains("primaryKey()"));
        assert!(out.contains("defaultRandom()"));
        assert!(out.contains(r#"uuid("id")"#));
    }

    #[test]
    fn table_renders_columns_and_timestamps() {
        let style = StyleConfig::default();
        let table = pg_table("usersTable")
            .col(uuid_pk("id"))
            .col(text_col("email").not_null().unique())
            .timestamps();
        let out = table.render_indented(&style, 0);
        assert!(out.contains("pgTable"));
        assert!(out.contains("usersTable"));
        assert!(out.contains("createdAt"));
        assert!(out.contains("updatedAt"));
        assert!(out.contains("email"));
    }

    #[test]
    fn forge_file_full() {
        let style = StyleConfig::default();
        let mut file = ForgeFile::new("schema.ts");
        file.import("drizzle-orm/pg-core").named(["pgTable", "text", "uuid", "timestamp"]);
        let file = file
            .body(
                pg_table("usersTable")
                    .col(uuid_pk("id"))
                    .col(text_col("email").not_null())
                    .timestamps()
            )
            .export(["usersTable"]);

        let out = file.render_to_string(&style).unwrap();
        assert!(out.contains("import { pgTable"));
        assert!(out.contains("export const usersTable"));
        assert!(out.contains("export { usersTable }"));
    }

    #[test]
    fn to_snake_case_converts_camel() {
        assert_eq!(to_snake_case("usersTable"), "users_table");
        assert_eq!(to_snake_case("myField"), "my_field");
    }

    #[test]
    fn to_pascal_case_converts_snake() {
        assert_eq!(to_pascal_case("users_table"), "UsersTable");
        assert_eq!(to_pascal_case("my-field"), "MyField");
    }
}
