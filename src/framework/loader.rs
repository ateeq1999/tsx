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
}

impl Default for FrameworkLoader {
    fn default() -> Self {
        Self::new(PathBuf::from("frameworks"))
    }
}
