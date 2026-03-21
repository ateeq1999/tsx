use crate::framework::registry::{FrameworkInfo, FrameworkRegistry};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct FrameworkLoader {
    builtin_path: PathBuf,
    /// User-installed frameworks directory (e.g. `.tsx/frameworks/` in the project root).
    user_path: Option<PathBuf>,
    cache: HashMap<String, FrameworkRegistry>,
}

impl FrameworkLoader {
    pub fn new(builtin_path: PathBuf) -> Self {
        Self {
            builtin_path,
            user_path: None,
            cache: HashMap::new(),
        }
    }

    /// Set an additional search path for user-installed framework packages
    /// (e.g. `.tsx/frameworks/` inside a project).
    pub fn with_user_path(mut self, path: PathBuf) -> Self {
        self.user_path = Some(path);
        self
    }

    pub fn load_builtin_frameworks(&mut self) -> Vec<FrameworkInfo> {
        let mut frameworks = Vec::new();

        // Scan builtin path
        frameworks.extend(self.scan_dir(self.builtin_path.clone()));

        // Also scan user-installed frameworks dir if set or auto-detected
        let user_dir = self.user_path.clone().or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|d| d.join(".tsx").join("frameworks"))
        });
        if let Some(dir) = user_dir {
            if dir.exists() {
                frameworks.extend(self.scan_dir(dir));
            }
        }

        frameworks
    }

    fn scan_dir(&mut self, dir: PathBuf) -> Vec<FrameworkInfo> {
        let mut found = Vec::new();
        if !dir.exists() {
            return found;
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Try registry.json (legacy format) first, then manifest.json (v6 format)
                    if path.join("registry.json").exists() {
                        if let Ok(registry) = self.load_registry_from_path(&path) {
                            self.cache.insert(registry.slug.clone(), registry.clone());
                            found.push(FrameworkInfo::from(&registry));
                        }
                    } else if path.join("manifest.json").exists() {
                        if let Some(info) = load_manifest_as_info(&path) {
                            // Build a minimal FrameworkRegistry so ask/where/how can find it
                            let registry = manifest_to_registry(&path, &info);
                            self.cache.insert(registry.slug.clone(), registry.clone());
                            found.push(info);
                        }
                    }
                }
            }
        }
        found
    }

    pub fn load_registry_from_path(&self, path: &PathBuf) -> Result<FrameworkRegistry, String> {
        let registry_path = path.join("registry.json");
        let content = fs::read_to_string(&registry_path)
            .map_err(|e| format!("Failed to read registry: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse registry: {}", e))
    }

    #[allow(dead_code)]
    pub async fn load_registry_from_npm(&mut self, slug: &str) -> Result<FrameworkRegistry, String> {
        let package_name = format!("@tsx-framework/{}", slug);
        
        let url = format!(
            "https://registry.npmjs.org/{}",
            package_name.replace("/", "%2F")
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch package: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Package not found: {}", slug));
        }

        let package_info: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let latest_version = package_info
            .get("dist-tags")
            .and_then(|tags| tags.get("latest"))
            .and_then(|v| v.as_str())
            .ok_or("No latest version found")?;

        let registry_url = package_info
            .get("versions")
            .and_then(|versions| versions.get(latest_version))
            .and_then(|version| version.get("tsx-framework"))
            .and_then(|tf| tf.get("registry"))
            .and_then(|r| r.as_str())
            .map(|s| s.to_string());

        if let Some(registry_content_url) = registry_url {
            let registry_response = client
                .get(&registry_content_url)
                .send()
                .await
                .map_err(|e| format!("Failed to fetch registry: {}", e))?;

            let registry: FrameworkRegistry = registry_response
                .json()
                .await
                .map_err(|e| format!("Failed to parse registry: {}", e))?;

            self.cache.insert(registry.slug.clone(), registry.clone());
            return Ok(registry);
        }

        let tarball_url = package_info
            .get("versions")
            .and_then(|versions| versions.get(latest_version))
            .and_then(|version| version.get("dist"))
            .and_then(|dist| dist.get("tarball"))
            .and_then(|t| t.as_str())
            .ok_or("No tarball URL found")?
            .to_string();

        Err(format!("Registry not found in package. Tarball: {}", tarball_url))
    }

    pub fn get_registry(&self, slug: &str) -> Option<&FrameworkRegistry> {
        self.cache.get(slug)
    }

    pub fn load_conventions(&self, slug: &str) -> Result<serde_json::Value, String> {
        let conventions_path = self.builtin_path.join(slug).join("conventions.json");

        if !conventions_path.exists() {
            return Err(format!("Conventions not found for {}", slug));
        }

        let content = fs::read_to_string(&conventions_path)
            .map_err(|e| format!("Failed to read conventions: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse conventions: {}", e))
    }

    pub fn list_frameworks(&self) -> Vec<FrameworkInfo> {
        self.cache.values().map(FrameworkInfo::from).collect()
    }

    pub fn discover_frameworks(&mut self) -> Vec<FrameworkInfo> {
        self.load_builtin_frameworks()
    }
}

/// Load a `manifest.json` (v6 format) and return a `FrameworkInfo`.
fn load_manifest_as_info(pkg_dir: &std::path::Path) -> Option<FrameworkInfo> {
    use crate::framework::registry::{FrameworkCategory};
    let content = fs::read_to_string(pkg_dir.join("manifest.json")).ok()?;
    let m: serde_json::Value = serde_json::from_str(&content).ok()?;

    let slug = m.get("id").and_then(|v| v.as_str())?.to_string();
    let name = m.get("name").and_then(|v| v.as_str()).unwrap_or(&slug).to_string();
    let version = m.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();
    let description = m.get("description").and_then(|v| v.as_str())
        .unwrap_or(&name).to_string();
    let docs = m.get("docs").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let github = m.get("github").and_then(|v| v.as_str()).map(|s| s.to_string());

    let category = match m.get("category").and_then(|v| v.as_str()).unwrap_or("framework") {
        "orm" => FrameworkCategory::Orm,
        "auth" => FrameworkCategory::Auth,
        "ui" => FrameworkCategory::Ui,
        "tool" => FrameworkCategory::Tool,
        _ => FrameworkCategory::Framework,
    };

    Some(FrameworkInfo { slug, name, version, description, docs, category, github })
}

/// Build a minimal `FrameworkRegistry` from a `manifest.json` directory so that
/// query commands (`ask`, `where`, `how`) can find the framework in the cache.
/// Loads questions from `knowledge/faq.md` if present.
fn manifest_to_registry(
    pkg_dir: &std::path::Path,
    info: &FrameworkInfo,
) -> FrameworkRegistry {
    use crate::framework::registry::{
        Conventions, FrameworkRegistry, NamingConvention, ProjectStructure,
    };
    use crate::framework::knowledge::load_section;

    // Try to load FAQ questions from knowledge/faq.md
    let questions = load_section(pkg_dir, "faq")
        .map(|entry| parse_faq_to_questions(&entry.content))
        .unwrap_or_default();

    FrameworkRegistry {
        framework: info.name.clone(),
        version: info.version.clone(),
        slug: info.slug.clone(),
        category: info.category.clone(),
        docs: info.docs.clone(),
        github: info.github.clone(),
        structure: ProjectStructure::default(),
        generators: vec![],
        conventions: Conventions {
            files: Default::default(),
            naming: NamingConvention::default(),
            patterns: vec![],
        },
        injection_points: vec![],
        integrations: vec![],
        questions,
    }
}

/// Parse FAQ markdown into `Question` structs.
/// Each question block starts with `## question` and has an `answer:` section.
fn parse_faq_to_questions(content: &str) -> Vec<crate::framework::registry::Question> {
    use crate::framework::registry::Question;

    let mut questions = Vec::new();
    let mut current_topic: Option<String> = None;
    let mut current_answer_lines: Vec<String> = Vec::new();
    let mut in_answer = false;

    for line in content.lines() {
        if line.starts_with("## ") {
            // Save previous question
            if let Some(topic) = current_topic.take() {
                let answer = current_answer_lines.join("\n").trim().to_string();
                if !answer.is_empty() {
                    questions.push(Question {
                        topic,
                        answer,
                        steps: vec![],
                        files_affected: vec![],
                        dependencies: vec![],
                        learn_more: vec![],
                    });
                }
            }
            current_topic = Some(line.trim_start_matches("## ").trim().to_string());
            current_answer_lines.clear();
            in_answer = false;
        } else if line.trim_start().starts_with("answer:") || line.trim() == "answer:" {
            in_answer = true;
            let after = line.trim_start_matches("answer:").trim();
            if !after.is_empty() {
                current_answer_lines.push(after.to_string());
            }
        } else if in_answer && !line.starts_with("---") {
            current_answer_lines.push(line.to_string());
        }
    }

    // Final question
    if let Some(topic) = current_topic {
        let answer = current_answer_lines.join("\n").trim().to_string();
        if !answer.is_empty() {
            questions.push(Question {
                topic,
                answer,
                steps: vec![],
                files_affected: vec![],
                dependencies: vec![],
                learn_more: vec![],
            });
        }
    }

    questions
}

impl Default for FrameworkLoader {
    fn default() -> Self {
        // Check next to the installed binary first, then fall back to cwd.
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        let builtin_path = exe_dir
            .map(|d| d.join("frameworks"))
            .filter(|p| p.exists())
            .unwrap_or_else(|| PathBuf::from("frameworks"));

        // Auto-detect user-installed frameworks in the project's .tsx/frameworks/ dir
        let user_path = std::env::current_dir()
            .ok()
            .map(|d| d.join(".tsx").join("frameworks"));

        Self {
            builtin_path,
            user_path,
            cache: HashMap::new(),
        }
    }
}
