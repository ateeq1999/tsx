use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainResult {
    pub topic: String,
    pub purpose: String,
    pub decisions: Vec<Decision>,
    pub learn_more: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub title: String,
    pub rationale: String,
    pub alternative: Option<String>,
}

static DECISION_KNOWLEDGE_BASE: &[(&str, &str, &[(&str, &str, Option<&str>)])] = &[
    (
        "atom",
        "Atoms are the smallest reusable UI components that cannot be broken down further",
        &[
            (
                "Single file component",
                "Atoms should be self-contained in a single file for easy reuse",
                Some("Consider splitting if the component grows complex"),
            ),
            (
                "No internal state",
                "Atoms should be stateless - use molecules for components with state",
                Some("Use useState or useReducer in molecules instead"),
            ),
            (
                "Named exports",
                "Use named exports for better tree-shaking",
                Some("Default exports work but are less explicit"),
            ),
        ],
    ),
    (
        "molecule",
        "Molecules are composite components that combine atoms with local state",
        &[
            (
                "Props drilling avoidance",
                "Molecules manage their own state to avoid prop drilling",
                Some("For deeply nested state, use context in organisms"),
            ),
            (
                "Composition over creation",
                "Molecules compose atoms rather than creating new UI",
                Some("Reuse existing atoms before creating new ones"),
            ),
        ],
    ),
    (
        "layout",
        "Layouts define the structure and wrapping of routes",
        &[
            (
                "File-based routing",
                "Layouts map directly to route file structure",
                Some("Use nested routes for complex hierarchies"),
            ),
            (
                "Shared state via context",
                "Layouts can provide context to child routes",
                Some("Keep layout state minimal to avoid coupling"),
            ),
        ],
    ),
    (
        "feature",
        "Features are complete CRUD modules with routes, components, and server functions",
        &[
            (
                "Convention over configuration",
                "Features follow strict conventions for consistency",
                Some("Custom structures require more maintenance"),
            ),
            (
                "Server functions colocated",
                "Server functions live next to their route for co-location",
                Some("Extract to lib/ only if shared across features"),
            ),
            (
                "Barrel exports",
                "Features export everything from index for clean imports",
                Some("Barrels can hide internal structure - use with care"),
            ),
        ],
    ),
    (
        "schema",
        "Database schemas define table structures using Drizzle ORM",
        &[
            (
                "Type safety",
                "Schemas generate TypeScript types automatically",
                Some("Manual types can drift from schema"),
            ),
            (
                "Idiomatic column order",
                "id, created_at, updated_at, then user columns",
                Some("Consistent order helps readability"),
            ),
        ],
    ),
    (
        "server function",
        "Server functions are type-safe RPC calls between client and server",
        &[
            (
                "Named exports",
                "Server functions must be named exports",
                Some("Anonymous functions lose type info"),
            ),
            (
                "Input validation",
                "Always validate inputs with Zod",
                Some("Skip validation only for internal APIs"),
            ),
        ],
    ),
    (
        "query",
        "TanStack Query hooks for data fetching with caching",
        &[
            (
                "Query keys as arrays",
                "Use array format for invalidation granularity",
                Some("String keys prevent partial invalidation"),
            ),
            (
                "Enabled: false pattern",
                "Disable queries until required data exists",
                Some("Always enabled can cause unnecessary fetches"),
            ),
        ],
    ),
    (
        "form",
        "TanStack Form components with validation",
        &[
            (
                "Controlled inputs",
                "Forms use controlled components for state",
                Some("Uncontrolled has performance benefits but less control"),
            ),
            (
                "Field-level validation",
                "Validate individual fields for better UX",
                Some("Form-level is simpler but less responsive"),
            ),
        ],
    ),
    (
        "auth",
        "Authentication configuration using Better Auth",
        &[
            (
                "Singleton pattern",
                "Auth instance created once and reused",
                Some("Multiple instances can cause session conflicts"),
            ),
            (
                "TypeScript strict",
                "Full type safety for auth types",
                Some("Loose types hide bugs"),
            ),
        ],
    ),
];

/// Returns a relevance score (0–100) for fuzzy topic matching.
fn fuzzy_score(needle: &str, haystack: &str) -> u32 {
    let n = needle.to_lowercase();
    let h = haystack.to_lowercase();

    if h == n {
        return 100;
    }
    if h.contains(&n) || n.contains(&h) {
        return 80;
    }

    let n_words: std::collections::HashSet<&str> = n.split_whitespace().collect();
    let h_words: std::collections::HashSet<&str> = h.split_whitespace().collect();
    let overlap = n_words.intersection(&h_words).count();
    if overlap > 0 {
        return 60 + (overlap.min(4) as u32 * 5);
    }

    0
}

pub fn explain(topic: String, verbose: bool) -> CommandResult {
    let start = Instant::now();

    // Rank all knowledge-base entries by fuzzy score.
    let best = DECISION_KNOWLEDGE_BASE
        .iter()
        .map(|(key, purpose, decisions)| (key, purpose, decisions, fuzzy_score(&topic, key)))
        .filter(|(_, _, _, score)| *score > 0)
        .max_by_key(|(_, _, _, score)| *score);

    if let Some((key, purpose, decisions, _)) = best {
        let duration_ms = start.elapsed().as_millis() as u64;
        let result = ExplainResult {
            topic: key.to_string(),
            purpose: purpose.to_string(),
            decisions: decisions
                .iter()
                .map(|(title, rationale, alternative)| Decision {
                    title: title.to_string(),
                    rationale: rationale.to_string(),
                    alternative: alternative.map(|s| s.to_string()),
                })
                .collect(),
            learn_more: vec![],
        };

        let response = ResponseEnvelope::success(
            "explain",
            serde_json::to_value(result).unwrap(),
            duration_ms,
        );

        if verbose {
            let context = crate::json::response::Context {
                project_root: std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                tsx_version: env!("CARGO_PKG_VERSION").to_string(),
            };
            response.with_context(context).print();
        } else {
            response.print();
        }

        return CommandResult::ok("explain", vec![]);
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let available: Vec<String> = DECISION_KNOWLEDGE_BASE
        .iter()
        .map(|(key, _, _)| key.to_string())
        .collect();

    let error = crate::json::error::ErrorResponse::validation(&format!(
        "Topic '{}' not found. Available: {}",
        topic,
        available.join(", ")
    ));
    ResponseEnvelope::error("explain", error, duration_ms).print();
    CommandResult::err("explain", "Topic not found")
}
