use std::path::Path;

/// Detects the active framework slug from the project's `package.json` dependencies.
/// Returns the first matching known framework slug, or `None` if no match.
pub fn detect_framework(project_root: &Path) -> Option<String> {
    let pkg_path = project_root.join("package.json");
    let content = std::fs::read_to_string(pkg_path).ok()?;
    let pkg: serde_json::Value = serde_json::from_str(&content).ok()?;

    // Collect all dep keys from dependencies + devDependencies
    let mut dep_keys: Vec<String> = vec![];
    for section in ["dependencies", "devDependencies"] {
        if let Some(deps) = pkg.get(section).and_then(|v| v.as_object()) {
            dep_keys.extend(deps.keys().cloned());
        }
    }

    // Map known dependency patterns → framework slugs (ordered by specificity)
    for key in &dep_keys {
        let k = key.as_str();
        if k == "@tanstack/start" || k.starts_with("@tanstack/start-") {
            return Some("tanstack-start".to_string());
        }
    }

    // Secondary heuristics — broader packages
    for key in &dep_keys {
        let k = key.as_str();
        if k == "@tanstack/react-router" || k == "@tanstack/router" {
            return Some("tanstack-start".to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_pkg(dir: &std::path::Path, content: &str) {
        fs::write(dir.join("package.json"), content).unwrap();
    }

    #[test]
    fn detects_tanstack_start_direct() {
        let tmp = tempfile::TempDir::new().unwrap();
        make_pkg(
            tmp.path(),
            r#"{"dependencies":{"@tanstack/start":"0.1.0"}}"#,
        );
        assert_eq!(detect_framework(tmp.path()), Some("tanstack-start".to_string()));
    }

    #[test]
    fn detects_tanstack_router_fallback() {
        let tmp = tempfile::TempDir::new().unwrap();
        make_pkg(
            tmp.path(),
            r#"{"devDependencies":{"@tanstack/react-router":"1.0.0"}}"#,
        );
        assert_eq!(detect_framework(tmp.path()), Some("tanstack-start".to_string()));
    }

    #[test]
    fn returns_none_for_unknown_deps() {
        let tmp = tempfile::TempDir::new().unwrap();
        make_pkg(tmp.path(), r#"{"dependencies":{"react":"18.0.0"}}"#);
        assert_eq!(detect_framework(tmp.path()), None);
    }

    #[test]
    fn returns_none_for_missing_package_json() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert_eq!(detect_framework(tmp.path()), None);
    }
}
