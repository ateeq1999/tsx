use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainResult {
    pub topic: String,
    pub purpose: String,
    pub decisions: Vec<Decision>,
    pub tree: DecisionTree,
    pub learn_more: Vec<LearnMoreLink>,
    pub version: String,
    pub changelog: Vec<ChangelogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub title: String,
    pub rationale: String,
    pub alternative: Option<String>,
}

/// A visual decision tree rendered as structured data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTree {
    pub root: String,
    pub branches: Vec<TreeBranch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeBranch {
    pub condition: String,
    pub outcome: String,
    pub children: Vec<TreeBranch>,
}

/// A resolved learn-more link with label and URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnMoreLink {
    pub label: String,
    pub url: String,
}

/// A versioned changelog entry for a topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub version: String,
    pub date: String,
    pub change: String,
}

struct TopicEntry {
    key: &'static str,
    version: &'static str,
    purpose: &'static str,
    decisions: &'static [(&'static str, &'static str, Option<&'static str>)],
    tree: TreeDef,
    learn_more: &'static [(&'static str, &'static str)],
    changelog: &'static [(&'static str, &'static str, &'static str)],
}

struct TreeDef {
    root: &'static str,
    branches: &'static [(&'static str, &'static str)],
}

static KNOWLEDGE_BASE: &[TopicEntry] = &[
    TopicEntry {
        key: "atom",
        version: "1.0.0",
        purpose: "Atoms are the smallest reusable UI components that cannot be broken down further",
        decisions: &[
            ("Single file component", "Atoms should be self-contained in a single file for easy reuse", Some("Consider splitting if the component grows complex")),
            ("No internal state", "Atoms should be stateless - use molecules for components with state", Some("Use useState or useReducer in molecules instead")),
            ("Named exports", "Use named exports for better tree-shaking", Some("Default exports work but are less explicit")),
        ],
        tree: TreeDef {
            root: "Is this the smallest possible unit?",
            branches: &[
                ("Yes → no further decomposition needed", "This is an Atom — place in templates/atoms/"),
                ("No → contains multiple concerns", "Break it down: extract sub-pieces as Atoms, compose in a Molecule"),
                ("Has internal state?", "Promote to Molecule"),
                ("Includes other atoms?", "Promote to Molecule"),
            ],
        },
        learn_more: &[
            ("RustGen Atoms Architecture", "https://github.com/your-org/tsx/wiki/atoms"),
            ("MiniJinja include docs", "https://docs.rs/minijinja/latest/minijinja/"),
        ],
        changelog: &[
            ("1.0.0", "2026-03-16", "Initial atom tier definition with drizzle, zod, form, query atoms"),
        ],
    },
    TopicEntry {
        key: "molecule",
        version: "1.0.0",
        purpose: "Molecules are composite components that combine atoms with local state",
        decisions: &[
            ("Props drilling avoidance", "Molecules manage their own state to avoid prop drilling", Some("For deeply nested state, use context in organisms")),
            ("Composition over creation", "Molecules compose atoms rather than creating new UI", Some("Reuse existing atoms before creating new ones")),
        ],
        tree: TreeDef {
            root: "Does this compose multiple atoms?",
            branches: &[
                ("Yes + has local state", "This is a Molecule — place in templates/molecules/"),
                ("Yes + no state", "Consider making it a parameterised Atom"),
                ("No — single concern", "Keep as Atom"),
                ("Needs cross-feature state?", "Promote to Feature or use context"),
            ],
        },
        learn_more: &[
            ("MiniJinja macro docs", "https://docs.rs/minijinja/latest/minijinja/syntax/index.html"),
        ],
        changelog: &[
            ("1.0.0", "2026-03-16", "Initial molecules: drizzle table_body, zod schema_block, server_fn handler, query hooks, form component, data table, auth config"),
        ],
    },
    TopicEntry {
        key: "layout",
        version: "1.0.0",
        purpose: "Layouts define the structure and wrapping of routes using Jinja2 extends/block",
        decisions: &[
            ("File-based routing", "Layouts map directly to route file structure", Some("Use nested routes for complex hierarchies")),
            ("Import hoisting via render_imports()", "Layouts drain the ImportCollector as the first statement so imports appear at the top of every generated file", Some("Without hoisting, imports would be scattered where atoms render them")),
            ("Shared state via context", "Layouts can provide context to child routes", Some("Keep layout state minimal to avoid coupling")),
        ],
        tree: TreeDef {
            root: "What wraps this file?",
            branches: &[
                ("A React component file", "Use templates/layouts/component.jinja"),
                ("A TanStack Router route", "Use templates/layouts/route.jinja"),
                ("A raw TypeScript file", "Use templates/layouts/base.jinja"),
            ],
        },
        learn_more: &[
            ("Jinja2 template inheritance", "https://jinja.palletsprojects.com/en/3.1.x/templates/#template-inheritance"),
            ("ImportCollector design", "https://github.com/your-org/tsx/wiki/import-collector"),
        ],
        changelog: &[
            ("1.0.0", "2026-03-16", "Initial layouts: base, component, route — all with render_imports() drain"),
        ],
    },
    TopicEntry {
        key: "feature",
        version: "1.0.0",
        purpose: "Features are complete CRUD modules with routes, components, and server functions",
        decisions: &[
            ("Convention over configuration", "Features follow strict conventions for consistency", Some("Custom structures require more maintenance")),
            ("Server functions colocated", "Server functions live next to their route for co-location", Some("Extract to lib/ only if shared across features")),
            ("Barrel exports", "Features export everything from index for clean imports", Some("Barrels can hide internal structure - use with care")),
        ],
        tree: TreeDef {
            root: "What does this feature need?",
            branches: &[
                ("Full CRUD (list, create, update, delete)", "Use tsx generate feature — creates 7+ files"),
                ("Read-only list", "Use tsx generate feature with operations=[list]"),
                ("Single page, no data", "Use tsx generate page directly"),
                ("Needs auth guard?", "Pass auth=true to generate feature or add it with tsx add auth-guard"),
            ],
        },
        learn_more: &[
            ("tsx generate feature docs", "https://github.com/your-org/tsx/wiki/generate-feature"),
            ("TanStack Start file conventions", "https://tanstack.com/start/latest/docs/framework/react/guide/file-based-routing"),
        ],
        changelog: &[
            ("1.0.0", "2026-03-16", "Initial feature: schema + server_fn + query + table + form + 2 route pages"),
        ],
    },
    TopicEntry {
        key: "schema",
        version: "1.0.0",
        purpose: "Database schemas define table structures using Drizzle ORM with full TypeScript type inference",
        decisions: &[
            ("Type safety", "Schemas generate TypeScript types automatically via Drizzle inferSelect/inferInsert", Some("Manual types can drift from schema")),
            ("Idiomatic column order", "id (auto-increment), created_at, updated_at, then domain columns", Some("Consistent order helps readability and tooling")),
            ("SQLite by default", "TSX targets SQLite for zero-config local development", Some("PostgreSQL dialect is a drop-in swap in drizzle.config.ts")),
        ],
        tree: TreeDef {
            root: "What kind of data?",
            branches: &[
                ("Simple key-value entity", "Use string/number/boolean fields"),
                ("Relational (belongs to another table)", "Use id field type with references"),
                ("Needs soft delete?", "Pass softDelete=true — adds deletedAt column"),
                ("Needs timestamps?", "Pass timestamps=true (default) — adds createdAt/updatedAt"),
            ],
        },
        learn_more: &[
            ("Drizzle ORM docs", "https://orm.drizzle.team/docs/overview"),
            ("tsx generate schema", "https://github.com/your-org/tsx/wiki/generate-schema"),
        ],
        changelog: &[
            ("1.0.0", "2026-03-16", "Initial schema template: all 11 FieldTypes, timestamps, soft_delete, relations"),
        ],
    },
    TopicEntry {
        key: "server function",
        version: "1.0.0",
        purpose: "Server functions are type-safe RPC calls between TanStack Start client and server",
        decisions: &[
            ("Named exports", "Server functions must be named exports for tree-shaking", Some("Anonymous functions lose type information")),
            ("Input validation via Zod", "Always validate inputs to prevent injection and type errors", Some("Skip only for trusted internal callers")),
            ("createServerFn pattern", "Uses @tanstack/start createServerFn() for automatic serialisation", Some("Raw API routes lack type safety")),
        ],
        tree: TreeDef {
            root: "What operation?",
            branches: &[
                ("list — fetch many rows", "Returns array, no input required"),
                ("create — insert one row", "Takes validated input, returns created row"),
                ("update — modify existing", "Takes id + partial input, returns updated row"),
                ("delete — remove row", "Takes id, returns void"),
                ("Needs auth?", "Pass auth=true — wraps handler with session check"),
            ],
        },
        learn_more: &[
            ("TanStack Start server functions", "https://tanstack.com/start/latest/docs/framework/react/guide/server-functions"),
            ("tsx generate server-fn", "https://github.com/your-org/tsx/wiki/generate-server-fn"),
        ],
        changelog: &[
            ("1.0.0", "2026-03-16", "Initial server_fn template: all 4 CRUD operations with optional auth guard"),
        ],
    },
    TopicEntry {
        key: "query",
        version: "1.0.0",
        purpose: "TanStack Query hooks provide caching, background refetch, and Suspense-compatible data fetching",
        decisions: &[
            ("Query keys as arrays", "Use [resource, id] format for fine-grained cache invalidation", Some("String keys prevent partial invalidation")),
            ("useSuspenseQuery by default", "Suspense mode integrates cleanly with TanStack Router loaders", Some("useQuery with isLoading checks is more verbose")),
            ("Mutation returns invalidateQueries", "After a mutation, invalidate the relevant list query key", Some("Optimistic updates are faster but more complex")),
        ],
        tree: TreeDef {
            root: "Fetching or mutating?",
            branches: &[
                ("Fetching data (read)", "Use useSuspenseQuery with queryKey + queryFn"),
                ("Writing data (create/update/delete)", "Use useMutation + onSuccess → queryClient.invalidateQueries"),
                ("Needs pagination?", "Add page param to queryKey and queryFn"),
                ("Needs infinite scroll?", "Use useInfiniteQuery instead"),
            ],
        },
        learn_more: &[
            ("TanStack Query docs", "https://tanstack.com/query/latest/docs/framework/react/overview"),
            ("tsx generate query", "https://github.com/your-org/tsx/wiki/generate-query"),
        ],
        changelog: &[
            ("1.0.0", "2026-03-16", "Initial query template: useSuspenseQuery + useMutation with queryKey atoms"),
        ],
    },
    TopicEntry {
        key: "form",
        version: "1.0.0",
        purpose: "TanStack Form components provide type-safe, field-level validation with minimal re-renders",
        decisions: &[
            ("Controlled inputs", "Forms use controlled components for predictable state", Some("Uncontrolled (ref-based) has performance benefits but less control")),
            ("Field-level validation", "Validate each field independently for better UX", Some("Form-level validation is simpler but less responsive")),
            ("useForm with validators", "TanStack Form validators run on blur and submit", Some("Manual onChange validation is more verbose")),
        ],
        tree: TreeDef {
            root: "What input types?",
            branches: &[
                ("Text/email/password/url/number", "Renders field_input atom"),
                ("Boolean/toggle", "Renders field_switch atom"),
                ("Enum/select", "Renders field_select atom"),
                ("Date", "Renders field_datepicker atom"),
                ("Long text", "Renders field_textarea atom"),
            ],
        },
        learn_more: &[
            ("TanStack Form docs", "https://tanstack.com/form/latest/docs/overview"),
            ("tsx generate form", "https://github.com/your-org/tsx/wiki/generate-form"),
        ],
        changelog: &[
            ("1.0.0", "2026-03-16", "Initial form template: 5 field atoms dispatched by FieldType"),
        ],
    },
    TopicEntry {
        key: "auth",
        version: "1.0.0",
        purpose: "Authentication via Better Auth — a modern, TypeScript-first auth framework",
        decisions: &[
            ("Singleton pattern", "One auth instance exported from lib/auth.ts, reused everywhere", Some("Multiple instances can cause session conflicts")),
            ("Database sessions", "Sessions stored in SQLite via Drizzle adapter", Some("JWT sessions are stateless but harder to revoke")),
            ("Provider-based", "GitHub/Google OAuth configured in betterAuth({ socialProviders: {} })", Some("Email/password also supported via emailAndPassword plugin")),
        ],
        tree: TreeDef {
            root: "What auth do you need?",
            branches: &[
                ("Social login (GitHub/Google)", "Pass providers=['github','google'] to tsx add auth"),
                ("Email + password", "Pass providers=['email'] — enables emailAndPassword plugin"),
                ("Protect a route", "Use tsx add auth-guard --route /path"),
                ("Check session in server fn?", "Pass auth=true to tsx generate server-fn"),
            ],
        },
        learn_more: &[
            ("Better Auth docs", "https://www.better-auth.com/docs"),
            ("tsx add auth", "https://github.com/your-org/tsx/wiki/add-auth"),
        ],
        changelog: &[
            ("1.0.0", "2026-03-16", "Initial auth: betterAuth config with social providers and session fields"),
        ],
    },
];

fn fuzzy_score(needle: &str, haystack: &str) -> u32 {
    let n = needle.to_lowercase();
    let h = haystack.to_lowercase();
    if h == n { return 100; }
    if h.contains(&n) || n.contains(&h) { return 80; }
    let n_words: std::collections::HashSet<&str> = n.split_whitespace().collect();
    let h_words: std::collections::HashSet<&str> = h.split_whitespace().collect();
    let overlap = n_words.intersection(&h_words).count();
    if overlap > 0 { return 60 + (overlap.min(4) as u32 * 5); }
    0
}

pub fn explain(topic: String, verbose: bool) -> CommandResult {
    let start = Instant::now();

    let best = KNOWLEDGE_BASE
        .iter()
        .map(|e| (e, fuzzy_score(&topic, e.key)))
        .filter(|(_, score)| *score > 0)
        .max_by_key(|(_, score)| *score);

    if let Some((entry, _)) = best {
        let duration_ms = start.elapsed().as_millis() as u64;

        let tree = DecisionTree {
            root: entry.tree.root.to_string(),
            branches: entry
                .tree
                .branches
                .iter()
                .map(|(cond, outcome)| TreeBranch {
                    condition: cond.to_string(),
                    outcome: outcome.to_string(),
                    children: vec![],
                })
                .collect(),
        };

        let result = ExplainResult {
            topic: entry.key.to_string(),
            version: entry.version.to_string(),
            purpose: entry.purpose.to_string(),
            decisions: entry
                .decisions
                .iter()
                .map(|(title, rationale, alt)| Decision {
                    title: title.to_string(),
                    rationale: rationale.to_string(),
                    alternative: alt.map(|s| s.to_string()),
                })
                .collect(),
            tree,
            learn_more: entry
                .learn_more
                .iter()
                .map(|(label, url)| LearnMoreLink {
                    label: label.to_string(),
                    url: url.to_string(),
                })
                .collect(),
            changelog: entry
                .changelog
                .iter()
                .map(|(ver, date, change)| ChangelogEntry {
                    version: ver.to_string(),
                    date: date.to_string(),
                    change: change.to_string(),
                })
                .collect(),
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
    let available: Vec<String> = KNOWLEDGE_BASE.iter().map(|e| e.key.to_string()).collect();
    let error = crate::json::error::ErrorResponse::validation(&format!(
        "Topic '{}' not found. Available: {}",
        topic,
        available.join(", ")
    ));
    ResponseEnvelope::error("explain", error, duration_ms).print();
    CommandResult::err("explain", "Topic not found")
}
