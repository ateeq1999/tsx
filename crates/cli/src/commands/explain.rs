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

pub fn explain(topic: String, verbose: bool) -> CommandResult {
    let start = Instant::now();
    let duration_ms = start.elapsed().as_millis() as u64;

    let topic_lower = topic.to_lowercase();

    let found = DECISION_KNOWLEDGE_BASE
        .iter()
        .find(|(key, _, _)| topic_lower.contains(key) || key.contains(&topic_lower));

    if let Some((key, purpose, decisions)) = found {
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

    let available: Vec<String> = DECISION_KNOWLEDGE_BASE
        .iter()
        .map(|(key, _, _)| key.to_string())
        .collect();

    let error = crate::json::error::ErrorResponse::validation(&format!(
        "Topic '{}' not found. Available: {}",
        topic,
        available.join(", ")
    ));
    let response = ResponseEnvelope::error("explain", error, duration_ms);
    response.print();
    CommandResult::err("explain", "Topic not found")
}
