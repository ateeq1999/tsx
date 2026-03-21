//! Formatter configuration.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FmtConfig {
    /// Spaces per indent level (default: 2)
    pub indent: usize,
    /// Quote style for string literals in @import and Tera expressions
    pub quotes: QuoteStyle,
    /// Maximum consecutive blank lines (default: 1)
    pub max_blank_lines: usize,
    /// Whether to normalise Tera delimiter spacing  (default: true)
    pub normalise_tera_spacing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuoteStyle {
    Single,
    Double,
}

impl Default for FmtConfig {
    fn default() -> Self {
        Self {
            indent: 2,
            quotes: QuoteStyle::Double,
            max_blank_lines: 1,
            normalise_tera_spacing: true,
        }
    }
}

impl FmtConfig {
    pub fn from_file(path: &std::path::Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Load from `.tsx/fmt.json` if it exists, falling back to defaults.
    pub fn load_project() -> Self {
        let project_cfg = crate::find_project_root()
            .map(|r| r.join(".tsx").join("fmt.json"))
            .filter(|p| p.exists());

        match project_cfg {
            Some(p) => Self::from_file(&p),
            None => Self::default(),
        }
    }

    pub fn quote(&self, s: &str) -> String {
        match self.quotes {
            QuoteStyle::Double => format!("\"{}\"", s),
            QuoteStyle::Single => format!("'{}'", s),
        }
    }
}

pub(crate) fn find_project_root() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let mut dir = cwd.as_path();
    loop {
        if dir.join("package.json").exists() || dir.join("Cargo.toml").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}
