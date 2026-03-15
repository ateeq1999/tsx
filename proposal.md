# TSX — Technical Proposal & Implementation

### TanStack Start Code Generation CLI — Rust Edition + Framework Protocol

**Technical Proposal & Architecture Design — Version 5.0 · March 2026**

> RustGen Atoms · TanStack Ecosystem · shadcn/ui · Better Auth · Drizzle ORM · Agent-Friendly JSON API · Framework Protocol

---

## Part 1: Core TSX CLI

### 1. Executive Summary

TSX is a command-line code generation tool written entirely in Rust, designed to dramatically reduce the token overhead of AI coding agents when building TanStack Start applications. Rather than asking an agent to write every file from scratch, the agent invokes concise CLI commands with JSON payloads and receives fully-wired, production-ready files on disk.

The core insight is simple: boilerplate is expensive in tokens and error-prone when generated on-the-fly. TSX encodes the team's architecture decisions — file structure, import conventions, type patterns, API contracts — into a layered Rust-native template system called **RustGen Atoms**, so agents spend tokens on business logic, not scaffolding.

Being implemented in Rust means TSX ships as a single statically-linked binary with no runtime dependency. Install once, run anywhere — no Node.js, no npm, no version conflicts.

**Goals at a Glance**

- Reduce agent token usage by 60–80% for common scaffolding tasks
- Enforce consistent patterns across all generated files via composable, versioned Atoms
- Keep every generated file immediately compilable with zero manual fixup
- Remain composable — commands can be chained in agent tool-call loops
- Ship as a single self-contained binary with zero runtime dependencies
- Enable the template system to evolve without rewriting command handlers
- Provide first-class JSON input/output for AI agent integration

---

### 2. Problem Statement

Modern AI coding agents are powerful but wasteful when it comes to repetitive file generation. Consider what happens today when an agent is asked to add a new resource — say, a `products` module:

- The agent generates a route file, guessing at the import paths
- It generates a server function, often forgetting to wire the query client
- It generates a Drizzle schema, sometimes mixing up column types
- It generates a TanStack Form component, re-deriving validation patterns
- It generates shadcn/ui bindings, duplicating component wiring seen elsewhere in the codebase

Each of these files can cost 300–800 tokens of generation and 200–400 tokens of verification. Multiply by 10 resources and the overhead compounds significantly. Worse, inconsistencies accumulate — the agent may use slightly different patterns across files, making future automated edits harder.

A secondary problem emerges as the project matures: **template drift**. When patterns evolve (a new auth pattern, an updated Drizzle API, a revised form hook), static template files must be updated one by one. With a flat template structure there is no single source of truth — the same Drizzle column pattern might be copy-pasted across templates with no mechanism to update them all.

TSX solves both problems. It replaces ad-hoc generation with deterministic instantiation, and it replaces static copy-paste templates with a composable **Atoms** layer — so a change to one atom propagates automatically to every template that includes it.

---

### 3. Technology Stack

#### 3.1 Target Application Stack

All generated applications are built on a fixed, opinionated stack. TSX does not attempt to be universal — it is a precision tool for this exact combination of technologies.

| Category | Technology | Role |
|---|---|---|
| Framework | TanStack Start | Full-stack React meta-framework |
| Routing | TanStack Router | Type-safe file-based routing |
| Data Fetching | TanStack Query | Server state & caching |
| Forms | TanStack Form | Type-safe form handling |
| Tables | TanStack Table | Headless data tables |
| UI Components | shadcn/ui | Accessible component library |
| Auth | Better Auth | Modern authentication framework |
| Database ORM | Drizzle ORM | Type-safe SQL query builder |
| Styling | Tailwind CSS | Utility-first CSS |

#### 3.2 CLI Implementation Stack (Rust)

| Crate | Role |
|---|---|
| `clap` (v4, derive feature) | Argument parsing and subcommand routing |
| `minijinja` | Template engine for the RustGen Atoms system |
| `serde` + `serde_json` | JSON payload deserialisation and structured stdout output |
| `anyhow` | Ergonomic error propagation across the render pipeline |
| `walkdir` | Project root auto-detection via `package.json` walk |
| `heck` | Case conversion helpers (`snake_case`, `PascalCase`, `camelCase`) |
| `prettyplease` | Post-render TypeScript/TSX formatting (via Prettier child process) |
| `reqwest` | HTTP client for npm registry loading |
| `tokio` | Async runtime for npm package fetching |

---

### 4. Architecture

#### 4.1 Repository Structure

TSX is a standalone Rust CLI crate. The `templates/` directory is organised around the Atoms hierarchy:

```
crates/tsx/
  src/
    commands/            # One module per top-level subcommand
      init.rs
      add_feature.rs
      add_schema.rs
      add_server_fn.rs
      add_query.rs
      add_form.rs
      add_table.rs
      add_page.rs
      add_auth.rs
      add_auth_guard.rs
      add_migration.rs
      add_seed.rs
      list.rs            # Introspection commands
      inspect.rs         # Project inspection
      batch.rs           # Batch operations
      ask.rs             # Framework Q&A
      where_cmd.rs       # File location queries
      how.rs             # Integration how-tos
      explain.rs         # Learning mode
    framework/           # Framework Protocol module
      mod.rs
      registry.rs        # Framework registry types
      loader.rs          # Framework loader
    schemas/             # Serde structs for JSON payload validation
    render/
      engine.rs          # MiniJinja environment bootstrap + atom loading
      context.rs         # Typed render context builders
    utils/
      paths.rs           # Project root detection + output path resolution
      write.rs           # Atomic file writes with --overwrite guard
      imports.rs         # Import deduplication and injection utilities
      barrel.rs          # Barrel file (index.ts) auto-update
      format.rs          # Prettier integration
    json/                # JSON input/output handling
      payload.rs         # Command payload structures
      response.rs        # Response envelope
      error.rs           # Error types and codes
    output.rs            # JSON result contract serialisation
    main.rs              # Entry point: clap app definition
  templates/
    atoms/               # Tier 1 — indivisible code fragments (.jinja)
    molecules/           # Tier 2 — atoms composed into logical blocks
    layouts/             # Tier 3 — layout macros (file outer shells)
    features/            # Tier 4 — feature templates (one per output file type)
    metadata.json        # Template metadata for introspection
  frameworks/            # Built-in framework definitions
    tanstack-start/
    drizzle-orm/
    better-auth/
    react/
    nextjs/
    prisma/
    clerk/
    vue/
    svelte/
    authjs/
  Cargo.toml
```

#### 4.2 Template Engine — MiniJinja & the RustGen Atoms System

All code generation is powered by MiniJinja (`.jinja` template files). The four Jinja2 primitives map directly onto the four Atoms tiers:

| Jinja2 Primitive | Atoms Tier | Responsibility |
|---|---|---|
| `{% include %}` | **Atoms** | Inject an indivisible code fragment |
| `{% macro %}` / `{{ caller() }}` | **Molecules** | Compose atoms into a typed, reusable block |
| `{% extends %}` + `{% block %}` | **Layouts** | Wrap rendered molecules in a file-level shell |
| `{% set imports %}` namespace + `{{ render_imports() }}` | **Import hoisting** | Collect imports from deep inside atoms, emit them at the file top |

A complete generated file is assembled as: **Layout → Molecule(s) → Atoms → collected imports resolved at top**. Import hoisting is implemented via a thread-local `ImportCollector` that atoms push into during render; the layout drains it as its first output statement.

#### 4.3 Command Execution Pipeline

Every CLI invocation follows the same deterministic pipeline:

1. **Parse** — `clap` parses the subcommand and JSON string argument (or from stdin/file)
2. **Deserialise** — `serde_json` deserialises the JSON into the command's typed payload struct; validation errors are surfaced with field-level messages
3. **Resolve** — output paths are resolved relative to the project root (auto-detected via `package.json` walk using `walkdir`)
4. **Render** — MiniJinja renders the feature template; the Atoms framework assembles atoms → molecules → layout internally; imports are collected and hoisted
5. **Format** — rendered output is piped through a Prettier child process to match project style
6. **Write** — files are written atomically; existing files require an explicit `--overwrite` flag
7. **Wire** — import injections and barrel file updates are applied where needed
8. **Report** — a JSON result summary is serialised and printed to stdout for the agent to parse

---

### 5. Agent-Friendly JSON Interface

TSX is designed from the ground up for AI agent integration. All features below ensure agents can reliably control TSX programmatically.

#### 5.1 JSON Input Mode

The CLI accepts JSON input via three mechanisms:

| Flag | Description |
|------|-------------|
| `--json <payload>` | Parse remaining arguments as JSON |
| `--stdin` | Read the entire command payload from stdin |
| `--file <path>` | Read command payload from a file |

#### 5.2 Structured JSON Output

All output is returned as JSON with a consistent envelope:

```json
{
  "success": true,
  "version": "1.0",
  "command": "add:feature",
  "result": {
    "files_created": ["db/schema/products.ts", "server-functions/products.ts", "queries/products.ts", "routes/products/index.tsx", "routes/products/$id.tsx", "components/products/products-table.tsx", "components/products/product-form.tsx", "components/products/delete-dialog.tsx"]
  },
  "metadata": {
    "timestamp": "2026-03-15T10:30:00Z",
    "duration_ms": 45
  },
  "next_steps": ["Run: tsx add:migration {}"]
}
```

#### 5.3 Structured Error Format

All errors follow a consistent structure with error codes:

| Code | Description |
|------|-------------|
| `INVALID_PAYLOAD` | JSON payload is malformed |
| `VALIDATION_ERROR` | Payload fails schema validation |
| `UNKNOWN_COMMAND` | Command is not recognized |
| `UNKNOWN_KIND` | Generator or list kind is not recognized |
| `FILE_EXISTS` | Target file already exists (use `--overwrite`) |
| `DIRECTORY_NOT_FOUND` | Required parent directory does not exist |
| `PERMISSION_DENIED` | Cannot write to target location |
| `TEMPLATE_NOT_FOUND` | Specified template does not exist |
| `PROJECT_NOT_FOUND` | Not running inside a TanStack Start project |
| `INTERNAL_ERROR` | Unexpected error in CLI |

#### 5.4 Introspection Commands

- `tsx list templates` — List available project templates
- `tsx list generators` — List all CLI commands with option schemas
- `tsx list components` — List available shadcn/ui components
- `tsx list frameworks` — List registered frameworks (Framework Protocol)

#### 5.5 Project Inspection

`tsx inspect` returns project structure, database provider, auth config, etc.

#### 5.6 Batch Operations

`tsx batch` executes multiple commands in sequence with aggregated results.

#### 5.7 Dry-Run Mode

`tsx --dry-run` previews changes without writing files.

---

### 6. Command Reference

| Command | Description |
|---|---|
| `tsx init` | Bootstrap a new TanStack Start project |
| `tsx add:feature` | Scaffold complete CRUD feature module |
| `tsx add:schema` | Generate Drizzle schema table |
| `tsx add:server-fn` | Generate typed server function |
| `tsx add:query` | Generate TanStack Query hook |
| `tsx add:form` | Generate TanStack Form component |
| `tsx add:table` | Generate TanStack Table component |
| `tsx add:page` | Add new route page |
| `tsx add:auth` | Configure Better Auth |
| `tsx add:auth-guard` | Add auth guard to route |
| `tsx add:migration` | Run drizzle-kit generate + migrate |
| `tsx add:seed` | Generate seed file |
| `tsx list` | List templates, generators, components, frameworks |
| `tsx inspect` | Query project state |
| `tsx batch` | Execute multiple commands |
| `tsx ask` | Ask framework questions (Framework Protocol) |
| `tsx where` | Query file locations (Framework Protocol) |
| `tsx how` | Get integration steps (Framework Protocol) |
| `tsx explain` | Explain template decisions (Framework Protocol) |

---

### 7. RustGen Atoms — Template Architecture

The **RustGen Atoms** framework defines how code generation knowledge is stored, composed, and evolved.

#### 7.1 Four-Tier System

```
Tier 1 — Atoms       Indivisible fragments. Cannot include other atoms.
Tier 2 — Molecules   Atoms composed into a logical, self-contained block.
Tier 3 — Layouts     Jinja2 base templates that give a file its outer structure.
Tier 4 — Features    Feature templates that wire molecules into layouts.
```

#### 7.2 ImportCollector — The Rust Side

```rust
thread_local! {
    static IMPORT_COLLECTOR: RefCell<BTreeSet<String>> = RefCell::new(BTreeSet::new());
    static PRIORITY_IMPORTS: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

pub fn collect_import(value: String) -> String {
    IMPORT_COLLECTOR.with(|c| c.borrow_mut().insert(value));
    String::new()
}

pub fn render_imports() -> String {
    // Drains collector, deduplicates, returns sorted imports
}
```

---

## Part 2: Framework Protocol

### 8. Executive Summary

TSX evolves from a TanStack Start-specific code generator into a **universal framework bootstrapping protocol**. Framework developers (React, Vue, Svelte, Solid, etc.) and package authors can register their frameworks with TSX to provide AI agents with:

1. **Where** — Canonical file locations and project structure
2. **What** — Code templates for integration patterns
3. **How** — Injection points for user custom code
4. **Dependencies** — Required packages and configurations

AI agents use TSX as a **conversation partner** to learn any framework, not just generate code — but also understand conventions, patterns, and best practices.

### 9. Core Concepts

#### 9.1 Framework Registry

Each framework registers via a `registry.json` file:

```json
{
  "framework": "TanStack Start",
  "version": "1.0",
  "slug": "tanstack-start",
  "category": "framework",
  "docs": "https://tanstack.com/start",
  "structure": { "routes": "routes/", "components": "components/" },
  "generators": [...],
  "conventions": {...},
  "injection_points": [...],
  "integrations": [...],
  "questions": [...]
}
```

#### 9.2 Convention Protocol

Frameworks define file structure and naming conventions that agents can query.

#### 9.3 Injection Points

Templates define where developers can add custom code, preserved during regeneration.

### 10. Framework Protocol Commands

#### 10.1 Framework Discovery

```bash
tsx list --frameworks
```

Lists all registered frameworks with name, version, category, docs URL.

#### 10.2 Ask Command

```bash
tsx ask --question "How do I add authentication?" --framework tanstack-start
```

Returns answer with steps, files affected, and dependencies.

#### 10.3 Where Command

```bash
tsx where --thing atom --framework tanstack-start
```

Returns canonical file path, pattern, and conventions.

#### 10.4 How Command

```bash
tsx how --integration @tanstack/react-router --framework tanstack-start
```

Returns install command, setup steps, and patterns.

#### 10.5 Explain Command

```bash
tsx explain --topic atom
```

Returns purpose, design decisions, and rationale.

### 11. Built-in Frameworks

TSX ships with 10 framework definitions:

| Framework | Category |
|---|---|
| TanStack Start | Framework |
| React | Framework |
| Next.js | Framework |
| Vue | Framework |
| Svelte | Framework |
| Drizzle ORM | ORM |
| Prisma | ORM |
| Better Auth | Auth |
| Clerk | Auth |
| Auth.js | Auth |

---

## Part 3: Implementation Status

### 12. Completed Implementation

#### Core TSX CLI (131/131 tasks complete)

- Phase 1: Foundation (Cargo, modules, CLI skeleton, JSON I/O, schemas, output, paths, writer, engine)
- Phase 2: Atoms, Molecules, Layouts (all template tiers + tests)
- Phase 3: Agent-Friendly JSON API (errors, list, inspect, batch, dry-run, verbose)
- Phase 4: Command Handlers (all 12 commands + utilities)
- Phase 5: Hardening (Prettier, embedding, e2e, flags, release)

#### Framework Protocol (42/42 tasks complete)

- Phase 1: Foundation (framework module, registry types, loader, discover_frameworks)
- Phase 2: Query Interface (ask, where, how commands)
- Phase 3: Learning Mode (explain command, decision knowledge base)
- Phase 4: Ecosystem (publishing design, 10 framework definitions)

---

## Part 4: Future Enhancements

### 13. Dev Server JSON Events

For `dev` mode, the CLI can emit file change events as JSON:

```json
{
  "event": "file_changed",
  "timestamp": "2026-03-15T10:30:00Z",
  "data": {
    "kind": "modified",
    "path": "routes/dashboard.tsx",
    "action": "regenerated"
  }
}
```

**Event Types**:

| Event | Description |
|-------|-------------|
| `started` | Dev server has started |
| `file_changed` | A file was modified |
| `file_added` | A new file was created |
| `file_deleted` | A file was removed |
| `build_started` | Build process started |
| `build_completed` | Build completed successfully |
| `build_failed` | Build failed with errors |
| `hot_reload` | Hot reload triggered |
| `error` | Server encountered an error |
| `stopped` | Dev server stopped |

**Invocation**:

```bash
tsx dev --json-events
```

### 14. Suggested New Features

The following features are proposed for future development:

### 14. Suggested New Features

#### 14.1 Template Versioning

- [ ] Implement atom version pinning per project
- [ ] Add `tsx upgrade` command to update atom versions
- [ ] Support breaking change detection

#### 14.2 Custom Template Plugins

- [ ] Add `--plugin` flag to load custom template packages from npm
- [ ] Support template overrides for specific generators
- [ ] Implement plugin sandboxing for security

#### 14.3 WebSocket Dev Server Events

- [ ] Add `--watch` mode for file regeneration on template changes
- [ ] Implement WebSocket server for real-time events
- [ ] Support hot module replacement integration

#### 14.4 Enhanced Learning Mode

- [ ] Add semantic search for question matching
- [ ] Implement learn-more URL resolution from frameworks
- [ ] Add decision explanation versioning

#### 14.5 Registry Publishing

- [ ] Build `tsx publish` command for sharing custom registries
- [ ] Implement registry validation and testing
- [ ] Create framework registry website

#### 14.6 Additional Framework Support

- [ ] Add Solid.js framework registry
- [ ] Add Kysely ORM registry
- [ ] Add Tailwind CSS integration patterns
- [ ] Add state management patterns (Zustand, Jotai)

---

## Part 5: Conclusion

TSX represents a shift in how we think about AI-assisted development. Rather than training agents to write better boilerplate, we eliminate boilerplate from the agent's responsibility entirely. The agent becomes a domain expert — it decides *what* to build — while TSX handles *how* to build it correctly and consistently.

The Framework Protocol transforms TSX from a code generator into an **AI agent development partner**. Framework developers no longer need to write agent prompts — they register their conventions once, and all AI agents instantly understand how to build with their framework.

**Key insight: code generation is the side effect, learning is the product.**

---

*Last Updated: March 2026*
*Version: 5.0*
*Status: Core CLI Complete (131/131) • Framework Protocol Complete (42/42)*
