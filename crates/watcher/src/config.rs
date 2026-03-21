//! Watcher configuration.

use std::time::Duration;

#[derive(Debug, Clone)]
pub struct WatchConfig {
    /// Root directories to watch (recursive)
    pub roots: Vec<String>,
    /// Only watch files with these extensions (empty = all files)
    pub extensions: Vec<String>,
    /// Debounce window — wait this long after the last change before firing
    pub debounce: Duration,
    /// Emit structured JSON events on stdout
    pub json_events: bool,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            roots: vec![".".to_string()],
            extensions: Vec::new(),
            debounce: Duration::from_millis(300),
            json_events: false,
        }
    }
}

impl WatchConfig {
    /// Watch Rust source files in a crate
    pub fn rust_sources(root: impl Into<String>) -> Self {
        Self {
            roots: vec![root.into()],
            extensions: vec!["rs".to_string()],
            ..Default::default()
        }
    }

    /// Watch .forge / .jinja template files
    pub fn templates(root: impl Into<String>) -> Self {
        Self {
            roots: vec![root.into()],
            extensions: vec!["forge".to_string(), "jinja".to_string(), "jinja2".to_string()],
            ..Default::default()
        }
    }

    /// Watch TypeScript/JavaScript source files
    pub fn typescript(root: impl Into<String>) -> Self {
        Self {
            roots: vec![root.into()],
            extensions: vec![
                "ts".to_string(), "tsx".to_string(),
                "js".to_string(), "jsx".to_string(),
            ],
            ..Default::default()
        }
    }
}
