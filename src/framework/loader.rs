use crate::framework::registry::{FrameworkInfo, FrameworkRegistry};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct FrameworkLoader {
    builtin_path: PathBuf,
    cache: HashMap<String, FrameworkRegistry>,
}

impl FrameworkLoader {
    pub fn new(builtin_path: PathBuf) -> Self {
        Self {
            builtin_path,
            cache: HashMap::new(),
        }
    }

    pub fn load_builtin_frameworks(&mut self) -> Vec<FrameworkInfo> {
        let mut frameworks = Vec::new();

        if !self.builtin_path.exists() {
            return frameworks;
        }

        if let Ok(entries) = fs::read_dir(&self.builtin_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let registry_path = path.join("registry.json");
                    if registry_path.exists() {
                        if let Ok(registry) = self.load_registry_from_path(&path) {
                            self.cache.insert(registry.slug.clone(), registry.clone());
                            frameworks.push(FrameworkInfo::from(&registry));
                        }
                    }
                }
            }
        }

        frameworks
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

impl Default for FrameworkLoader {
    fn default() -> Self {
        Self::new(PathBuf::from("frameworks"))
    }
}
