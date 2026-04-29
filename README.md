# tsx — Universal Framework Protocol CLI

> Agent-native code generation for TanStack Start and any framework that speaks the Protocol.

[![crates.io](https://img.shields.io/crates/v/tsx.svg)](https://crates.io/crates/tsx)
[![license](https://img.shields.io/crates/l/tsx.svg)](LICENSE)

```bash
tsx run add-feature --json '{"name":"products","fields":[{"name":"title","type":"string"}]}'
# → db/schema/products.ts
# → server-functions/products.ts
# → hooks/use-products.ts
# → components/ProductsForm.tsx
# → components/ProductsTable.tsx
# → routes/products/index.tsx
# → routes/products/$id.tsx
```

---

## What tsx does

**tsx** is a Rust CLI that eliminates boilerplate for AI agents building TanStack Start applications. Instead of an agent spending 300–800 tokens writing each file from scratch, it calls a single CLI command and receives fully-wired, production-ready files on disk.

The key shift in **v6** is that tsx becomes a **Universal Framework Protocol** — any framework author can publish a tsx-compatible package that gives agents instant, token-efficient access to their framework's knowledge and code generation.

### Token comparison

```text
Without tsx (agent writes files directly):
  generate schema    → ~600 tokens
  generate server fn → ~500 tokens
  generate query     → ~400 tokens
  generate form      → ~700 tokens
  generate table     → ~600 tokens
  total per feature  → ~2,800 tokens

With tsx:
  tsx describe tanstack-start              → ~80 tokens  (discover what's available)
  tsx ask "how to add auth"               → ~120 tokens (specific answer)
  tsx run add-feature --json '{...}'      → 0 tokens    (file written to disk)
  total per feature                       → ~200 tokens
```

**80–95% token reduction** per typical scaffolding task.

---

## Installation

### Quick Install

**Via Cargo (Rust required):**
```bash
cargo install tsx
```

**Via Pre-built Binary (Linux/macOS):**
```bash
curl -fsSL https://raw.githubusercontent.com/ateeq1999/tsx/main/scripts/install.sh | sh
```

**Via Winget (Windows):**
```powershell
winget install tsx
```

**Build from Source:**
```bash
git clone https://github.com/ateeq1999/tsx.git
cd tsx
cargo build --release
# binary: target/release/tsx.exe (Windows) or target/release/tsx
```

**Requirements:** Rust 1.88+ (for building), PostgreSQL 14+ (for registry server). No Node.js or npm required — single statically-linked binary (~11 MB).

> **Full installation guide** with Docker setup, registry server, platform-specific instructions, and troubleshooting: see [INSTALL.md](INSTALL.md)

---

## Quick Start

```bash
# 1. Scaffold a new project from a starter recipe
tsx create --from tanstack-start --starter basic

# 2. Generate a complete CRUD feature
tsx run add-feature --json '{"name":"products","fields":[{"name":"title","type":"string"},{"name":"price","type":"number"}]}'

# 3. Apply the migration
tsx add migration

# 4. List what else you can generate
tsx run --list
```

---

## Commands

### Code Generation

#### `tsx run <id>` — Universal dispatcher *(recommended)*

Run any generator defined by any installed framework. Validates your JSON input against the generator's schema before executing.

```bash
# List all available generators
tsx run --list
tsx run --list --fw tanstack-start

# Run a generator (by id or command name — both work)
tsx run add-schema    --json '{"name":"users","fields":[{"name":"email","type":"string"}]}'
tsx run add:schema    --json '{"name":"users","fields":[{"name":"email","type":"string"}]}'

# Preview output paths without writing files
tsx run add-feature --json '{"name":"orders"}' --dry-run

# Pipe JSON from stdin
echo '{"name":"orders"}' | tsx run add-feature --stdin

# Read JSON from a file
tsx run add-feature --file feature.json
```

The response includes `next_steps` — the generator tells you what to run next:

```json
{
  "success": true,
  "command": "run",
  "result": {
    "id": "add-feature",
    "framework": "tanstack-start",
    "files_created": ["db/schema/orders.ts", "server-functions/orders.ts", "..."],
    "next_steps": [
      "Run `tsx add migration` to apply the schema",
      "Add <Link to=\"/orders\" /> to your navigation"
    ]
  },
  "metadata": { "duration_ms": 42 }
}
```

#### Built-in generators (TanStack Start)

| Generator | Alias | Output |
| --- | --- | --- |
| `add-schema` | `add:schema` | `db/schema/<name>.ts` — Drizzle ORM table definition |
| `add-server-fn` | `add:server-fn` | `server-functions/<name>.ts` — typed server function |
| `add-query` | `add:query` | `hooks/use-<name>.ts` — TanStack Query hook |
| `add-form` | `add:form` | `components/<name>Form.tsx` — TanStack Form component |
| `add-table` | `add:table` | `components/<name>Table.tsx` — TanStack Table component |
| `add-page` | `add:page` | `routes/<path>/index.tsx` — route page |
| `add-seed` | `add:seed` | `db/seed/<name>.ts` — Drizzle seed file |
| `add-feature` | `add:feature` | All 7 files above in one command |

#### Generator input schemas

Every generator declares its input schema (subset of JSON Schema). `tsx run` validates your input and fills defaults before executing — bad input is rejected with a clear error, not a crash.

#### `add-schema`

```json
{
  "name": "users",
  "fields": [
    { "name": "email", "type": "string", "unique": true },
    { "name": "role",  "type": "string" }
  ],
  "timestamps": true,
  "soft_delete": false
}
```

#### `add-server-fn`

```json
{
  "name": "getUser",
  "method": "GET",
  "auth": true,
  "return_type": "User"
}
```

#### `add-query`

```json
{
  "name": "user",
  "operations": ["list", "get", "create", "update", "delete"]
}
```

#### `add-form`

```json
{
  "name": "User",
  "fields": [
    { "name": "email", "type": "email", "required": true },
    { "name": "role",  "type": "select" }
  ],
  "submit_label": "Save"
}
```

#### `add-table`

```json
{
  "name": "User",
  "columns": [
    { "key": "email", "label": "Email", "sortable": true },
    { "key": "role",  "label": "Role" }
  ],
  "searchable": true,
  "pagination": true
}
```

#### `add-page`

```json
{
  "path": "dashboard",
  "auth": true,
  "loader": false
}
```

#### `add-seed`

```json
{
  "name": "users",
  "count": 20
}
```

#### `add-feature`

```json
{
  "name": "products",
  "fields": [
    { "name": "title",  "type": "string", "required": true },
    { "name": "price",  "type": "number" },
    { "name": "active", "type": "boolean" }
  ],
  "timestamps": true,
  "auth": false
}
```

---

### Named subcommands (also available)

These are stable aliases for the most common generators. They accept the same `--json` / `--stdin` / `--file` flags.

```bash
tsx generate schema    --json '{"name":"users",...}'
tsx generate server-fn --json '{"name":"getUser",...}'
tsx generate query     --json '{"name":"user",...}'
tsx generate form      --json '{"name":"User",...}'
tsx generate table     --json '{"name":"User",...}'
tsx generate page      --json '{"path":"dashboard",...}'
tsx generate seed      --json '{"name":"users",...}'
tsx generate feature   --json '{"name":"products",...}'

tsx add auth       --json '{"providers":["github","google"]}'
tsx add auth-guard --json '{"route_path":"/dashboard","redirect_to":"/login"}'
tsx add migration
```

---

### Scaffolding

#### `tsx create`

Scaffold a full project from a starter recipe.

```bash
tsx create --from tanstack-start                       # built-in basic starter
tsx create --from tanstack-start --starter with-auth   # starter with Better Auth
tsx create --from tanstack-start --starter saas        # SaaS starter
tsx create --from @tsx-pkg/tanstack-start              # from npm package
tsx create --from github:user/my-tsx-pkg               # from GitHub repo
tsx create --from tanstack-start --dry-run             # preview steps only
```

Available starters for `tanstack-start`: `basic`, `with-auth`, `saas`.

---

### Framework Knowledge

tsx is a conversation partner for any installed framework — not just a code generator.

#### `tsx describe <framework>`

Agent entry point. Returns what knowledge is available and its token cost before committing to loading anything.

```bash
tsx describe tanstack-start
tsx describe tanstack-start --section overview
tsx describe tanstack-start --section faq
```

```json
{
  "framework": "TanStack Start",
  "version": "1.0.0",
  "available_knowledge": {
    "overview":  { "token_estimate": 150, "cmd": "tsx describe tanstack-start --section overview" },
    "concepts":  { "token_estimate": 400, "cmd": "tsx describe tanstack-start --section concepts" },
    "patterns":  { "token_estimate": 600 },
    "faq_topics": 28
  },
  "generators": 8,
  "starters": ["basic", "with-auth", "saas"],
  "quick_start": "tsx create --from tanstack-start --starter basic"
}
```

#### `tsx ask`

Answer questions about a framework. Framework is auto-detected from `package.json` when omitted.

```bash
tsx ask --question "How do I add authentication?"
tsx ask --question "How do I add authentication?" --framework tanstack-start
tsx ask --question "How do I add authentication?" --framework tanstack-start --depth brief
tsx ask --question "How do I add authentication?" --framework tanstack-start --depth full
```

`--depth` values: `brief` (~50 tokens), `default` (~150 tokens), `full` (~400 tokens).

#### `tsx where`

Find where things live in a framework.

```bash
tsx where --thing schema
tsx where --thing "route page"  --framework tanstack-start
```

#### `tsx how`

Get integration steps for a package.

```bash
tsx how --integration "@tanstack/react-router"
tsx how --integration better-auth --framework tanstack-start
```

#### `tsx explain`

Explain template decisions and architecture.

```bash
tsx explain --topic atom
tsx explain --topic "why tera over minijinja"
```

---

### Framework Management

#### `tsx framework` — Author tools

```bash
tsx framework init --name my-framework      # scaffold a new framework package
tsx framework validate                       # lint manifest + templates in cwd
tsx framework validate --path ./my-pkg       # lint a specific path
tsx framework preview --template auth.forge --data '{"name":"users"}'
tsx framework add ./my-pkg                   # install from local path
tsx framework add @tsx-pkg/stripe            # install from npm
tsx framework list                           # list installed packages
tsx framework publish                        # publish to npm as @tsx-pkg/<id>
tsx framework publish --dry-run              # validate without uploading
```

---

### Operations

#### `tsx batch`

Execute multiple generators in a single call with rollback support.

```bash
tsx batch --json '{
  "stop_on_failure": true,
  "rollback_on_failure": true,
  "commands": [
    { "command": "add:schema",    "options": {"name":"orders","fields":[...]} },
    { "command": "add:server-fn", "options": {"name":"getOrders"} },
    { "command": "add:query",     "options": {"name":"order"} }
  ]
}'
```

Supports `--stream` to emit each result as newline-delimited JSON as it completes.

#### `tsx inspect`

Scan the current project and return its structure, detected framework, auth config, and migration status.

```bash
tsx inspect
tsx inspect --verbose
```

#### `tsx list`

```bash
tsx list --kind templates
tsx list --kind generators
tsx list --kind frameworks
tsx list --kind components
```

#### `tsx dev`

Start the development server with optional JSON event streaming and WebSocket support.

```bash
tsx dev
tsx dev --json-events              # emit structured JSON events to stdout
tsx dev --watch                    # regenerate on template changes
tsx dev --ws-port 7332             # WebSocket server for IDE integration
```

#### `tsx subscribe`

Start a Server-Sent Events server for external tool integration.

```bash
tsx subscribe --port 7331
```

---

### Utility Commands

#### `tsx path` — Manage system PATH

Add current directory or specified directory to system PATH:

```bash
# Add current directory to PATH (session-only)
tsx path .

# Add specific directory
tsx path /path/to/add

# Add permanently (Windows: setx, Unix: shell profile)
tsx path . --permanent

# List current PATH entries
tsx path --list
```

**Windows:** Uses `setx /M PATH` (requires admin) for permanent addition.
**Unix:** Appends export to `.bashrc` or `.zshrc`.

#### `tsx adb` — Android Debug Bridge

Manage ADB server and devices:

```bash
# Kill ADB server
tsx adb kill

# Start ADB server
tsx adb start

# Check ADB status and list devices
tsx adb status

# Reverse port from device to host
tsx adb reverse --port 3333

# Execute arbitrary adb command
tsx adb exec devices -l
```

#### `tsx flutter` — Flutter Development

Run Flutter commands through tsx:

```bash
# Run Flutter app in profile mode (default)
tsx flutter run

# Run in debug mode on specific device
tsx flutter run --mode debug --device emulator-5554

# Run on custom port
tsx flutter run --port 8080

# Build APK
tsx flutter build --target apk

# Build in release mode
tsx flutter build --release

# Clean build artifacts
tsx flutter clean

# Get packages
tsx flutter pub-get
```

#### `tsx port` — Port/Process Management

Find and kill processes using specific ports:

```bash
# Find processes using port 8080
tsx port find --port 8080

# Kill all processes using port 3000
tsx port kill --port 3000
```

**Windows:** Uses `netstat -ano` and `taskkill /F /PID`.
**Unix:** Uses `lsof -ti` and `kill -9`.

---

### Plugin System

```bash
tsx plugin list
tsx plugin install --source ./my-plugin
tsx plugin install --source @my-org/tsx-plugin-stripe
tsx plugin remove  --package @my-org/tsx-plugin-stripe
```

---

### Global Flags

| Flag | Description |
| --- | --- |
| `--overwrite` | Overwrite existing files without prompting |
| `--dry-run` | Preview what would be written without creating files |
| `--verbose` | Include project root, tsx version, and extended context in response |
| `--stdin` | Read JSON payload from stdin |
| `--file <PATH>` | Read JSON payload from a file |

---

## JSON API

All commands return a consistent JSON envelope — designed for AI agent consumption.

### Success response

```json
{
  "success": true,
  "version": "1.0",
  "command": "run",
  "result": {
    "id": "add-feature",
    "framework": "tanstack-start",
    "files_created": ["db/schema/products.ts", "..."],
    "next_steps": ["Run `tsx add migration`"]
  },
  "metadata": {
    "timestamp": "2026-03-16T10:00:00Z",
    "duration_ms": 38
  }
}
```

### Error response

```json
{
  "success": false,
  "command": "run",
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "missing required field 'name'"
  },
  "metadata": { "duration_ms": 1 }
}
```

### Error codes

| Code | Meaning |
| --- | --- |
| `INVALID_PAYLOAD` | JSON payload is malformed |
| `VALIDATION_ERROR` | Input fails schema validation |
| `UNKNOWN_COMMAND` | Generator ID not found in any installed framework |
| `UNKNOWN_KIND` | `--kind` value not recognised by `tsx list` |
| `FILE_EXISTS` | Target file exists and `--overwrite` was not set |
| `DIRECTORY_NOT_FOUND` | Required parent directory does not exist |
| `PERMISSION_DENIED` | Cannot write to target location |
| `TEMPLATE_NOT_FOUND` | Generator template file missing |
| `PROJECT_NOT_FOUND` | No `package.json` found — run from a project directory |
| `INTERNAL_ERROR` | Unexpected error in the CLI |

---

## Framework Package Format

Framework authors publish tsx-compatible packages to npm as `@tsx-pkg/<name>`. Once installed with `tsx framework add @tsx-pkg/<name>`, all generators become available via `tsx run`.

### Directory layout

```text
@tsx-pkg/tanstack-start/
  manifest.json           ← package identity, generators, starters
  knowledge/
    overview.md           ← what this framework is      (≤ 150 tokens)
    concepts.md           ← key terms + glossary         (≤ 400 tokens)
    patterns.md           ← common patterns with snippets (≤ 600 tokens)
    faq.md                ← Q&A pairs (frontmatter-structured)
    decisions.md          ← design rationale             (≤ 500 tokens)
  generators/
    add-schema.json       ← generator spec (id + command + schema + output_paths)
    add-feature.json
    ...
  templates/
    atoms/                ← indivisible code fragments (.forge files)
    molecules/            ← composed blocks
    layouts/              ← file-level shells
    features/             ← full output templates
  starters/
    basic.json            ← ordered command steps
    with-auth.json
  integrations/
    better-auth.json      ← integration pattern
    drizzle-orm.json
```

### manifest.json

```json
{
  "id": "tanstack-start",
  "name": "TanStack Start",
  "version": "1.0.0",
  "category": "framework",
  "description": "Full-stack React meta-framework",
  "docs": "https://tanstack.com/start",
  "peer_dependencies": {
    "@tanstack/start": "^1.0",
    "@tanstack/react-router": "^1.0"
  },
  "knowledge_token_budget": {
    "overview": 150,
    "concepts": 400,
    "patterns": 600,
    "faq_per_entry": 120
  },
  "generators": ["add-schema", "add-server-fn", "add-query", "add-form", "add-table", "add-page", "add-seed", "add-feature"],
  "starters": ["basic", "with-auth", "saas"],
  "templates_dir": "templates",
  "template_format": "forge"
}
```

### Generator spec (`generators/<id>.json`)

```json
{
  "id": "add-schema",
  "command": "add:schema",
  "description": "Generate a Drizzle ORM schema table definition",
  "token_estimate": 30,
  "output_paths": ["db/schema/{{name}}.ts"],
  "next_steps": [
    "Run `tsx add migration` to apply the schema to your database"
  ],
  "schema": {
    "type": "object",
    "required": ["name"],
    "properties": {
      "name": { "type": "string", "description": "Table name (snake_case)" },
      "timestamps": { "type": "boolean", "default": true },
      "soft_delete": { "type": "boolean", "default": false }
    }
  }
}
```

### Knowledge file format (frontmatter)

```markdown
---
id: add-auth
question: How do I add authentication?
tags: [auth, security, setup]
token_estimate: 120
requires: [better-auth]
related: [add-migration, add-auth-guard]
---

## Adding Authentication

Run `tsx add auth` to scaffold Better Auth. This creates `lib/auth.ts`...
```

### Template format (`.forge` files — Tera syntax)

```jinja
{# forge:tier atom #}
{# forge:token_estimate 12 #}

{% if field.type == "string" %}
  {{ field.name | snake_case }}: text('{{ field.name }}'){{ collect_import("text", "drizzle-orm/sqlite-core") }},
{% elif field.type == "number" %}
  {{ field.name | snake_case }}: integer('{{ field.name }}'){{ collect_import("integer", "drizzle-orm/sqlite-core") }},
{% endif %}
```

Built-in forge filters: `collect_import`, `snake_case`, `pascal_case`, `camel_case`, `kebab_case`.
Built-in forge functions: `render_imports()` — drains the ImportCollector at the file top.

### Starter recipe (`starters/<id>.json`)

```json
{
  "id": "with-auth",
  "name": "With Auth Starter",
  "description": "TanStack Start with Better Auth pre-configured",
  "token_estimate": 40,
  "steps": [
    { "cmd": "init",        "args": {} },
    { "cmd": "add:auth",    "args": { "providers": ["email"] } },
    { "cmd": "add:schema",  "args": { "name": "users", "timestamps": true } },
    { "cmd": "add:migration","args": {} }
  ]
}
```

---

## Architecture

### Repository structure

```text
tsx/
├── src/
│   ├── main.rs                   ← clap CLI definition + dispatch
│   ├── commands/
│   │   ├── generate/             ← add_schema, add_feature, etc. (compiled-in)
│   │   ├── manage/               ← create, framework_cmd, init, dev
│   │   ├── ops/
│   │   │   ├── batch.rs          ← batch executor + execute_command_pub
│   │   │   ├── run.rs            ← universal dispatcher (tsx run)
│   │   │   └── generate.rs       ← framework generator runner
│   │   └── query/                ← ask, describe, where, how, explain
│   ├── framework/
│   │   ├── command_registry.rs   ← scans generators/ dirs, validates input
│   │   ├── loader.rs             ← loads manifest.json + registry.json formats
│   │   ├── detect.rs             ← auto-detects framework from package.json
│   │   ├── knowledge.rs          ← markdown frontmatter parser
│   │   ├── token_budget.rs       ← depth system (brief/default/full)
│   │   └── package_cache.rs      ← .tsx/frameworks/packages.json tracking
│   ├── schemas/                  ← serde structs for all command payloads
│   ├── render/                   ← MiniJinja engine + ImportCollector
│   ├── utils/                    ← paths, write, imports, barrel, format
│   └── json/                     ← payload, response, error types
├── crates/forge/                 ← the forge engine crate (published as tsx-forge)
│   └── src/
│       ├── engine.rs             ← Tera wrapper with tier awareness
│       ├── collector.rs          ← ImportCollector (thread-local BTreeSet)
│       ├── tier.rs               ← Atom/Molecule/Layout/Feature types
│       ├── context.rs            ← ForgeContext builder with provide/inject + slots
│       ├── slots.rs              ← Component slot system (thread-local)
│       ├── provide.rs            ← Provide/Inject context propagation (thread-local)
│       └── metadata.rs           ← token_estimate frontmatter reader
├── frameworks/
│   └── tanstack-start/           ← built-in reference implementation
│       ├── manifest.json
│       ├── knowledge/            ← overview, concepts, patterns, faq, decisions
│       ├── generators/           ← 8 generator specs (JSON Schema + output_paths)
│       ├── templates/            ← 33 .forge templates (atoms/molecules/layouts/features)
│       ├── starters/             ← basic, with-auth, saas recipes
│       └── integrations/         ← better-auth, drizzle-orm, shadcn-ui, etc.
└── templates/                    ← compiled-in MiniJinja templates (embedded in binary)
    ├── atoms/
    ├── molecules/
    ├── layouts/
    └── features/
```

### The forge engine (4-tier system)

Code generation is powered by the `forge` crate (built on Tera). Templates are composed in four tiers:

| Tier | Role | Jinja2 primitive |
| --- | --- | --- |
| **Atoms** | Indivisible code fragments (a single column definition, a single Zod rule) | `{% include %}` |
| **Molecules** | Atoms composed into logical blocks (a full table body, a form component) | `{% macro %}` / `{{ caller() }}` |
| **Layouts** | File-level shells that emit imports at the top and wrap molecules | `{% extends %}` + `{% block %}` |
| **Features** | Complete output templates that wire molecules into layouts | Top-level template |

**ImportCollector** — imports are accumulated via `{{ collect_import("text", "drizzle-orm/sqlite-core") }}` deep inside atoms, then drained and emitted as a deduplicated, sorted block at the file top via `{{ render_imports() }}` in the layout. This ensures every generated file has correct imports regardless of which atoms were included.

**Component slots** — layouts declare `{{ slot(name='body') }}` placeholders; feature templates fill them with `.slot("body", content)` on `ForgeContext`.

**Provide/Inject** — parent contexts make values available via `.provide("theme", "dark")`; any descendant template reads them with `{{ inject(key='theme') }}`.

### CommandRegistry — how `tsx run` finds generators

At startup, `tsx run` instantiates `CommandRegistry::load_all()` which:

1. Scans `<exe_dir>/frameworks/` (built-in, shipped with the binary)
2. Scans `<cwd>/.tsx/frameworks/` (user-installed packages)
3. Reads every `generators/<id>.json` from each framework directory
4. Resolves by `id` (`"add-schema"`) or `command` (`"add:schema"`) — both work

This means adding a new generator to any installed framework package requires zero Rust code — just drop a JSON file.

---

## Development

### Build

```bash
cargo build           # debug build
cargo build --release # production build (LTO, stripped, ~11 MB)
```

### Test

```bash
cargo test               # all tests
cargo test --lib         # unit tests only
cargo test -p tsx-forge  # forge crate tests
```

### Benchmark (forge engine)

```bash
cargo bench --bench render_bench -p tsx-forge
```

Benchmarks compare `forge` (Tera-based) vs MiniJinja on the full atom → molecule → layout → feature pipeline.

### Project layout for contributors

- Command handlers live in `src/commands/` — one module per logical group
- All commands return `CommandResult` and print a `ResponseEnvelope` — never `eprintln!`
- New generators for `tanstack-start` go in `frameworks/tanstack-start/generators/<id>.json` — no Rust required
- New compiled-in commands need a handler in `src/commands/`, a schema in `src/schemas/`, a dispatch arm in `src/main.rs`, and registration in `src/commands/ops/batch.rs`'s `execute_command()`

---

## Technology Stack

### CLI (Rust)

| Crate | Role |
| --- | --- |
| `clap` v4 | Argument parsing and subcommand routing |
| `tsx-forge` | 4-tier code generation engine (Tera-based) |
| `minijinja` | Template engine for compiled-in templates |
| `serde` + `serde_json` | JSON payload deserialisation and structured output |
| `anyhow` | Error propagation |
| `walkdir` | Project root auto-detection |
| `heck` | Case conversion (`snake_case`, `PascalCase`, `camelCase`) |
| `reqwest` + `tokio` | npm package fetching |
| `notify` | File system watcher (dev mode) |
| `tungstenite` | WebSocket server (dev mode) |

### Generated application stack

| Technology | Role |
| --- | --- |
| TanStack Start | Full-stack React meta-framework |
| TanStack Router | Type-safe file-based routing |
| TanStack Query | Server state and caching |
| TanStack Form | Type-safe form handling |
| TanStack Table | Headless data tables |
| shadcn/ui | Accessible component library |
| Better Auth | Authentication |
| Drizzle ORM | Type-safe SQL query builder |
| Tailwind CSS | Utility-first CSS |

---

## License

MIT — see [LICENSE](LICENSE).
