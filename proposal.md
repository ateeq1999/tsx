# TSX

### TanStack Start Code Generation CLI — Rust Edition

**Technical Proposal & Architecture Design — Version 4.0 · March 2026**

> RustGen Atoms · TanStack Ecosystem · shadcn/ui · Better Auth · Drizzle ORM · Agent-Friendly JSON API

---

## 1. Executive Summary

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

## 2. Problem Statement

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

## 3. Technology Stack

### 3.1 Target Application Stack

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

### 3.2 CLI Implementation Stack (Rust)

| Crate | Role |
|---|---|
| `clap` (v4, derive feature) | Argument parsing and subcommand routing |
| `minijinja` | Template engine for the RustGen Atoms system (see §4.2) |
| `serde` + `serde_json` | JSON payload deserialisation and structured stdout output |
| `anyhow` | Ergonomic error propagation across the render pipeline |
| `walkdir` | Project root auto-detection via `package.json` walk |
| `heck` | Case conversion helpers (`snake_case`, `PascalCase`, `camelCase`) |
| `prettyplease` | Post-render TypeScript/TSX formatting (via Prettier child process) |

---

## 4. Architecture

### 4.1 Repository Structure

TSX is a standalone Rust CLI crate. The `templates/` directory is organised around the Atoms hierarchy described in section 7:

```
crates/
  tsx/
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
      schemas/             # Serde structs for JSON payload validation
        feature.rs
        schema.rs
        server_fn.rs
        query.rs
        form.rs
        field.rs           # Shared FieldSchema type
      render/
        engine.rs          # MiniJinja environment bootstrap + atom loading
        filters.rs         # Custom Jinja filters: snake_case, pascal_case, etc.
        context.rs         # Typed render context builders
      utils/
        paths.rs           # Project root detection + output path resolution
        write.rs           # Atomic file writes with --overwrite guard
        imports.rs         # Import deduplication and injection utilities
        barrel.rs          # Barrel file (index.ts) auto-update
      json/                # JSON input/output handling
        payload.rs         # Command payload structures
        response.rs        # Response envelope
        error.rs           # Error types and codes
        events.rs          # Dev server events
      output.rs            # JSON result contract serialisation
      main.rs              # Entry point: clap app definition
    templates/
      atoms/               # Tier 1 — indivisible code fragments (.jinja)
        drizzle/           #   Column, relation, index atoms
        zod/               #   Field rule, object wrapper atoms
        query/             #   queryKey, queryFn, mutation atoms
        form/              #   Field, submit, validation atoms
        imports/           #   Named import-line atoms
      molecules/           # Tier 2 — atoms composed into logical blocks
        drizzle/           #   Full table definition molecule
        zod/               #   Complete schema molecule
        form/              #   Full form component molecule
        table/             #   Full table component molecule
        server_fn/         #   Complete server function molecule
        query/             #   Query hooks block molecule
        auth/              #   Auth config molecule
      layouts/             # Tier 3 — layout macros (file outer shells)
        base.jinja
        component.jinja
        route.jinja
      features/            # Tier 4 — feature templates (one per output file type)
        schema.jinja
        server_fn.jinja
        query.jinja
        form.jinja
        table.jinja
        page.jinja
        seed.jinja
        auth_config.jinja
      metadata.json        # Template metadata for introspection
    Cargo.toml
```

### 4.2 Template Engine — MiniJinja & the RustGen Atoms System

> **Why not Ramhorns?**
>
> Ramhorns is a fast Mustache implementation, but Mustache's feature set is intentionally minimal: it provides variable substitution, sections, and partials — nothing more. The RustGen Atoms system requires named layouts, composable block/slot regions, macro-style includes with typed arguments, and a mechanism equivalent to import-stack hoisting. Mustache has none of these. Ramhorns would require implementing all of this logic in Rust outside the template layer, collapsing the separation between renderer and template author.
>
> **MiniJinja** is the correct choice. It is a Rust-native implementation of the Jinja2 template language with full support for template inheritance (`{% extends %}`), named blocks (`{% block %}`), macros (`{% macro %}`), includes (`{% include %}`), and custom filters. Its Jinja2 semantics map directly and deliberately onto the four Atoms tiers — the same structural thinking that made EdgeJS the right choice in the Node.js version maps cleanly onto Jinja2's feature set in Rust.

All code generation is powered by MiniJinja (`.jinja` template files). The four Jinja2 primitives map directly onto the four Atoms tiers:

| Jinja2 Primitive | Atoms Tier | Responsibility |
|---|---|---|
| `{% include %}` | **Atoms** | Inject an indivisible code fragment |
| `{% macro %}` / `{{ caller() }}` | **Molecules** | Compose atoms into a typed, reusable block |
| `{% extends %}` + `{% block %}` | **Layouts** | Wrap rendered molecules in a file-level shell |
| `{% set imports %}` namespace + `{{ render_imports() }}` | **Import hoisting** | Collect imports from deep inside atoms, emit them at the file top |

A complete generated file is assembled as: **Layout → Molecule(s) → Atoms → collected imports resolved at top**. Import hoisting is implemented via a thread-local `ImportCollector` that atoms push into during render; the layout drains it as its first output statement.

### 4.3 Command Execution Pipeline

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

## 5. Agent-Friendly JSON Interface

TSX is designed from the ground up for AI agent integration. All features below ensure agents can reliably control TSX programmatically.

### 5.1 JSON Input Mode

The CLI accepts JSON input via three mechanisms:

| Flag | Description |
|------|-------------|
| `--json <payload>` | Parse remaining arguments as JSON |
| `--stdin` | Read the entire command payload from stdin |
| `--file <path>` | Read command payload from a file |

#### Command Payload Structure

```json
{
  "version": "1.0",
  "command": "add:feature",
  "options": {
    "name": "products",
    "fields": [
      { "name": "title", "type": "string", "required": true },
      { "name": "price", "type": "number", "required": true }
    ],
    "auth": true,
    "paginated": true,
    "operations": ["list", "create", "update", "delete"]
  }
}
```

#### Example Invocation

```bash
# Via stdin
echo '{"command": "add:feature", "options": {"name": "products"}}' | tsx --stdin

# Via file
tsx --file generate-feature.json

# Via flag (single command)
tsx --json '{"command": "add:schema", "options": {"name": "categories"}}'
```

### 5.2 Structured JSON Output

All output is returned as JSON with a consistent envelope:

```json
{
  "success": true,
  "version": "1.0",
  "command": "add:feature",
  "result": {
    "files_created": [
      "db/schema/products.ts",
      "server-functions/products.ts",
      "queries/products.ts",
      "routes/products/index.tsx",
      "routes/products/$id.tsx",
      "components/products/products-table.tsx",
      "components/products/product-form.tsx",
      "components/products/delete-dialog.tsx"
    ]
  },
  "metadata": {
    "timestamp": "2026-03-15T10:30:00Z",
    "duration_ms": 45
  },
  "next_steps": ["Run: tsx add:migration {}"]
}
```

#### Verbose Mode (`--verbose`)

Adds additional context to the response:

```json
{
  "success": true,
  "version": "1.0",
  "command": "add:feature",
  "result": {
    "files_created": ["db/schema/products.ts", "server-functions/products.ts"]
  },
  "metadata": {
    "timestamp": "2026-03-15T10:30:00Z",
    "duration_ms": 45,
    "warnings": ["Consider adding an index on categoryId for better query performance"]
  },
  "context": {
    "project_root": "/path/to/project",
    "tsx_version": "1.0.0"
  },
  "next_steps": ["Run: tsx add:migration {}"]
}
```

### 5.3 Structured Error Format

All errors follow a consistent structure:

```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid command payload",
    "details": [
      {
        "field": "options.name",
        "message": "Name must match pattern ^[a-z0-9-]+$"
      }
    ]
  }
}
```

#### Error Codes

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

### 5.4 Introspection Commands

Agents can discover available capabilities before using them.

#### List Templates

```json
{
  "command": "list",
  "options": {
    "kind": "templates"
  }
}
```

**Response**:

```json
{
  "success": true,
  "result": {
    "templates": [
      {
        "id": "default",
        "name": "Full Stack",
        "description": "Complete TanStack Start app with auth, DB, and routing",
        "path": "templates/default",
        "files": ["src/main.tsx", "routes/", "components/", "lib/"]
      },
      {
        "id": "minimal",
        "name": "Minimal",
        "description": "Minimal boilerplate to get started",
        "path": "templates/minimal",
        "files": ["src/main.tsx", "routes/index.tsx"]
      }
    ]
  }
}
```

#### List Generators

```json
{
  "command": "list",
  "options": {
    "kind": "generators"
  }
}
```

**Response**:

```json
{
  "success": true,
  "result": {
    "generators": [
      {
        "id": "add:feature",
        "description": "Scaffold a complete CRUD feature module",
        "options": {
          "name": { "type": "string", "required": true, "pattern": "^[a-z0-9-]+$" },
          "fields": { "type": "array", "required": true },
          "auth": { "type": "boolean", "default": false },
          "paginated": { "type": "boolean", "default": true },
          "operations": { "type": "array", "default": ["list", "create", "update", "delete"] }
        }
      },
      {
        "id": "add:schema",
        "description": "Generate a Drizzle schema table definition",
        "options": {
          "name": { "type": "string", "required": true },
          "fields": { "type": "array", "required": true },
          "timestamps": { "type": "boolean", "default": true },
          "softDelete": { "type": "boolean", "default": false }
        }
      },
      {
        "id": "add:server-fn",
        "description": "Generate a typed server function",
        "options": {
          "name": { "type": "string", "required": true },
          "table": { "type": "string", "required": true },
          "operation": { "type": "string", "enum": ["list", "create", "update", "delete"], "required": true }
        }
      },
      {
        "id": "add:query",
        "description": "Generate a TanStack Query hook",
        "options": {
          "name": { "type": "string", "required": true },
          "serverFn": { "type": "string", "required": true }
        }
      },
      {
        "id": "add:form",
        "description": "Generate a TanStack Form component",
        "options": {
          "name": { "type": "string", "required": true },
          "fields": { "type": "array", "required": true }
        }
      },
      {
        "id": "add:table",
        "description": "Generate a TanStack Table component",
        "options": {
          "name": { "type": "string", "required": true },
          "columns": { "type": "array", "required": true }
        }
      },
      {
        "id": "add:page",
        "description": "Add a new route page",
        "options": {
          "path": { "type": "string", "required": true },
          "title": { "type": "string" }
        }
      },
      {
        "id": "add:auth",
        "description": "Configure Better Auth",
        "options": {
          "providers": { "type": "array", "default": ["github", "google"] }
        }
      }
    ]
  }
}
```

### 5.5 Project Inspection

Agents can query the current project state:

```json
{
  "command": "inspect"
}
```

**Response**:

```json
{
  "success": true,
  "result": {
    "project_root": "/path/to/my-app",
    "tsx_version": "1.0.0",
    "app_name": "My App",
    "structure": {
      "schemas": ["products", "categories", "users"],
      "server_functions": ["products/list", "products/create", "users/get"],
      "queries": ["useProducts", "useCategories"],
      "forms": ["product-form", "login-form"],
      "tables": ["products-table", "users-table"],
      "routes": ["index", "products", "products/$id", "login"]
    },
    "database": {
      "provider": "sqlite",
      "url": "sqlite://app.db",
      "migrations_pending": 2
    },
    "auth": {
      "configured": true,
      "providers": ["github", "google"]
    },
    "config": {
      "tsconfig_path": "tsconfig.json",
      "shadcn_path": "components/ui"
    }
  }
}
```

### 5.6 Batch Operations

Execute multiple commands in a single invocation:

```json
{
  "command": "batch",
  "options": {
    "commands": [
      {"command": "add:schema", "options": {"name": "products", "fields": [{"name": "title", "type": "string", "required": true}]}},
      {"command": "add:server-fn", "options": {"name": "list-products", "table": "products", "operation": "list"}},
      {"command": "add:query", "options": {"name": "products", "serverFn": "list-products"}},
      {"command": "add:page", "options": {"path": "products", "title": "Products"}}
    ]
  }
}
```

**Response**:

```json
{
  "success": true,
  "result": {
    "total": 4,
    "succeeded": 4,
    "failed": 0,
    "results": [
      {
        "index": 0,
        "success": true,
        "result": {"kind": "schema", "path": "db/schema/products.ts"}
      },
      {
        "index": 1,
        "success": true,
        "result": {"kind": "server_fn", "path": "server-functions/products.ts"}
      },
      {
        "index": 2,
        "success": true,
        "result": {"kind": "query", "path": "queries/products.ts"}
      },
      {
        "index": 3,
        "success": true,
        "result": {"kind": "page", "path": "routes/products.tsx"}
      }
    ]
  }
}
```

**Failure Handling**:

```json
{
  "success": false,
  "result": {
    "total": 3,
    "succeeded": 2,
    "failed": 1,
    "results": [
      {
        "index": 0,
        "success": true,
        "result": {"kind": "schema", "path": "db/schema/products.ts"}
      },
      {
        "index": 1,
        "success": false,
        "error": {
          "code": "FILE_EXISTS",
          "message": "Schema 'products' already exists at db/schema/products.ts",
          "path": "db/schema/products.ts"
        }
      },
      {
        "index": 2,
        "success": true,
        "result": {"kind": "query", "path": "queries/products.ts"}
      }
    ]
  }
}
```

### 5.7 Dry-Run Mode

Agents can preview changes without modifying the filesystem:

```json
{
  "command": "add:feature",
  "options": {
    "name": "products",
    "fields": [
      { "name": "title", "type": "string", "required": true }
    ],
    "dryRun": true
  }
}
```

**Response**:

```json
{
  "success": true,
  "dryRun": true,
  "result": {
    "wouldCreate": 8,
    "files": [
      {"path": "db/schema/products.ts", "overwrites": false},
      {"path": "server-functions/products.ts", "overwrites": false},
      {"path": "queries/products.ts", "overwrites": false},
      {"path": "routes/products/index.tsx", "overwrites": false},
      {"path": "routes/products/$id.tsx", "overwrites": false},
      {"path": "components/products/products-table.tsx", "overwrites": false},
      {"path": "components/products/product-form.tsx", "overwrites": false},
      {"path": "components/products/delete-dialog.tsx", "overwrites": false}
    ]
  }
}
```

---

## 6. Command Reference

All commands accept a `--json '<payload>'` flag. This makes them trivially callable from any agent tool-call interface without shell escaping complexity.

### 6.1 Project & Feature Scaffolding

| Command | Description | Key Payload Fields |
|---|---|---|
| `tsx init` | Bootstrap a new TanStack Start project with all stack dependencies wired | `name, description, dbProvider` |
| `tsx add:feature` | Scaffold a complete CRUD feature module (schema + server fns + queries + route + table + form) | `name, fields[], auth, paginated` |
| `tsx add:page` | Add a new route file with layout slot and loader | `path, title, auth, loader` |

### 6.2 Data Layer

| Command | Description | Key Payload Fields |
|---|---|---|
| `tsx add:schema` | Generate a Drizzle schema table definition | `name, fields[], timestamps, softDelete` |
| `tsx add:migration` | Run drizzle-kit generate + migrate in sequence | — |
| `tsx add:seed` | Generate a seed file for a given schema | `name, count` |

### 6.3 Server Functions & Queries

| Command | Description | Key Payload Fields |
|---|---|---|
| `tsx add:server-fn` | Generate a typed server function | `name, table, operation, auth, input` |
| `tsx add:query` | Generate a TanStack Query hook wrapping a server function | `name, serverFn, suspense, mutation` |
| `tsx add:loader` | Generate a route loader that prefetches a query | `routePath, queryKey` |

### 6.4 UI Components

| Command | Description | Key Payload Fields |
|---|---|---|
| `tsx add:form` | Generate a TanStack Form with shadcn/ui fields + Zod schema | `name, fields[], submitFn, layout` |
| `tsx add:table` | Generate a TanStack Table with shadcn/ui DataTable wrapper | `name, columns[], queryHook, actions` |
| `tsx add:dialog` | Generate a shadcn Dialog wrapping a form | `name, formName, trigger` |
| `tsx add:component` | Generate a plain shadcn-wired component | `name, props[], variant` |

### 6.5 Auth

| Command | Description | Key Payload Fields |
|---|---|---|
| `tsx add:auth` | Install and configure Better Auth with chosen providers | `providers[], sessionFields[], emailVerification` |
| `tsx add:auth-guard` | Wrap a route or layout with a session guard | `routePath, redirectTo` |

### 6.6 Introspection & Inspection

| Command | Description | Key Payload Fields |
|---|---|---|
| `tsx list` | List available templates, generators, or components | `kind: "templates" | "generators" | "components"` |
| `tsx inspect` | Query current project state | — |

### 6.7 Batch Operations

| Command | Description | Key Payload Fields |
|---|---|---|
| `tsx batch` | Execute multiple commands in one invocation | `commands: Command[]` |

---

## 7. JSON Payload Design

Payloads are intentionally minimal. Every field has a sensible default and the Serde schema provides clear error messages when required fields are missing. The agent never needs to know file paths, import strings, or naming conventions — the CLI derives all of that.

### 7.1 Payload Structs (Rust)

```rust
// schemas/feature.rs
#[derive(Debug, Deserialize)]
pub struct AddFeatureArgs {
    pub name: String,
    pub fields: Vec<FieldSchema>,
    #[serde(default)]
    pub auth: bool,
    #[serde(default)]
    pub paginated: bool,
    #[serde(default = "default_operations")]
    pub operations: Vec<Operation>,
}

// schemas/field.rs
#[derive(Debug, Deserialize, Clone)]
pub struct FieldSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(default = "bool_true")]
    pub required: bool,
    pub unique: Option<bool>,
    pub references: Option<String>,
    pub values: Option<Vec<String>>, // for enum fields
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    String, Number, Boolean, Date, Id, Enum, Json, Decimal, Email, Url, Password,
}
```

### 7.2 Example — `add:feature`

```bash
tsx add:feature --json '{
  "name": "products",
  "fields": [
    { "name": "title",       "type": "string",  "required": true },
    { "name": "price",       "type": "number",  "required": true },
    { "name": "description", "type": "string",  "required": false },
    { "name": "categoryId",  "type": "id",      "references": "categories" }
  ],
  "auth": true,
  "paginated": true,
  "operations": ["list", "create", "update", "delete"]
}'
```

Files generated from this one command:

| File | Contents |
|---|---|
| `db/schema/products.ts` | Drizzle table with all columns, timestamps, relations |
| `server-functions/products.ts` | CRUD server functions, all auth-guarded |
| `queries/products.ts` | TanStack Query hooks for each operation |
| `routes/products/index.tsx` | List page with table, pagination, create button |
| `routes/products/$id.tsx` | Detail/edit page with form |
| `components/products/products-table.tsx` | TanStack Table with actions column |
| `components/products/product-form.tsx` | TanStack Form with Zod validation |
| `components/products/delete-dialog.tsx` | Confirm delete shadcn Dialog |

---

## 8. RustGen Atoms — Template Architecture

### 8.1 Overview

The **RustGen Atoms** framework is the template layer at the heart of TSX. It defines how code generation knowledge is stored, composed, and evolved over time.

The system has four tiers:

```
Tier 1 — Atoms       Indivisible fragments. Cannot include other atoms.
Tier 2 — Molecules   Atoms composed into a logical, self-contained block.
Tier 3 — Layouts     Jinja2 base templates that give a file its outer structure.
Tier 4 — Features    Feature templates that wire molecules into layouts.
```

Each tier is implemented using a distinct MiniJinja mechanism, making the mapping between concept and code unambiguous.

### 8.2 Tier 1 — Atoms

An atom is a single `.jinja` partial that renders **one indivisible code fragment**. It accepts a small, well-typed context object and produces a deterministic string. Atoms never `{% include %}` other atoms — they are the leaf nodes of the composition tree.

Atoms never emit imports directly into the output stream. Instead, they call a `collect_import(ns, statement)` custom filter that pushes import lines into a thread-local `ImportCollector`. The layout drains the collector at render time, producing a correctly ordered, deduplicated import block at the top of the file.

**Atom catalogue:**

```
atoms/drizzle/column.jinja            { field: FieldSchema }
atoms/drizzle/timestamp_cols.jinja    { }
atoms/drizzle/soft_delete_col.jinja   { }
atoms/drizzle/relation.jinja          { field: FieldSchema, table_name: string }
atoms/zod/field_rule.jinja            { field: FieldSchema }
atoms/zod/object_wrapper.jinja         { name: string }
atoms/form/field_input.jinja          { field: FieldSchema, form_name: string }
atoms/form/field_select.jinja         { field: FieldSchema, options: string }
atoms/form/field_switch.jinja         { field: FieldSchema, form_name: string }
atoms/form/field_datepicker.jinja     { field: FieldSchema, form_name: string }
atoms/form/field_textarea.jinja       { field: FieldSchema, form_name: string }
atoms/query/query_key.jinja           { name: string, params: string[] }
atoms/query/suspense_query.jinja      { name: string, server_fn: string }
atoms/query/mutation.jinja            { name: string, server_fn: string }
atoms/imports/named.jinja             { from: string, names: string[] }
```

**Example — `atoms/drizzle/column.jinja`:**

```jinja
{#- Register the import — deduplicated by ImportCollector, drained by layout -#}
{{ "import { " ~ drizzle_col_fn(field.type) ~ " } from 'drizzle-orm/sqlite-core'" | collect_import }}

{#- Render the column definition inline -#}
{% if field.type == "string" or field.type == "text" %}
  {{ field.name }}: text('{{ field.name }}'){{ ".notNull()" if field.required }}{{ ".unique()" if field.unique }},
{% elif field.type == "number" %}
  {{ field.name }}: real('{{ field.name }}').notNull(),
{% elif field.type == "boolean" %}
  {{ field.name }}: integer('{{ field.name }}', { mode: 'boolean' }).notNull().default(false),
{% elif field.type == "date" %}
  {{ field.name }}: integer('{{ field.name }}', { mode: 'timestamp' }){{ ".notNull()" if field.required }},
{% elif field.type == "id" %}
  {{ field.name }}: text('{{ field.name }}').references(() => {{ field.references }}.id){{ ".notNull()" if field.required }},
{% elif field.type == "enum" %}
  {{ field.name }}: text('{{ field.name }}', { enum: [{{ field.values | map("tojson") | join(", ") }}] }){{ ".notNull()" if field.required }},
{% elif field.type == "json" %}
  {{ field.name }}: text('{{ field.name }}', { mode: 'json' }){{ ".notNull()" if field.required }},
{% endif %}
```

### 8.3 Tier 2 — Molecules

A molecule is a **MiniJinja macro** that composes multiple atoms into a complete, logically self-contained block. Molecules receive typed context and can expose named caller regions for optional caller-supplied content.

Molecules produce the *body* of a file section, not a complete file. The same `drizzle/table_body` molecule is used by both `add:schema` and `add:feature`, ensuring those two commands can never produce structurally different schema files.

**Example — `molecules/drizzle/table_body.jinja`:**

```jinja
{#
  Context:
    name:        string   — table identifier
    fields:      Field[]  — column definitions
    timestamps:  boolean
    soft_delete: boolean
#}
{{ "import { sqliteTable } from 'drizzle-orm/sqlite-core'" | collect_import }}

export const {{ name | snake_case }} = sqliteTable('{{ name | snake_case }}', {
  id: text('id').primaryKey().$defaultFn(() => crypto.randomUUID()),

  {% for field in fields %}
    {% include "atoms/drizzle/column.jinja" %}
  {% endfor %}

  {% if timestamps %}
    {% include "atoms/drizzle/timestamp_cols.jinja" %}
  {% endif %}

  {% if soft_delete %}
    {% include "atoms/drizzle/soft_delete_col.jinja" %}
  {% endif %}
})

{% if relations is defined %}
  {{ caller() }}
{% endif %}

export type {{ name | pascal_case }} = typeof {{ name | snake_case }}.$inferSelect
export type New{{ name | pascal_case }} = typeof {{ name | snake_case }}.$inferInsert
```

**Molecule catalogue:**

```
molecules/drizzle/table_body.jinja     Full sqliteTable block + type exports
molecules/zod/schema_block.jinja       Complete z.object({...}) with all field rules
molecules/server_fn/handler.jinja      createServerFn().validator().handler() chain
molecules/form/form_component.jinja    useForm hook + JSX field loop + submit button
molecules/table/data_table.jinja       useReactTable columns + thead/tbody/pagination
molecules/query/hooks_block.jinja      useQuery / useSuspenseQuery / useMutation exports
molecules/auth/config_block.jinja      betterAuth({...}) full config body
```

### 8.4 Tier 3 — Layouts

A layout is a MiniJinja base template that provides the **outer shell of a generated file**. It has exactly two responsibilities:

1. Drain the `ImportCollector` — calling `{{ render_imports() }}` which flushes all imports accumulated during child template rendering, deduplicated and sorted, as a single clean block at the top of the file
2. Expose named `{% block %}` regions for molecules to fill

**`layouts/base.jinja`** — plain TypeScript file:

```jinja
{{ render_imports() }}

{% block body %}{% endblock %}
```

**`layouts/component.jinja`** — React component file:

```jinja
{{ "import React from 'react'" | collect_import_priority }}
{{ render_imports() }}

{% block body %}{% endblock %}
```

**`layouts/route.jinja`** — TanStack route file:

```jinja
{{ "import { createFileRoute } from '@tanstack/react-router'" | collect_import_priority }}
{{ "import { useQueryClient } from '@tanstack/react-query'" | collect_import }}
{{ render_imports() }}

export const Route = createFileRoute('{{ route_path }}')({
  {% block loader %}{% endblock %}
  component: RouteComponent,
})

function RouteComponent() {
  {% block body %}{% endblock %}
}
```

### 8.5 Tier 4 — Feature Templates

A feature template is the entry point that a Rust command handler calls. It extends a layout, invokes the appropriate molecule(s) inside layout blocks, and passes the validated payload as context. Feature templates contain **no logic of their own** — they are pure wiring between the command payload and the molecule layer.

**`features/schema.jinja`:**

```jinja
{% extends "layouts/base.jinja" %}
{% block body %}
  {% with name=name, fields=fields, timestamps=timestamps, soft_delete=soft_delete %}
    {% include "molecules/drizzle/table_body.jinja" %}
  {% endwith %}
{% endblock %}
```

**`features/server_fn.jinja`:**

```jinja
{% extends "layouts/base.jinja" %}
{% block body %}
  {% for operation in operations %}
    {% with name=name, table=table, operation=operation, auth=auth %}
      {% include "molecules/server_fn/handler.jinja" %}
    {% endwith %}
  {% endfor %}
{% endblock %}
```

**`features/page.jinja`:**

```jinja
{% extends "layouts/route.jinja" %}
{% block loader %}
  loader: ({ context: { queryClient } }) => {
    return queryClient.ensureQueryData({{ name | camel_case }}QueryOptions())
  },
{% endblock %}
{% block body %}
  {% with name=name, query_hook=query_hook %}
    {% include "molecules/table/data_table.jinja" %}
  {% endwith %}
{% endblock %}
```

### 8.6 ImportCollector — The Rust Side

The import hoisting mechanism is implemented as a MiniJinja custom filter backed by a thread-local accumulator in Rust:

```rust
// render/engine.rs

use std::cell::RefCell;
use std::collections::BTreeSet;

thread_local! {
    static IMPORT_COLLECTOR: RefCell<BTreeSet<String>> = RefCell::new(BTreeSet::new());
    static PRIORITY_IMPORTS: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

pub fn collect_import(value: String) -> String {
    IMPORT_COLLECTOR.with(|c| c.borrow_mut().insert(value));
    String::new() // atom emits nothing inline
}

pub fn collect_import_priority(value: String) -> String {
    PRIORITY_IMPORTS.with(|c| c.borrow_mut().push(value));
    String::new()
}

pub fn render_imports() -> String {
    let priority = PRIORITY_IMPORTS.with(|c| c.borrow().clone());
    let rest: Vec<_> = IMPORT_COLLECTOR.with(|c| c.borrow().iter().cloned().collect());
    let mut all = priority;
    for imp in rest {
        if !all.contains(&imp) {
            all.push(imp);
        }
    }
    all.join("\n")
}
```

These are registered as MiniJinja global functions and filters during engine bootstrap. The thread-local state is reset before each render call, so concurrent renders (if parallelised in future) remain isolated.

### 8.7 Field Type Mapping

Every field type passed via the `fields` array maps deterministically to one atom in each layer:

| Field Type | `drizzle/column` variant | `zod/field_rule` type | `form/field_*` variant | Table column variant |
|---|---|---|---|---|
| `string` | `text` | `z.string()` | `field_input` | `text` |
| `number` | `real` | `z.number()` | `field_input` (type=number) | `text` |
| `boolean` | `integer({ mode: 'boolean' })` | `z.boolean()` | `field_switch` | `boolean` |
| `date` | `integer({ mode: 'timestamp' })` | `z.date()` | `field_datepicker` | `date` |
| `id` | `text().references(...)` | `z.string().uuid()` | `field_select` | `text` |
| `enum` | `text({ enum: [...] })` | `z.enum([...])` | `field_select` | `badge` |
| `json` | `text({ mode: 'json' })` | `z.object({})` | `field_textarea` | `text` |
| `decimal` | `numeric` | `z.number()` | `field_input` | `text` |
| `email` | `text` | `z.string().email()` | `field_input` (type=email) | `text` |
| `url` | `text` | `z.string().url()` | `field_input` (type=url) | `text` |
| `password` | `text` | `z.string().min(8)` | `field_input` (type=password) | — |

---

## 9. Agent Integration

### 9.1 Tool Definition

TSX is exposed to the coding agent as a single shell tool:

```json
{
  "name": "tsx",
  "description": "Generate TanStack Start files from a template. Returns a JSON result with created file paths.",
  "input_schema": {
    "type": "object",
    "properties": {
      "command": { "type": "string", "description": "e.g. add:feature, add:form, add:schema, list, inspect, batch" },
      "options": { "type": "object", "description": "Command-specific options (see list generators)" },
      "overwrite": { "type": "boolean", "default": false },
      "dry_run": { "type": "boolean", "default": false },
      "verbose": { "type": "boolean", "default": false }
    },
    "required": ["command"]
  }
}
```

### 9.2 Agent Workflow Example

A typical agent session for adding a new resource takes 3–5 tool calls instead of 20–30 file writes:

1. `tsx list --json '{"command": "list", "options": {"kind": "generators"}}'` — discover available commands
2. `tsx inspect --json '{"command": "inspect"}'` — check current project state
3. `tsx add:feature --json '{"command": "add:feature", "options": {...}}'` — scaffold the feature
4. `tsx add:migration --json '{"command": "add:migration"}}'` — generate and apply DB migration
5. Remaining calls: business-logic edits to generated server functions only

### 9.3 Result Contract

Every CLI invocation exits with code `0` on success and prints structured JSON to stdout:

```rust
// output.rs
#[derive(Serialize)]
pub struct CommandResult {
    pub success: bool,
    pub version: String,
    pub command: String,
    pub result: Value,
    pub metadata: Metadata,
    pub context: Option<Context>,
    pub next_steps: Vec<String>,
}

#[derive(Serialize)]
pub struct Metadata {
    pub timestamp: String,
    pub duration_ms: u64,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
pub struct Context {
    pub project_root: String,
    pub tsx_version: String,
}
```

---

## 10. Implementation Plan

### Phase 1 — Foundation (Week 1–2)

- Set up Rust workspace; add `clap`, `minijinja`, `serde_json`, `anyhow`, `heck`, `walkdir` dependencies
- Implement MiniJinja engine bootstrap with template directory loading (`atoms/`, `molecules/`, `layouts/`, `features/`)
- Implement `ImportCollector` with `collect_import`, `collect_import_priority`, and `render_imports` as MiniJinja globals/filters
- Implement path resolution utilities and atomic file writing with `--overwrite` guard
- Register custom Jinja filters: `snake_case`, `pascal_case`, `camel_case`, `kebab_case` via `heck`
- Write the core atoms: `drizzle/column`, `drizzle/timestamp_cols`, `zod/field_rule`, `form/field_*`
- Establish atom test harness — render each atom in isolation with a fixture context using `cargo test`
- **NEW**: Implement JSON input mode (`--json`, `--stdin`, `--file` flags)
- **NEW**: Implement structured JSON response envelope with metadata
- **NEW**: Define error codes and structured error format

### Phase 2 — Molecules, Layouts & Core Commands (Week 3–4)

- Write all molecules: `drizzle/table_body`, `zod/schema_block`, `server_fn/handler`, `query/hooks_block`, `form/form_component`, `table/data_table`
- Write all three layouts: `base.jinja`, `component.jinja`, `route.jinja`
- Wire molecules + layouts into feature templates for all single-file commands
- Build field type mapping end-to-end through atom → molecule → feature → compiled output
- Write integration tests rendering full files against expected TypeScript output fixtures

### Phase 3 — Agent-Friendly Features (Week 5)

- **NEW**: Implement `list` command with templates, generators, and components sub-kinds
- **NEW**: Implement `inspect` command for project state
- **NEW**: Implement `batch` command for multiple command execution
- **NEW**: Implement `--verbose` mode with extended metadata
- **NEW**: Implement `dryRun` mode across all commands

### Phase 4 — Compound Commands (Week 5–6)

- Implement `add:feature` — orchestrates molecule rendering across 8 output files; calls single-file command handlers internally
- Write `molecules/auth/config_block.jinja` and implement `add:auth`
- Build barrel file auto-update and import injection utilities
- Implement `init` command

### Phase 5 — Binary Hardening & Distribution (Week 7–8)

- Stress-test with real agent sessions; measure token reduction
- Add `--dry-run`, `--overwrite`, and `--merge` flags
- Set up cross-compilation targets: `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-pc-windows-gnu`
- Publish binary releases via GitHub Actions; document all payload schemas and atom contracts

---

## 11. Success Metrics

| Metric | Target |
|---|---|
| Token reduction | Agent token usage for a 5-resource CRUD app ≤ 30% of no-CLI baseline |
| Consistency score | Zero import/type errors in generated files on first compile across 20 test payloads |
| Agent error rate | Manual edits needed for logic errors in ≤ 10% of generated files |
| Atom test coverage | Every atom has an isolated unit test; 100% branch coverage on field type handling |
| Template drift incidents | Zero — any field type change touches exactly 3 atom files, never N templates |
| Binary startup time | Cold start ≤ 10ms on all target platforms |
| JSON API coverage | 100% of commands available via JSON input/output |

---

## 12. Out of Scope — V1

The following are explicitly deferred to keep V1 focused and shippable:

- Non-TanStack state management (Zustand, Jotai, Redux)
- Alternative UI libraries (Radix primitives without shadcn, MUI, Chakra)
- REST or GraphQL API layer generation
- Multi-tenant / multi-schema Drizzle patterns
- Non-SQLite/PostgreSQL databases
- Interactive TUI prompts (the tool is designed for agent use, not human interactive prompts)
- Atom versioning and registry (planned for V2 — allows pinning specific atom versions per project)
- WebSocket for dev events (planned for V2)

---

## 13. Future Plan — TSX Studio (Desktop GUI)

**TSX Studio** is a planned desktop application, built with **Freya** (Rust-native GPU-accelerated UI framework), that provides a visual interface for managing and editing the RustGen Atoms template library.

### Vision

TSX Studio acts as a code editor and template management workspace, giving template authors a dedicated environment to write, preview, and organise atoms, molecules, layouts, and feature configurations — without touching raw files in a terminal.

### Freya as the UI Framework

Freya is built on top of Skia (via `skia-safe`) and the Dioxus component model, rendering at native GPU speed with no web engine dependency. It provides a React-like component API in Rust with hot-reload support, making it well-suited to a developer tool that needs a fast, code-forward UI.

### Planned Capabilities

- **Atom editor** — syntax-highlighted Jinja2 editor for individual atom files, with live preview rendering the atom against a fixture context
- **Molecule composer** — visual representation of which atoms a molecule includes, with prop contract documentation inline
- **Layout inspector** — shows which blocks a layout defines and which molecules fill them in each feature template
- **Feature configurator** — a form-driven interface to create and edit feature templates, mapping payload fields to molecule slots
- **Template file tree** — hierarchical view of the full `templates/` directory with tier badges (Atom / Molecule / Layout / Feature)
- **Manifest editor** — validated editor for `atoms/manifest.json`, keeping the single source of truth in sync with the filesystem

TSX Studio consumes the same `templates/` directory that the CLI reads, so any edits made in the GUI are immediately usable by the CLI — there is no separate sync step.

This feature is planned for post-V1 and will be tracked as a separate project milestone.

---

## 14. Conclusion

TSX represents a shift in how we think about AI-assisted development. Rather than training agents to write better boilerplate, we eliminate boilerplate from the agent's responsibility entirely. The agent becomes a domain expert — it decides *what* to build — while TSX handles *how* to build it correctly and consistently.

Implementing TSX in Rust brings concrete advantages beyond performance: a single self-contained binary with no runtime, compile-time correctness for the entire payload validation and rendering pipeline, and a native testing harness (`cargo test`) that treats atom rendering as first-class unit tests.

The RustGen Atoms framework, powered by MiniJinja's Jinja2 semantics, provides the same structural guarantees as the original EdgeJS design — layout inheritance, named blocks, macro composition, and import hoisting — in a form that compiles to a 5MB binary with zero external dependencies.

The agent-friendly JSON API — with structured input/output, introspection commands, batch operations, and comprehensive error codes — makes TSX a reliable building block for AI-driven development workflows. Agents can discover capabilities, inspect project state, execute complex operations in batch, and receive consistent, parseable results.

TSX is not a generic scaffolding tool. It is a precision instrument for this stack, and RustGen Atoms is the mechanism that keeps it precise as the stack evolves.
