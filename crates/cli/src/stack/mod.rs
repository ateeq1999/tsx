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
    /// Active tsx packages (e.g. "tanstack-start@1.2", "drizzle-pg")
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
    #[serde(default)]
    pub css: Option<String>,
    /// Component library: "shadcn", "radix", "headlessui", "none"
    #[serde(default)]
    pub components: Option<String>,
    /// Form library: "tanstack-form", "react-hook-form", "none"
    #[serde(default)]
    pub forms: Option<String>,
    /// Icon library: "lucide-react", "heroicons", "none"
    #[serde(default)]
    pub icons: Option<String>,
    /// Toast/notification library: "sonner", "react-hot-toast", "none"
    #[serde(default)]
    pub toast: Option<String>,
}

fn default_quotes() -> String {
    "double".to_string()
}
fn default_indent() -> u8 {
    2
}
fn default_semicolons() -> bool {
    false
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            quotes: default_quotes(),
            indent: default_indent(),
            semicolons: default_semicolons(),
            css: None,
            components: None,
            forms: None,
            icons: None,
            toast: None,
        }
    }
}

/// Output path overrides — values are relative to the project root.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathConfig {
    #[serde(default)]
    pub components: Option<String>,
    #[serde(default)]
    pub routes: Option<String>,
    #[serde(default)]
    pub db: Option<String>,
    #[serde(default)]
    pub server_fns: Option<String>,
    #[serde(default)]
    pub hooks: Option<String>,
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
            css: self.style.css.clone()
                .or_else(|| base.css.clone())
                .unwrap_or_default(),
            components: self.style.components.clone()
                .or_else(|| base.components.clone())
                .unwrap_or_default(),
            forms: self.style.forms.clone()
                .or_else(|| base.forms.clone())
                .unwrap_or_default(),
            icons: self.style.icons.clone()
                .or_else(|| base.icons.clone())
                .unwrap_or_default(),
            toast: self.style.toast.clone()
                .or_else(|| base.toast.clone())
                .unwrap_or_default(),
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
    pkg.split('@').next().unwrap_or(pkg)
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

    // npm package → @tsx-pkg suggestion
    let mappings: &[(&str, &str)] = &[
        ("@tanstack/start", "tanstack-start"),
        ("@tanstack/react-start", "tanstack-start"),
        ("next", "nextjs"),
        ("drizzle-orm", "drizzle-pg"), // dialect refined below
        ("better-auth", "better-auth"),
        ("@clerk/nextjs", "clerk"),
        ("@tanstack/react-form", "tanstack-form"),
        ("@tanstack/react-table", "tanstack-table"),
        ("@tanstack/react-query", "tanstack-query"),
        ("tailwindcss", "tailwindcss"),
        ("prisma", "prisma"),
        ("@prisma/client", "prisma"),
        ("kysely", "kysely"),
        ("jotai", "jotai"),
        ("svelte", "svelte"),
        ("solid-js", "solid"),
        ("stripe", "stripe"),
    ];

    for (npm_pkg, tsx_pkg) in mappings {
        if all_deps.contains_key(*npm_pkg) {
            let ver = all_deps[*npm_pkg]
                .trim_start_matches('^')
                .trim_start_matches('~')
                .to_string();
            let suggestion = if ver.is_empty() {
                tsx_pkg.to_string()
            } else {
                format!("{}@{}", tsx_pkg, ver)
            };
            if !d.packages.iter().any(|p| base_name(p) == *tsx_pkg) {
                d.packages.push(suggestion);
            }
        }
    }

    // Refine drizzle dialect
    if all_deps.contains_key("drizzle-orm") {
        let drizzle_idx = d.packages.iter().position(|p| base_name(p) == "drizzle-pg");
        let dialect = if all_deps.contains_key("mysql2") {
            Some("drizzle-mysql")
        } else if all_deps.contains_key("better-sqlite3") || all_deps.contains_key("@libsql/client") {
            Some("drizzle-sqlite")
        } else {
            None // keep drizzle-pg as default for postgres/pg
        };
        if let Some(d_pkg) = dialect {
            if let Some(i) = drizzle_idx {
                d.packages[i] = d_pkg.to_string();
            } else {
                d.packages.push(d_pkg.to_string());
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
        profile.packages.push("tanstack-start".to_string());
        profile.save(dir.path()).unwrap();
        let loaded = StackProfile::load(dir.path()).unwrap();
        assert_eq!(loaded.packages, vec!["tanstack-start"]);
    }

    #[test]
    fn add_package_deduplicates_by_base_name() {
        let mut p = StackProfile::default();
        p.add_package("drizzle-pg@0.36");
        p.add_package("drizzle-pg@0.37");
        assert_eq!(p.packages.len(), 1);
        assert_eq!(p.packages[0], "drizzle-pg@0.37");
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
        assert!(d.packages.iter().any(|p| p.starts_with("tanstack-start")));
        assert!(d.packages.iter().any(|p| p.starts_with("drizzle-pg")));
        assert!(d.packages.iter().any(|p| p.starts_with("better-auth")));
    }
}
