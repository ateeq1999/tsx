//! Import hoisting for code generation templates.
//!
//! Templates call `{{ "import foo from 'foo'" | collect_import }}` during render.
//! After render, `{{ render_imports() }}` drains the collector and emits a
//! deduplicated, sorted import block at the top of the generated file.
//!
//! Uses thread-local storage so Tera filter/function callbacks (which must be
//! `Send + Sync + 'static`) can still mutate state on the current thread.

use std::cell::RefCell;
use std::collections::BTreeSet;

thread_local! {
    static IMPORTS: RefCell<BTreeSet<String>> = RefCell::new(BTreeSet::new());
    static PRIORITY: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

/// Reset both collectors. Call this before each render to avoid cross-render pollution.
pub fn reset() {
    IMPORTS.with(|c| c.borrow_mut().clear());
    PRIORITY.with(|c| c.borrow_mut().clear());
}

/// Push an import to the regular collector. Returns an empty string so it can
/// be used as a Tera filter: `{{ "import x from 'x'" | collect_import }}`.
pub fn collect(import: String) -> String {
    IMPORTS.with(|c| {
        c.borrow_mut().insert(import);
    });
    String::new()
}

/// Push an import to the priority queue (rendered before regular imports).
pub fn collect_priority(import: String) -> String {
    PRIORITY.with(|c| c.borrow_mut().push(import));
    String::new()
}

/// Drain both collectors and return the complete import block.
/// Priority imports appear first, then the sorted regular imports.
pub fn drain() -> String {
    let priority = PRIORITY.with(|c| c.borrow().clone());
    let rest: Vec<String> = IMPORTS.with(|c| c.borrow().iter().cloned().collect());
    let mut all = priority;
    for imp in rest {
        if !all.contains(&imp) {
            all.push(imp);
        }
    }
    all.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_and_drain() {
        reset();
        collect("import { z } from 'zod'".to_string());
        collect("import React from 'react'".to_string());
        let out = drain();
        assert!(out.contains("import { z }"));
        assert!(out.contains("import React"));
        reset();
    }

    #[test]
    fn priority_comes_first() {
        reset();
        collect("import { z } from 'zod'".to_string());
        collect_priority("import React from 'react'".to_string());
        let out = drain();
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "import React from 'react'");
        reset();
    }

    #[test]
    fn deduplicates_imports() {
        reset();
        collect("import { z } from 'zod'".to_string());
        collect("import { z } from 'zod'".to_string());
        let out = drain();
        assert_eq!(out.matches("import { z }").count(), 1);
        reset();
    }
}
