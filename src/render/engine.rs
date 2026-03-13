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
    let mut env = minijinja::Environment::new();

    if templates_dir.exists() {
        let templates_dir = templates_dir.to_path_buf();
        env.set_loader(move |name| {
            use minijinja::Error;
            let path = templates_dir.join(name);
            if path.exists() {
                std::fs::read_to_string(&path).map(Some).map_err(|e| {
                    Error::new(minijinja::ErrorKind::InvalidOperation, format!("{}", e))
                })
            } else {
                Ok(None)
            }
        });
    }

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
    fn test_snake_case_filter() {
        let mut env = build_engine(Path::new("."));
        env.add_template("test", "{{ name | snake_case }}").unwrap();
        let template = env.get_template("test").unwrap();
        let result = template
            .render(minijinja::context!(name => "HelloWorld"))
            .unwrap();
        assert_eq!(result, "hello_world");
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
    fn test_camel_case_filter() {
        let mut env = build_engine(Path::new("."));
        env.add_template("test", "{{ name | camel_case }}").unwrap();
        let template = env.get_template("test").unwrap();
        let result = template
            .render(minijinja::context!(name => "hello_world"))
            .unwrap();
        assert_eq!(result, "helloWorld");
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
    fn test_import_collector() {
        reset_import_collector();

        let mut env = build_engine(Path::new("."));
        env.add_template("test", "{{ 'import foo from \"foo\"' | collect_import }}\n{{ 'import bar from \"bar\"' | collect_import }}\n{{ render_imports() }}").unwrap();
        let template = env.get_template("test").unwrap();
        let result = template.render(minijinja::context!()).unwrap();

        assert!(result.contains("import foo from \"foo\""));
        assert!(result.contains("import bar from \"bar\""));

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
}
