use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Project-level stack configuration stored at `.tsx/stack.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackProfile {
    #[serde(default = "default_version")]
    pub version: String,
    /// Primary programming language: "typescript", "python", "rust", "go", "php"
    #[serde(default = "default_lang")]
    pub lang: String,
    /// Runtime environment: "node", "bun", "deno", "python", "go"
    #[serde(default)]
    pub runtime: Option<String>,
    /// Active packages stored as npm names (e.g. "@tanstack/start@1.2", "drizzle-orm")
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub style: StyleConfig,
    #[serde(default)]
    pub paths: PathConfig,
}

fn default_version() -> String {
    "1".to_string()
}
fn default_lang() -> String {
    "typescript".to_string()
}

/// Code style preferences applied to all generated files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    #[serde(default = "default_quotes")]
    pub quotes: String,
    #[serde(default = "default_indent")]
    pub indent: u8,
    #[serde(default = "default_semicolons")]
    pub semicolons: bool,
    /// CSS framework: "tailwind", "css-modules", "styled-components"
    #[serde(default = "default_css")]
    pub css: String,
    /// Component library: "shadcn", "radix", "headlessui", "none"
    #[serde(default = "default_components_style")]
    pub components: String,
    /// Form library: "tanstack-form", "react-hook-form", "none"
    #[serde(default = "default_forms")]
    pub forms: String,
    /// Icon library: "lucide-react", "heroicons", "none"
    #[serde(default = "default_icons")]
    pub icons: String,
    /// Toast/notification library: "sonner", "react-hot-toast", "none"
    #[serde(default = "default_toast")]
    pub toast: String,
}

fn default_quotes() -> String { "double".to_string() }
fn default_indent() -> u8 { 2 }
fn default_semicolons() -> bool { false }
fn default_css() -> String { "tailwind".to_string() }
fn default_components_style() -> String { "shadcn".to_string() }
fn default_forms() -> String { "tanstack-form".to_string() }
fn default_icons() -> String { "lucide-react".to_string() }
fn default_toast() -> String { "sonner".to_string() }

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            quotes: default_quotes(),
            indent: default_indent(),
            semicolons: default_semicolons(),
            css: default_css(),
            components: default_components_style(),
            forms: default_forms(),
            icons: default_icons(),
            toast: default_toast(),
        }
    }
}

/// Output path overrides — values are relative to the project root.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    #[serde(default = "default_path_components")]
    pub components: String,
    #[serde(default = "default_path_routes")]
    pub routes: String,
    #[serde(default = "default_path_db")]
    pub db: String,
    #[serde(default = "default_path_server_fns")]
    pub server_fns: String,
    #[serde(default = "default_path_hooks")]
    pub hooks: String,
}

fn default_path_components() -> String { "app/components".to_string() }
fn default_path_routes() -> String { "app/routes".to_string() }
fn default_path_db() -> String { "app/db".to_string() }
fn default_path_server_fns() -> String { "app/server".to_string() }
fn default_path_hooks() -> String { "app/hooks".to_string() }

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            components: default_path_components(),
            routes: default_path_routes(),
            db: default_path_db(),
            server_fns: default_path_server_fns(),
            hooks: default_path_hooks(),
        }
    }
}

// ---------------------------------------------------------------------------
// PatternConfig — code convention preferences
// ---------------------------------------------------------------------------

/// Code-pattern preferences applied to all generated files.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatternConfig {
    /// Component export style: "named-export", "default-export"
    #[serde(default)]
    pub component_style: Option<String>,
    /// File naming convention: "kebab-case", "camelCase", "PascalCase"
    #[serde(default)]
    pub file_naming: Option<String>,
    /// Import alias prefix: "@/", "~/", etc.
    #[serde(default)]
    pub import_alias: Option<String>,
}

// ---------------------------------------------------------------------------
// UserStack — user-local overrides (user-stack.json, not committed)
// ---------------------------------------------------------------------------

/// User-local stack overrides stored at `<project-root>/user-stack.json`.
/// This file is gitignored by convention — it captures per-developer preferences
/// on top of the shared `.tsx/stack.json`.
///
/// Merge resolution order (lowest → highest priority):
///   built-in defaults → framework registry.json → .tsx/stack.json
///   → user-stack.json → --path / --overwrite flags at call time
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserStack {
    /// Framework slug to extend (e.g. "tanstack-start")
    #[serde(default)]
    pub extends: Option<String>,
    /// Per-generator output path templates.
    /// Keys: "schema", "route", "server-fn", "component", "query-hook", etc.
    /// Values: path templates supporting {{name}}, {{feature}}, {{PascalName}},
    ///         {{kebab-name}}, {{snake_name}} placeholders.
    #[serde(default)]
    pub paths: HashMap<String, String>,
    /// Code-pattern preferences
    #[serde(default)]
    pub patterns: PatternConfig,
    /// Style overrides merged on top of .tsx/stack.json style
    #[serde(default)]
    pub style: UserStyleOverride,
    /// Template file overrides — key is generator id, value is path to custom .forge file
    #[serde(default)]
    pub templates: HashMap<String, String>,
    /// Slot content overrides — key is slot name, value is content or file path
    #[serde(default)]
    pub slots: HashMap<String, String>,
}

/// Partial style overrides from user-stack.json (all fields optional).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserStyleOverride {
    #[serde(default)]
    pub quotes: Option<String>,
    #[serde(default)]
    pub indent: Option<u8>,
    #[serde(default)]
    pub semicolons: Option<bool>,
    #[serde(default)]
    pub css: Option<String>,
    #[serde(default)]
    pub components: Option<String>,
    #[serde(default)]
    pub forms: Option<String>,
    #[serde(default)]
    pub icons: Option<String>,
    #[serde(default)]
    pub toast: Option<String>,
}

impl UserStack {
    /// Load from `<dir>/user-stack.json`. Returns `None` if the file doesn't exist.
    pub fn load(dir: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(dir.join("user-stack.json")).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Resolve a named path template (e.g. "schema") substituting `name` and `feature`.
    ///
    /// Supported placeholders: `{{name}}`, `{{feature}}`, `{{PascalName}}`,
    /// `{{kebab-name}}`, `{{snake_name}}`
    pub fn resolve_path(&self, key: &str, name: &str, feature: &str) -> Option<String> {
        use heck::{ToKebabCase, ToPascalCase, ToSnakeCase};
        let template = self.paths.get(key)?;
        let result = template
            .replace("{{name}}", name)
            .replace("{{feature}}", feature)
            .replace("{{PascalName}}", &name.to_pascal_case())
            .replace("{{kebab-name}}", &name.to_kebab_case())
            .replace("{{snake_name}}", &name.to_snake_case());
        Some(result)
    }

    /// Return the effective style by merging stack.json style with user-stack overrides.
    pub fn effective_style<'a>(&'a self, base: &'a StyleConfig) -> EffectiveStyle {
        EffectiveStyle {
            quotes: self.style.quotes.as_deref().unwrap_or(&base.quotes).to_string(),
            indent: self.style.indent.unwrap_or(base.indent),
            semicolons: self.style.semicolons.unwrap_or(base.semicolons),
            css: self.style.css.clone().unwrap_or_else(|| base.css.clone()),
            components: self.style.components.clone().unwrap_or_else(|| base.components.clone()),
            forms: self.style.forms.clone().unwrap_or_else(|| base.forms.clone()),
            icons: self.style.icons.clone().unwrap_or_else(|| base.icons.clone()),
            toast: self.style.toast.clone().unwrap_or_else(|| base.toast.clone()),
        }
    }
}

/// Resolved style configuration — all fields filled in after merging stack + user-stack.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EffectiveStyle {
    pub quotes: String,
    pub indent: u8,
    pub semicolons: bool,
    pub css: String,
    pub components: String,
    pub forms: String,
    pub icons: String,
    pub toast: String,
}

impl Default for StackProfile {
    fn default() -> Self {
        Self {
            version: default_version(),
            lang: default_lang(),
            runtime: None,
            packages: Vec::new(),
            style: StyleConfig::default(),
            paths: PathConfig::default(),
        }
    }
}

impl StackProfile {
    /// Canonical path: `<dir>/.tsx/stack.json`
    pub fn stack_file(dir: &Path) -> PathBuf {
        dir.join(".tsx").join("stack.json")
    }

    /// Load from `.tsx/stack.json` in the given directory.  Returns `None` if the file
    /// does not exist or cannot be parsed.
    pub fn load(dir: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(Self::stack_file(dir)).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Write to `.tsx/stack.json`, creating the `.tsx/` directory if needed.
    pub fn save(&self, dir: &Path) -> anyhow::Result<()> {
        let path = Self::stack_file(dir);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Add a package, replacing any existing entry with the same base name.
    pub fn add_package(&mut self, pkg: &str) {
        let base = base_name(pkg);
        self.packages.retain(|p| base_name(p) != base);
        self.packages.push(pkg.to_string());
    }

    /// Package names without version suffixes.
    pub fn package_names(&self) -> Vec<&str> {
        self.packages.iter().map(|p| base_name(p)).collect()
    }

    /// Inspect project files and return detected stack information.
    pub fn detect(dir: &Path) -> DetectedStack {
        let mut d = DetectedStack::default();

        let pkg_json = dir.join("package.json");
        if pkg_json.exists() {
            detect_js(dir, &pkg_json, &mut d);
        } else if dir.join("Cargo.toml").exists() {
            detect_rust(dir, &mut d);
        } else if dir.join("requirements.txt").exists() || dir.join("pyproject.toml").exists() {
            detect_python(dir, &mut d);
        } else if dir.join("go.mod").exists() {
            detect_go(dir, &mut d);
        }

        d
    }
}

fn base_name(pkg: &str) -> &str {
    if let Some(rest) = pkg.strip_prefix('@') {
        // Scoped package: @scope/name@version → @scope/name
        if let Some(at_idx) = rest.find('@') {
            &pkg[..at_idx + 1] // +1 for the leading '@'
        } else {
            pkg // no version suffix
        }
    } else {
        pkg.split('@').next().unwrap_or(pkg)
    }
}

// ---------------------------------------------------------------------------
// Detection helpers
// ---------------------------------------------------------------------------

fn detect_js(dir: &Path, pkg_json: &Path, d: &mut DetectedStack) {
    let Ok(content) = std::fs::read_to_string(pkg_json) else {
        return;
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) else {
        return;
    };

    d.lang = "typescript".to_string();
    d.runtime = Some(if dir.join("bun.lockb").exists() || dir.join("bun.lock").exists() {
        "bun".to_string()
    } else {
        "node".to_string()
    });

    let mut all_deps: HashMap<String, String> = HashMap::new();
    for key in ["dependencies", "devDependencies"] {
        if let Some(obj) = val.get(key).and_then(|v| v.as_object()) {
            for (k, v) in obj {
                all_deps.insert(k.clone(), v.as_str().unwrap_or("").to_string());
            }
        }
    }

    // npm package name in package.json → canonical npm name to store in stack.json
    // Use the canonical name (second column) for deduplication so aliases collapse.
    let mappings: &[(&str, &str)] = &[
        ("@tanstack/start",       "@tanstack/start"),
        ("@tanstack/react-start", "@tanstack/start"),   // alias
        ("next",                  "next"),
        ("drizzle-orm",           "drizzle-orm"),
        ("better-auth",           "better-auth"),
        ("@clerk/nextjs",         "@clerk/nextjs"),
        ("@tanstack/react-form",  "@tanstack/react-form"),
        ("@tanstack/react-table", "@tanstack/react-table"),
        ("@tanstack/react-query", "@tanstack/react-query"),
        ("tailwindcss",           "tailwindcss"),
        ("prisma",                "prisma"),
        ("@prisma/client",        "prisma"),            // alias
        ("kysely",                "kysely"),
        ("jotai",                 "jotai"),
        ("svelte",                "svelte"),
        ("solid-js",              "solid-js"),
        ("stripe",                "stripe"),
    ];

    for (npm_pkg, canonical) in mappings {
        if all_deps.contains_key(*npm_pkg) {
            let ver = all_deps[*npm_pkg]
                .trim_start_matches('^')
                .trim_start_matches('~')
                .to_string();
            let entry = if ver.is_empty() {
                canonical.to_string()
            } else {
                format!("{}@{}", canonical, ver)
            };
            if !d.packages.iter().any(|p| base_name(p) == *canonical) {
                d.packages.push(entry);
            }
        }
    }
}

fn detect_rust(dir: &Path, d: &mut DetectedStack) {
    d.lang = "rust".to_string();
    let content = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
    if content.contains("axum") {
        if content.contains("sea-orm") {
            d.packages.push("axum-sea-orm".to_string());
        } else {
            d.packages.push("axum".to_string());
        }
    } else if content.contains("actix-web") {
        d.packages.push("actix-web".to_string());
    }
    if content.contains("sqlx") {
        d.packages.push("sqlx".to_string());
    }
}

fn detect_python(dir: &Path, d: &mut DetectedStack) {
    d.lang = "python".to_string();
    d.runtime = Some("python".to_string());
    let content = std::fs::read_to_string(dir.join("requirements.txt"))
        .or_else(|_| std::fs::read_to_string(dir.join("pyproject.toml")))
        .unwrap_or_default();
    if content.contains("fastapi") {
        d.packages.push("fastapi-sqlalchemy".to_string());
    } else if content.contains("django") {
        d.packages.push("django".to_string());
    } else if content.contains("flask") {
        d.packages.push("flask".to_string());
    }
    if content.contains("sqlalchemy") && !d.packages.iter().any(|p| p.contains("sqlalchemy")) {
        d.packages.push("sqlalchemy".to_string());
    }
}

fn detect_go(dir: &Path, d: &mut DetectedStack) {
    d.lang = "go".to_string();
    d.runtime = Some("go".to_string());
    let content = std::fs::read_to_string(dir.join("go.mod")).unwrap_or_default();
    if content.contains("gin-gonic/gin") {
        d.packages.push("gin-gorm".to_string());
    } else if content.contains("gofiber/fiber") {
        d.packages.push("fiber".to_string());
    } else if content.contains("go-chi/chi") {
        d.packages.push("chi".to_string());
    }
    if content.contains("gorm.io") {
        d.packages.push("gorm".to_string());
    }
}

/// Output of `StackProfile::detect()`.
#[derive(Debug, Default)]
pub struct DetectedStack {
    pub lang: String,
    pub runtime: Option<String>,
    pub packages: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn save_and_load_round_trip() {
        let dir = TempDir::new().unwrap();
        let mut profile = StackProfile::default();
        profile.packages.push("@tanstack/start".to_string());
        profile.save(dir.path()).unwrap();
        let loaded = StackProfile::load(dir.path()).unwrap();
        assert_eq!(loaded.packages, vec!["@tanstack/start"]);
    }

    #[test]
    fn save_includes_style_and_path_defaults() {
        let dir = TempDir::new().unwrap();
        let profile = StackProfile::default();
        profile.save(dir.path()).unwrap();
        let json = std::fs::read_to_string(StackProfile::stack_file(dir.path())).unwrap();
        assert!(json.contains("\"css\""), "css field should be present");
        assert!(json.contains("\"tailwind\""), "css default should be tailwind");
        assert!(json.contains("\"components\""), "components field should be present");
        assert!(json.contains("\"app/routes\""), "routes path default should be present");
        assert!(json.contains("\"app/db\""), "db path default should be present");
    }

    #[test]
    fn add_package_deduplicates_by_base_name() {
        let mut p = StackProfile::default();
        p.add_package("drizzle-orm@0.36");
        p.add_package("drizzle-orm@0.37");
        assert_eq!(p.packages.len(), 1);
        assert_eq!(p.packages[0], "drizzle-orm@0.37");
    }

    #[test]
    fn add_package_deduplicates_scoped() {
        let mut p = StackProfile::default();
        p.add_package("@tanstack/start@1.2");
        p.add_package("@tanstack/start@1.3");
        assert_eq!(p.packages.len(), 1);
        assert_eq!(p.packages[0], "@tanstack/start@1.3");
    }

    #[test]
    fn detect_js_stack_from_package_json() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"dependencies":{"@tanstack/start":"^1.2","drizzle-orm":"^0.36","better-auth":"^1.0"}}"#,
        )
        .unwrap();
        let d = StackProfile::detect(dir.path());
        assert_eq!(d.lang, "typescript");
        assert!(d.packages.iter().any(|p| p.starts_with("@tanstack/start")), "should detect @tanstack/start");
        assert!(d.packages.iter().any(|p| p.starts_with("drizzle-orm")), "should detect drizzle-orm");
        assert!(d.packages.iter().any(|p| p.starts_with("better-auth")), "should detect better-auth");
    }

    #[test]
    fn detect_js_aliases_collapse() {
        let dir = TempDir::new().unwrap();
        // Both @tanstack/start and @tanstack/react-start map to @tanstack/start
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"dependencies":{"@tanstack/start":"^1.0","@tanstack/react-start":"^1.0"}}"#,
        )
        .unwrap();
        let d = StackProfile::detect(dir.path());
        let tanstack_count = d.packages.iter().filter(|p| base_name(p) == "@tanstack/start").count();
        assert_eq!(tanstack_count, 1, "aliases should collapse to one entry");
    }
}
