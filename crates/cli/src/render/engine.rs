use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::Path;

thread_local! {
    static IMPORT_COLLECTOR: RefCell<BTreeSet<String>> = RefCell::new(BTreeSet::new());
    static PRIORITY_IMPORTS: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

pub fn reset_import_collector() {
    IMPORT_COLLECTOR.with(|c| c.borrow_mut().clear());
    PRIORITY_IMPORTS.with(|c| c.borrow_mut().clear());
}

fn collect_import(value: String) -> String {
    IMPORT_COLLECTOR.with(|c| c.borrow_mut().insert(value));
    String::new()
}

fn collect_import_priority(value: String) -> String {
    PRIORITY_IMPORTS.with(|c| c.borrow_mut().push(value));
    String::new()
}

fn render_imports() -> String {
    let priority = PRIORITY_IMPORTS.with(|c| c.borrow().clone());
    let rest: Vec<_> = IMPORT_COLLECTOR.with(|c| c.borrow().iter().cloned().collect());
    let mut all = priority;
    for imp in rest {
        if !all.contains(&imp) {
            all.push(imp);
        }
    }
    all.join("\n")
}

pub fn build_engine(templates_dir: &Path) -> minijinja::Environment<'static> {
    build_engine_with_plugins(templates_dir, &[])
}

pub fn build_engine_with_plugins(
    templates_dir: &Path,
    plugin_dirs: &[std::path::PathBuf],
) -> minijinja::Environment<'static> {
    // Collect all template search dirs in priority order:
    // 1. plugin_dirs (user overrides)
    // 2. explicit templates_dir
    // 3. PackageStore template dirs (installed packages)
    // 4. Embedded fallback
    let store = crate::packages::PackageStore::default();
    let pkg_template_dirs = store.template_dirs();

    let mut all_dirs: Vec<std::path::PathBuf> = plugin_dirs.to_vec();
    if templates_dir.exists() {
        all_dirs.push(templates_dir.to_path_buf());
    }
    all_dirs.extend(pkg_template_dirs);

    build_engine_from_dirs(all_dirs)
}

/// Build a MiniJinja engine that searches `search_dirs` in order,
/// then falls back to embedded templates.
pub fn build_engine_from_dirs(
    search_dirs: Vec<std::path::PathBuf>,
) -> minijinja::Environment<'static> {
    let mut env = minijinja::Environment::new();

    let embedded_templates = crate::render::embedded::get_embedded_templates();

    env.set_loader(move |name| {
        use minijinja::Error;
        for dir in &search_dirs {
            let path = dir.join(name);
            if path.exists() {
                return std::fs::read_to_string(&path).map(Some).map_err(|e| {
                    Error::new(minijinja::ErrorKind::InvalidOperation, format!("{}", e))
                });
            }
        }
        if let Some(content) = embedded_templates.get(name) {
            Ok(Some(content.to_string()))
        } else {
            Ok(None)
        }
    });

    env.add_filter("snake_case", |v: &str| v.to_snake_case());
    env.add_filter("pascal_case", |v: &str| v.to_pascal_case());
    env.add_filter("camel_case", |v: &str| v.to_lower_camel_case());
    env.add_filter("kebab_case", |v: &str| v.to_kebab_case());

    env.add_filter("collect_import", |v: String| collect_import(v));
    env.add_filter("collect_import_priority", |v: String| {
        collect_import_priority(v)
    });

    env.add_function("render_imports", |_: &[minijinja::Value]| render_imports());

    env
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_key_with_params() {
        let mut env = build_engine(Path::new("templates"));
        env.add_template(
            "query_key",
            include_str!("../../../../templates/atoms/query/query_key.jinja"),
        )
        .unwrap();
        let template = env.get_template("query_key").unwrap();

        let result = template
            .render(minijinja::context!(
                name => "product",
                params => serde_json::json!(["id"])
            ))
            .unwrap();

        assert!(result.contains("productQueryKey"));
        assert!(result.contains("\"id\""));
    }

    #[test]
    fn test_pascal_case_filter() {
        let mut env = build_engine(Path::new("."));
        env.add_template("test", "{{ name | pascal_case }}")
            .unwrap();
        let template = env.get_template("test").unwrap();
        let result = template
            .render(minijinja::context!(name => "hello_world"))
            .unwrap();
        assert_eq!(result, "HelloWorld");
    }

    #[test]
    fn test_form_field_switch() {
        let mut env = build_engine(Path::new("templates"));
        env.add_template(
            "field_switch",
            include_str!("../../../../templates/atoms/form/field_switch.jinja"),
        )
        .unwrap();
        let template = env.get_template("field_switch").unwrap();

        let field = serde_json::json!({
            "name": "active",
            "type": "boolean",
            "required": false
        });

        let result = template
            .render(minijinja::context!(field => field, form => serde_json::json!({})))
            .unwrap();

        assert!(result.contains("id=\"active\""));
    }

    #[test]
    fn test_query_key() {
        let mut env = build_engine(Path::new("templates"));
        env.add_template(
            "query_key",
            include_str!("../../../../templates/atoms/query/query_key.jinja"),
        )
        .unwrap();
        let template = env.get_template("query_key").unwrap();

        let result = template
            .render(minijinja::context!(
                name => "products",
                params => serde_json::json!([])
            ))
            .unwrap();

        assert!(result.contains("productsQueryKey"));
        assert!(result.contains("['products']"));
    }

    #[test]
    fn test_kebab_case_filter() {
        let mut env = build_engine(Path::new("."));
        env.add_template("test", "{{ name | kebab_case }}").unwrap();
        let template = env.get_template("test").unwrap();
        let result = template
            .render(minijinja::context!(name => "helloWorld"))
            .unwrap();
        assert_eq!(result, "hello-world");
    }

    #[test]
    fn test_drizzle_column_atom_string() {
        reset_import_collector();

        let mut env = build_engine(Path::new("templates"));
        env.add_template(
            "column",
            include_str!("../../../../templates/atoms/drizzle/column.jinja"),
        )
        .unwrap();
        let template = env.get_template("column").unwrap();

        let field = serde_json::json!({
            "name": "title",
            "type": "string",
            "required": true
        });

        let result = template
            .render(minijinja::context!(field => field))
            .unwrap();

        assert!(result.contains("title: text('title').notNull()"));

        reset_import_collector();
    }

    #[test]
    fn test_import_collector_priority() {
        reset_import_collector();

        let mut env = build_engine(Path::new("."));
        env.add_template("test", "{{ 'import foo from \"foo\"' | collect_import }}\n{{ 'import react from \"react\"' | collect_import_priority }}\n{{ render_imports() }}").unwrap();
        let template = env.get_template("test").unwrap();
        let result = template.render(minijinja::context!()).unwrap();

        let lines: Vec<&str> = result.lines().filter(|l| !l.is_empty()).collect();
        assert!(!lines.is_empty());
        assert!(lines[0].contains("react") || lines[1].contains("react"));

        reset_import_collector();
    }

    #[test]
    fn test_drizzle_column_atom_with_reference() {
        reset_import_collector();

        let mut env = build_engine(Path::new("templates"));
        env.add_template(
            "column",
            include_str!("../../../../templates/atoms/drizzle/column.jinja"),
        )
        .unwrap();
        let template = env.get_template("column").unwrap();

        let field = serde_json::json!({
            "name": "categoryId",
            "type": "id",
            "references": "categories"
        });

        let result = template
            .render(minijinja::context!(field => field))
            .unwrap();

        assert!(result.contains("categoryId"));
        assert!(result.contains("references(() => categories.id"));

        reset_import_collector();
    }

    #[test]
    fn test_drizzle_timestamp_cols() {
        reset_import_collector();

        let mut env = build_engine(Path::new("templates"));
        env.add_template(
            "timestamp_cols",
            include_str!("../../../../templates/atoms/drizzle/timestamp_cols.jinja"),
        )
        .unwrap();
        let template = env.get_template("timestamp_cols").unwrap();

        let result = template.render(minijinja::context!()).unwrap();

        assert!(result.contains("createdAt"));
        assert!(result.contains("updatedAt"));

        reset_import_collector();
    }

    #[test]
    fn test_drizzle_soft_delete_col() {
        reset_import_collector();

        let mut env = build_engine(Path::new("templates"));
        env.add_template(
            "soft_delete",
            include_str!("../../../../templates/atoms/drizzle/soft_delete_col.jinja"),
        )
        .unwrap();
        let template = env.get_template("soft_delete").unwrap();

        let result = template.render(minijinja::context!()).unwrap();

        assert!(result.contains("deletedAt"));

        reset_import_collector();
    }

    #[test]
    fn test_drizzle_table_body() {
        reset_import_collector();

        let mut env = build_engine(Path::new("templates"));
        env.add_template(
            "table_body",
            include_str!("../../../../templates/molecules/drizzle/table_body.jinja"),
        )
        .unwrap();
        env.add_template(
            "atoms/drizzle/column",
            include_str!("../../../../templates/atoms/drizzle/column.jinja"),
        )
        .unwrap();
        env.add_template(
            "atoms/drizzle/timestamp_cols",
            include_str!("../../../../templates/atoms/drizzle/timestamp_cols.jinja"),
        )
        .unwrap();
        let template = env.get_template("table_body").unwrap();

        let fields = serde_json::json!([
            {"name": "title", "type": "string", "required": true},
            {"name": "price", "type": "number", "required": true}
        ]);

        let result = template
            .render(minijinja::context!(
                name => "products",
                fields => fields,
                timestamps => true,
                soft_delete => false
            ))
            .unwrap();

        assert!(result.contains("sqliteTable"));
        assert!(result.contains("export const products"));
        assert!(result.contains("export type Product"));

        reset_import_collector();
    }

    #[test]
    fn test_zod_field_rule_string() {
        reset_import_collector();

        let mut env = build_engine(Path::new("templates"));
        env.add_template(
            "field_rule",
            include_str!("../../../../templates/atoms/zod/field_rule.jinja"),
        )
        .unwrap();
        let template = env.get_template("field_rule").unwrap();

        let field = serde_json::json!({
            "name": "title",
            "type": "string",
            "required": true
        });

        let result = template
            .render(minijinja::context!(field => field))
            .unwrap();

        assert!(result.contains("title: z.string()"));

        reset_import_collector();
    }

    #[test]
    fn test_zod_field_rule_email() {
        reset_import_collector();

        let mut env = build_engine(Path::new("templates"));
        env.add_template(
            "field_rule",
            include_str!("../../../../templates/atoms/zod/field_rule.jinja"),
        )
        .unwrap();
        let template = env.get_template("field_rule").unwrap();

        let field = serde_json::json!({
            "name": "email",
            "type": "email",
            "required": false
        });

        let result = template
            .render(minijinja::context!(field => field))
            .unwrap();

        assert!(result.contains("email: z.string().email().optional()"));

        reset_import_collector();
    }

    #[test]
    fn test_form_field_input() {
        let mut env = build_engine(Path::new("templates"));
        env.add_template(
            "field_input",
            include_str!("../../../../templates/atoms/form/field_input.jinja"),
        )
        .unwrap();
        let template = env.get_template("field_input").unwrap();

        let field = serde_json::json!({
            "name": "title",
            "type": "string",
            "required": true
        });

        let result = template
            .render(minijinja::context!(field => field, form => serde_json::json!({})))
            .unwrap();

        assert!(result.contains("id=\"title\""));
        assert!(result.contains("name=\"title\""));
    }
}
