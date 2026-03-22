//! Template composition — `@extends` cycle detection.
//!
//! The actual `@extends` / `@slot` → `{% extends %}` / `{% block %}` translation
//! is handled by the preprocessor.  This module provides [`check_extends_cycle`],
//! which validates a template inheritance graph for circular dependencies before
//! any templates are rendered.
//!
//! # Usage
//!
//! ```rust,no_run
//! use tsx_forge::compose::{ExtendsGraph, check_extends_cycle};
//!
//! let mut graph = ExtendsGraph::new();
//! graph.add("child.forge", "base.forge");
//! graph.add("base.forge", "root.forge");
//!
//! // Returns Ok(()) — no cycle
//! check_extends_cycle(&graph).unwrap();
//!
//! graph.add("root.forge", "child.forge"); // introduces a cycle
//! assert!(check_extends_cycle(&graph).is_err());
//! ```

use std::collections::{HashMap, HashSet};

use crate::error::ForgeError;

// ---------------------------------------------------------------------------
// ExtendsGraph
// ---------------------------------------------------------------------------

/// A directed graph of `child → parent` `@extends` relationships.
#[derive(Debug, Default, Clone)]
pub struct ExtendsGraph {
    /// Maps each template name to the parent it extends, if any.
    edges: HashMap<String, String>,
}

impl ExtendsGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register that `child` extends `parent`.
    pub fn add(&mut self, child: impl Into<String>, parent: impl Into<String>) {
        self.edges.insert(child.into(), parent.into());
    }

    /// Return the parent of `template`, if one was registered.
    pub fn parent_of(&self, template: &str) -> Option<&str> {
        self.edges.get(template).map(String::as_str)
    }

    /// Return an iterator over all `(child, parent)` pairs.
    pub fn edges(&self) -> impl Iterator<Item = (&str, &str)> {
        self.edges.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

// ---------------------------------------------------------------------------
// Cycle detection
// ---------------------------------------------------------------------------

/// Walk the `@extends` chain starting from `start`, collecting the path.
/// Returns `Err(ForgeError::CircularDependency)` if a cycle is detected.
pub fn check_extends_cycle(graph: &ExtendsGraph) -> Result<(), ForgeError> {
    for start in graph.edges.keys() {
        let mut visited: HashSet<&str> = HashSet::new();
        let mut current = start.as_str();
        let mut path: Vec<&str> = vec![current];

        while let Some(parent) = graph.parent_of(current) {
            if !visited.insert(parent) {
                // Already seen — cycle detected
                path.push(parent);
                let chain = path.join(" → ");
                return Err(ForgeError::CircularDependency(chain));
            }
            path.push(parent);
            current = parent;
        }
    }
    Ok(())
}

/// Check if adding `child → parent` to `graph` would introduce a cycle.
/// Does **not** mutate the graph.
pub fn would_cycle(graph: &ExtendsGraph, child: &str, parent: &str) -> bool {
    // Walk upward from `parent`; if we reach `child`, there would be a cycle.
    let mut current = parent;
    let mut visited: HashSet<&str> = HashSet::new();
    visited.insert(child);

    loop {
        if current == child {
            return true;
        }
        if !visited.insert(current) {
            // Already in visited set (cycle among existing edges, not involving `child`)
            return false;
        }
        match graph.parent_of(current) {
            Some(p) => current = p,
            None    => return false,
        }
    }
}

// ---------------------------------------------------------------------------
// Schema extraction helper (used by Engine)
// ---------------------------------------------------------------------------

/// Extract the `@extends("path")` target from the first line of a forge template.
/// Returns `None` if the template does not start with `@extends`.
pub fn extract_extends_path(src: &str) -> Option<String> {
    for line in src.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("@extends(") {
            // Find the quoted path
            let inner = rest.trim_end_matches(')');
            let path = inner.trim().trim_matches(|c| c == '"' || c == '\'');
            return Some(path.to_string());
        }
        // Skip blank lines and @schema; stop at any other content
        if !trimmed.is_empty()
            && !trimmed.starts_with("{#")
            && !trimmed.starts_with("@schema")
        {
            break;
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_cycle_passes() {
        let mut g = ExtendsGraph::new();
        g.add("child.forge", "base.forge");
        g.add("base.forge", "root.forge");
        assert!(check_extends_cycle(&g).is_ok());
    }

    #[test]
    fn direct_cycle_detected() {
        let mut g = ExtendsGraph::new();
        g.add("a.forge", "b.forge");
        g.add("b.forge", "a.forge");
        let err = check_extends_cycle(&g).unwrap_err();
        assert!(err.to_string().contains("F002"), "{err}");
    }

    #[test]
    fn indirect_cycle_detected() {
        let mut g = ExtendsGraph::new();
        g.add("a.forge", "b.forge");
        g.add("b.forge", "c.forge");
        g.add("c.forge", "a.forge");
        assert!(check_extends_cycle(&g).is_err());
    }

    #[test]
    fn would_cycle_detects_future_cycle() {
        let mut g = ExtendsGraph::new();
        g.add("a.forge", "b.forge");
        // Adding b.forge → a.forge would close a cycle
        assert!(would_cycle(&g, "b.forge", "a.forge"));
        // Adding c.forge → a.forge is fine
        assert!(!would_cycle(&g, "c.forge", "a.forge"));
    }

    #[test]
    fn extract_extends_path_found() {
        let src = "@extends(\"layouts/base.forge\")\n@slot(\"x\")\nhello\n@end";
        assert_eq!(
            extract_extends_path(src),
            Some("layouts/base.forge".to_string())
        );
    }

    #[test]
    fn extract_extends_path_absent() {
        let src = "{% if x %}hello{% endif %}";
        assert_eq!(extract_extends_path(src), None);
    }
}
