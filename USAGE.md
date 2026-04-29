# tsx — Usage Guide

This guide covers every command available in the tsx CLI. All commands emit a consistent JSON envelope, making them safe to parse in AI agent pipelines or shell scripts.

---

## Table of Contents

- [Global Flags](#global-flags)
- [JSON API Contract](#json-api-contract)
- [Code Generation](#code-generation)
  - [tsx run](#tsx-run--universal-dispatcher)
  - [tsx generate](#tsx-generate--named-generators)
  - [tsx add](#tsx-add--integrations)
- [Scaffolding](#scaffolding)
  - [tsx create](#tsx-create)
  - [tsx init](#tsx-init)
- [Framework Knowledge](#framework-knowledge)
  - [tsx describe](#tsx-describe)
  - [tsx ask](#tsx-ask)
  - [tsx where](#tsx-where)
  - [tsx how](#tsx-how)
  - [tsx explain](#tsx-explain)
- [Framework Management](#framework-management)
  - [tsx framework](#tsx-framework)
  - [tsx registry](#tsx-registry)
- [Project Operations](#project-operations)
  - [tsx inspect](#tsx-inspect)
  - [tsx batch](#tsx-batch)
  - [tsx list](#tsx-list)
  - [tsx migrate](#tsx-migrate)
  - [tsx build](#tsx-build)
  - [tsx test](#tsx-test)
  - [tsx audit](#tsx-audit)
  - [tsx dev](#tsx-dev)
  - [tsx watch](#tsx-watch)
- [Pattern System](#pattern-system)
  - [tsx pattern](#tsx-pattern)
- [Snapshot Testing](#snapshot-testing)
  - [tsx snapshot](#tsx-snapshot)
- [Authentication & Registry](#authentication--registry)
  - [tsx login / logout / whoami](#tsx-login--logout--whoami)
  - [tsx pkg / package](#tsx-pkg--package)
  - [tsx publish](#tsx-publish)
- [Stack Management](#stack-management)
  - [tsx stack](#tsx-stack)
- [Plugin System](#plugin-system)
  - [tsx plugin](#tsx-plugin)
  - [tsx template](#tsx-template)
- [Developer Tools](#developer-tools)
  - [tsx upgrade](#tsx-upgrade)
  - [tsx codegen](#tsx-codegen)
  - [tsx analyze](#tsx-analyze)
  - [tsx plan](#tsx-plan)
  - [tsx context](#tsx-context)
  - [tsx repl](#tsx-repl)
  - [tsx replay](#tsx-replay)
  - [tsx atoms](#tsx-atoms)
  - [tsx doctor](#tsx-doctor)
  - [tsx fmt](#tsx-fmt)
  - [tsx lint-template](#tsx-lint-template)
  - [tsx env](#tsx-env)
  - [tsx config](#tsx-config)
  - [tsx docs](#tsx-docs)
  - [tsx tui](#tsx-tui)
  - [tsx snapshot](#tsx-snapshot)
  - [tsx completions](#tsx-completions)
  - [tsx lsp](#tsx-lsp)
  - [tsx mcp](#tsx-mcp)
- [Utility Commands](#utility-commands)
  - [tsx path](#tsx-path)
  - [tsx adb](#tsx-adb)
  - [tsx flutter](#tsx-flutter)
  - [tsx port](#tsx-port)

---

## Global Flags

These flags apply to every command.

| Flag | Description |
| --- | --- |
| `--overwrite` | Overwrite existing files without prompting |
| `--dry-run` | Preview what would be written without creating files |
| `--verbose` | Include project root, tsx version, and extended context in the response |
| `--diff` | Show a unified diff of what would change without writing files |
| `--stdin` | Read the JSON payload from stdin |
| `--file <PATH>` | Read the JSON payload from a file |

---

## JSON API Contract

Every command returns a JSON object to stdout. This makes tsx safe to drive from AI agents and scripts.

### Success envelope

```json
{
  "success": true,
  "version": "1.0",
  "command": "<command-name>",
  "result": { /* command-specific payload */ },
  "metadata": {
    "timestamp": "2026-04-29T10:00:00Z",
    "duration_ms": 38
  }
}
```

### Error envelope

```json
{
  "success": false,
  "command": "<command-name>",
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

## Code Generation

### `tsx run` — Universal dispatcher

Run any generator from any installed framework by its ID or command alias.

```bash
# List all available generators
tsx run --list
tsx run --list --fw tanstack-start

# Run by generator ID
tsx run add-schema --json '{"name":"users","fields":[{"name":"email","type":"string"}]}'

# Run by command alias (colon form also works)
tsx run add:schema --json '{"name":"users","fields":[{"name":"email","type":"string"}]}'

# Preview output paths without writing files
tsx run add-feature --json '{"name":"orders"}' --dry-run

# Show unified diff instead of writing
tsx run add-schema --json '{"name":"orders"}' --diff

# Pipe JSON from stdin
echo '{"name":"orders"}' | tsx run add-feature --stdin

# Read JSON from a file
tsx run add-feature --file feature.json

# Overwrite existing files
tsx run add-schema --json '{"name":"users"}' --overwrite

# Target a specific framework
tsx run add-schema --fw tanstack-start --json '{"name":"users"}'
```

The response includes `next_steps` — the generator tells you what to run next:

```json
{
  "success": true,
  "command": "run",
  "result": {
    "id": "add-feature",
    "framework": "tanstack-start",
    "files_created": [
      "db/schema/orders.ts",
      "server-functions/orders.ts",
      "hooks/use-orders.ts",
      "components/OrdersForm.tsx",
      "components/OrdersTable.tsx",
      "routes/orders/index.tsx",
      "routes/orders/$id.tsx"
    ],
    "next_steps": [
      "Run `tsx add migration` to apply the schema",
      "Add <Link to=\"/orders\" /> to your navigation"
    ]
  },
  "metadata": { "duration_ms": 42 }
}
```

#### Built-in generators (TanStack Start)

| Generator ID | Alias | Output |
| --- | --- | --- |
| `add-schema` | `add:schema` | `db/schema/<name>.ts` |
| `add-server-fn` | `add:server-fn` | `server-functions/<name>.ts` |
| `add-query` | `add:query` | `hooks/use-<name>.ts` |
| `add-form` | `add:form` | `components/<name>Form.tsx` |
| `add-table` | `add:table` | `components/<name>Table.tsx` |
| `add-page` | `add:page` | `routes/<path>/index.tsx` |
| `add-seed` | `add:seed` | `db/seed/<name>.ts` |
| `add-feature` | `add:feature` | All 7 files above in one command |

#### Generator input schemas

##### `add-schema`

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

##### `add-server-fn`

```json
{
  "name": "getUser",
  "method": "GET",
  "auth": true,
  "return_type": "User"
}
```

##### `add-query`

```json
{
  "name": "user",
  "operations": ["list", "get", "create", "update", "delete"]
}
```

##### `add-form`

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

##### `add-table`

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

##### `add-page`

```json
{
  "path": "dashboard",
  "auth": true,
  "loader": false
}
```

##### `add-seed`

```json
{
  "name": "users",
  "count": 20
}
```

##### `add-feature`

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

### `tsx generate` — Named generators

Stable named aliases for the most common generators. Accept the same `--json` / `--stdin` / `--file` / `--overwrite` / `--dry-run` flags.

```bash
tsx generate schema    --json '{"name":"users"}'
tsx generate server-fn --json '{"name":"getUser","auth":true}'
tsx generate query     --json '{"name":"user","operations":["list","get"]}'
tsx generate form      --json '{"name":"User","fields":[...]}'
tsx generate table     --json '{"name":"User","columns":[...]}'
tsx generate page      --json '{"path":"dashboard","auth":true}'
tsx generate seed      --json '{"name":"users","count":20}'
tsx generate feature   --json '{"name":"products","fields":[...]}'

# Run a framework-defined generator by ID (fallback for any installed framework)
tsx generate fw --id add-schema --json '{"name":"users"}'
tsx generate fw --id add-schema --fw tanstack-start --json '{"name":"users"}'
```

---

### `tsx add` — Integrations

```bash
# Configure Better Auth (email/OAuth providers)
tsx add auth --json '{"providers":["github","google"]}'

# Wrap a route with a session guard
tsx add auth-guard --json '{"route_path":"/dashboard","redirect_to":"/login"}'

# Generate and apply a Drizzle migration
tsx add migration
```

---

## Scaffolding

### `tsx create`

Scaffold a full project from a starter recipe.

```bash
# Built-in basic starter (no auth)
tsx create --from tanstack-start

# Starter with Better Auth pre-wired
tsx create --from tanstack-start --starter with-auth

# Full SaaS starter (auth, billing, org)
tsx create --from tanstack-start --starter saas

# Install from a published npm framework package
tsx create --from @tsx-pkg/tanstack-start

# Install from a GitHub repo
tsx create --from github:user/my-tsx-pkg

# Preview steps without executing
tsx create --from tanstack-start --dry-run
```

Available built-in starters for `tanstack-start`: `basic`, `with-auth`, `saas`.

---

### `tsx init`

Initialize tsx in an existing project.

```bash
# Auto-detect from package.json
tsx init

# Specify a project name
tsx init --name my-app

# Activate specific packages (comma-separated)
tsx init --stack tanstack-start,drizzle-pg,better-auth
```

---

## Framework Knowledge

### `tsx describe`

Agent entry point — returns available knowledge sections and their token cost before loading anything. Accepts a framework slug or generator command ID.

```bash
tsx describe tanstack-start
tsx describe tanstack-start --section overview
tsx describe tanstack-start --section concepts
tsx describe tanstack-start --section patterns
tsx describe tanstack-start --section faq
tsx describe tanstack-start --section decisions

# Or pass via --framework flag
tsx describe --framework tanstack-start
```

Example response:

```json
{
  "framework": "TanStack Start",
  "version": "1.0.0",
  "available_knowledge": {
    "overview":  { "token_estimate": 150, "cmd": "tsx describe tanstack-start --section overview" },
    "concepts":  { "token_estimate": 400 },
    "patterns":  { "token_estimate": 600 },
    "faq_topics": 28
  },
  "generators": 8,
  "starters": ["basic", "with-auth", "saas"],
  "quick_start": "tsx create --from tanstack-start --starter basic"
}
```

---

### `tsx ask`

Answer a natural-language question about a framework. Auto-detects the framework from `package.json` when `--framework` is omitted.

```bash
tsx ask --question "How do I add authentication?"
tsx ask --question "How do I set up server-side rendering?" --framework tanstack-start
tsx ask --question "What is a server function?" --framework tanstack-start --depth brief
tsx ask --question "How do I add pagination?" --framework tanstack-start --depth full
```

`--depth` values:
- `brief` — ~50 tokens, one-liner answer
- `default` — ~150 tokens (default)
- `full` — ~400 tokens, detailed with code examples

---

### `tsx where`

Find where things live in a framework.

```bash
tsx where --thing schema
tsx where --thing "route page" --framework tanstack-start
tsx where --thing "server function"
```

---

### `tsx how`

Get integration steps for a package or integration.

```bash
tsx how --integration "@tanstack/react-router"
tsx how --integration better-auth --framework tanstack-start
tsx how --integration drizzle-orm
```

---

### `tsx explain`

Explain template decisions and architecture conventions.

```bash
tsx explain --topic atom
tsx explain --topic "why tera over minijinja"
tsx explain --topic "import collector"
tsx explain --topic "slot system"
```

---

## Framework Management

### `tsx framework`

Author tools for creating and publishing tsx-compatible framework packages.

```bash
# Scaffold a new framework package directory
tsx framework init --name my-framework

# Validate a framework package (manifest + templates)
tsx framework validate
tsx framework validate --path ./my-pkg

# Render a template with test data
tsx framework preview --template auth.forge --data '{"name":"users"}'

# Install a framework from a local directory
tsx framework add ./my-pkg

# Install from npm
tsx framework add @tsx-pkg/stripe

# List installed framework packages
tsx framework list

# Publish to npm as @tsx-pkg/<id>
tsx framework publish
tsx framework publish --dry-run
tsx framework publish --path ./my-pkg
tsx framework publish --registry https://registry.example.com --api-key <key>
```

---

### `tsx registry`

Discover and manage community framework registries from npm.

```bash
# Search npm for tsx-framework-* packages
tsx registry search
tsx registry search --query "stripe"

# Install a community registry
tsx registry install --package @tsx-pkg/stripe

# List installed community registries
tsx registry list

# Check for and apply updates to all installed packages
tsx registry update

# Show metadata for a package
tsx registry info @tsx-pkg/drizzle-pg

# Generate a static HTML catalog website
tsx registry website
tsx registry website --output ./dist/catalog
```

---

## Project Operations

### `tsx inspect`

Scan the current project and return its structure, detected framework, auth config, and migration status.

```bash
tsx inspect
tsx inspect --verbose
```

---

### `tsx batch`

Execute multiple generators in a single call with optional rollback support.

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

# Stream each result as it completes (newline-delimited JSON)
tsx batch --json '{...}' --stream

# Plan mode: resolve commands and show what would be created without executing
tsx batch --json '{...}' --plan

# Same as --plan
tsx batch --json '{...}' --dry-run
```

---

### `tsx list`

List available templates, generators, components, or frameworks.

```bash
tsx list --kind templates
tsx list --kind generators
tsx list --kind frameworks
tsx list --kind components

# Agent mode: omit --kind to get all registry generators with full metadata
tsx list
tsx list --verbose
```

---

### `tsx migrate`

Run Drizzle-Kit database migrations.

```bash
# Generate migration SQL and apply it
tsx migrate

# Only generate (skip apply)
tsx migrate --generate-only

# Only apply pending migrations (skip generate)
tsx migrate --apply-only

# Preview without writing
tsx migrate --dry-run
```

---

### `tsx build`

Detect and run the project's build command.

```bash
tsx build
tsx build --json-events    # structured JSON output for agent consumption
```

---

### `tsx test`

Run the project's test suite (vitest / jest / playwright — auto-detected).

```bash
tsx test
tsx test --filter "auth"          # only tests matching pattern
tsx test --watch                  # re-run on file changes
tsx test --json                   # structured JSON test results
```

---

### `tsx audit`

Run `npm audit` and format vulnerability output.

```bash
tsx audit
tsx audit --severity high         # report only high and critical
tsx audit --fix                   # run npm audit fix
```

---

### `tsx dev`

Start the development server with optional structured event streaming.

```bash
tsx dev
tsx dev --json-events              # emit structured JSON events to stdout
tsx dev --watch                    # regenerate on template/source changes
tsx dev --ws-port 7332             # WebSocket server for IDE integration
```

---

### `tsx watch`

Watch files and re-run generators on change.

```bash
# Watch current directory (default extensions: ts tsx js rs forge jinja)
tsx watch

# Watch specific paths
tsx watch src/ templates/

# Watch specific extensions only
tsx watch --ext forge,jinja

# Run a command on each change
tsx watch --run "tsx run add-schema --json @schema.json"

# Tune the debounce window (default: 300ms)
tsx watch --debounce 500

# Structured JSON events
tsx watch --json
```

---

## Pattern System

### `tsx pattern`

User-defined generator patterns — reusable template packs stored in `.tsx/patterns/`.

```bash
# Scaffold a new pack
tsx pattern new my-service --name "Service Pack" --description "CRUD service layer"
tsx pattern new my-service --framework tanstack-start

# Run a pack
tsx pattern run my-service --arg name=Order --arg entity=order
tsx pattern run my-service --command create --arg name=User

# Preview without writing
tsx pattern run my-service --arg name=Order --dry-run

# Install from local path or GitHub
tsx pattern install ./my-pack
tsx pattern install github:user/repo#subpath@v1.0.0
tsx pattern install github:user/repo --id my-pack

# Lint a pack's templates and manifest
tsx pattern lint my-service

# Add a pattern from a template file
tsx pattern add --name add-service --template ./templates/service.forge
tsx pattern add --name add-service --description "CRUD service" --args "name:string,entity:string"

# Record file changes as a new pattern
tsx pattern record --name my-changes
# ...make file changes...
tsx pattern record --stop

# List local packs
tsx pattern list
tsx pattern list --builtin         # show built-in packs

# Show details of a pack
tsx pattern show my-service

# Remove a pack
tsx pattern remove my-service

# Publish a pack to the registry
tsx pattern publish my-service
tsx pattern publish my-service --registry https://registry.example.com

# Search registry packs
tsx pattern search "service layer"
tsx pattern search "form" --framework tanstack-start

# Eject: remove generated files and reverse marker injections
tsx pattern eject my-service

# Update installed packs from source
tsx pattern update
tsx pattern update my-service

# Publish via legacy share command
tsx pattern share --name my-service --version 1.0.0
```

---

## Snapshot Testing

### `tsx snapshot`

Save generator outputs as snapshots and diff against them to detect regressions.

```bash
# Run all generators with fixture inputs and save snapshots
tsx snapshot update
tsx snapshot update --generator add-schema

# Diff current output against saved snapshots
tsx snapshot diff
tsx snapshot diff --generator add-schema

# Accept current output as new baseline
tsx snapshot accept
tsx snapshot accept --generator add-schema

# List all registered snapshot fixtures
tsx snapshot list

# Register a new fixture
tsx snapshot add --generator add-schema --fixture users --input '{"name":"users"}'
```

---

## Authentication & Registry

### `tsx login / logout / whoami`

```bash
# Log in with an API key
tsx login --token <API_KEY>
tsx login --token <API_KEY> --registry https://registry.example.com

# Log out
tsx logout

# Show current user and registry
tsx whoami
```

---

### `tsx pkg / package`

Two complementary commands for registry packages.

**`tsx pkg`** — end-user package management (install, upgrade, publish):

```bash
tsx pkg install auth-form
tsx pkg install auth-form --version 1.2.0
tsx pkg install auth-form --target .tsx/packages/

tsx pkg info auth-form
tsx pkg upgrade auth-form

tsx pkg publish
tsx pkg publish --name my-pkg --version 1.0.0 --dry-run
```

**`tsx package`** — package authoring (scaffold, validate, pack, publish):

```bash
tsx package new my-framework
tsx package new my-framework --out ./packages/my-framework

tsx package validate
tsx package validate ./my-framework

tsx package pack
tsx package pack ./my-framework --out my-framework-1.0.0.tgz

tsx package publish
tsx package publish --registry https://registry.example.com --token <API_KEY>
tsx package install my-framework
```

---

### `tsx publish`

Publish or validate a registry manifest.

```bash
# Validate and publish a registry.json (prints to stdout)
tsx publish registry --registry registry.json

# Write published output to a file
tsx publish registry --registry registry.json --output published.json

# List registries installed in .tsx/frameworks/
tsx publish list
```

---

## Stack Management

### `tsx stack`

Manage the project stack profile (`.tsx/stack.json`).

```bash
# Auto-detect from package.json and write stack.json
tsx stack init

# Override detected language
tsx stack init --lang typescript

# Specify packages explicitly
tsx stack init --packages tanstack-start,drizzle-pg,better-auth

# Print current stack profile
tsx stack show

# Add a package to the active stack
tsx stack add better-auth
tsx stack add shadcn

# Remove a package
tsx stack remove better-auth

# Detect from project files and suggest packages
tsx stack detect

# Auto-install detected packages
tsx stack detect --install
```

---

## Plugin System

### `tsx plugin`

Manage installed template plugins.

```bash
tsx plugin list
tsx plugin install --source ./my-plugin
tsx plugin install --source @my-org/tsx-plugin-stripe
tsx plugin remove  --package @my-org/tsx-plugin-stripe
```

---

### `tsx template`

Manage reusable template bundles in `~/.tsx/templates/`.

```bash
# List installed templates
tsx template list
tsx template list --source global        # global only
tsx template list --source project       # project only
tsx template list --source framework     # framework templates

# Show details
tsx template info my-forms

# Scaffold a new bundle
tsx template init my-forms
tsx template init my-forms --dest ./templates/my-forms

# Install from a local directory
tsx template install ./my-forms

# Uninstall
tsx template uninstall my-forms

# Get JSON Schema for a template command (agent autocomplete)
tsx template schema my-forms form

# Lint .forge / .jinja files in a bundle
tsx template lint ./my-forms

# Config management
tsx template config show
tsx template config set registry_url https://registry.example.com
tsx template config init
tsx template config init --overwrite

# Auth
tsx template login --token <API_KEY>
tsx template login --token <API_KEY> --registry https://registry.example.com
tsx template logout

# Publish a bundle
tsx template publish --name my-forms --version 1.0.0
tsx template publish --name my-forms --version 1.0.0 --path ./my-forms
```

---

## Developer Tools

### `tsx upgrade`

Check and update atom versions or the tsx CLI binary.

```bash
# Pin atom templates to current versions
tsx upgrade atoms
tsx upgrade atoms --check    # report without writing

# Check for a newer tsx binary
tsx upgrade cli
tsx upgrade cli --check      # print latest version without downloading
```

---

### `tsx codegen`

Generate TypeScript interfaces and Zod schemas from Rust/OpenAPI/Drizzle sources.

```bash
# Parse Rust structs/enums → TypeScript interfaces + Zod schemas
tsx codegen rust-to-ts
tsx codegen rust-to-ts --input crates/shared/src/lib.rs
tsx codegen rust-to-ts --input crates/shared/src/lib.rs --out generated/types.ts
tsx codegen rust-to-ts --input crates/shared/src/lib.rs --watch

# Convert OpenAPI spec → Zod schemas
tsx codegen openapi-to-zod --spec openapi.yaml
tsx codegen openapi-to-zod --spec https://api.example.com/openapi.json --out generated/api.ts

# Auto-run drizzle-zod across all schema files
tsx codegen drizzle-to-zod
```

---

### `tsx analyze`

Scan project structure and report health/convention issues.

```bash
tsx analyze
tsx analyze --fix              # auto-apply safe fixes
tsx analyze --report           # structured JSON for CI
```

---

### `tsx plan`

Translate natural-language goals into a concrete command sequence.

```bash
tsx plan --json '[{"goal":"add a users schema with email and role"},{"goal":"add authentication"}]'
echo '[{"goal":"scaffold a products CRUD feature"}]' | tsx plan --stdin
```

---

### `tsx context`

Print agent-ready context: active stack, available commands, and usage summary. Use this as an AI agent preamble.

```bash
tsx context
tsx context --verbose
```

---

### `tsx repl`

Interactive goal-driven REPL.

```bash
# Interactive mode
tsx repl

# One-shot goal (agent mode — skips interactive loop)
tsx repl --goal "add a users schema with email and created_at"

# Execute proposed commands without prompting
tsx repl --goal "add authentication" --execute
```

---

### `tsx replay`

Record and replay generation sessions.

```bash
# Start recording
tsx replay record

# Record to specific file
tsx replay record --out .tsx/sessions/my-session.json

# Stop recording and save
tsx replay record --stop

# Replay a session
tsx replay run .tsx/sessions/my-session.json

# Dry-run replay
tsx replay run .tsx/sessions/my-session.json --dry-run

# List recorded sessions
tsx replay list
```

---

### `tsx atoms`

Queryable catalog of atoms and molecules for the active framework.

```bash
# List all atoms and molecules
tsx atoms list

# Filter by category
tsx atoms list --category drizzle
tsx atoms list --category form
tsx atoms list --category zod
tsx atoms list --category query

# Show raw template source for an atom
tsx atoms preview drizzle/column
tsx atoms preview form/field_input
```

---

### `tsx doctor`

Run diagnostic checks on the current project and environment.

```bash
tsx doctor
```

Checks include: Rust toolchain version, project layout, `package.json` presence, tsx version, template integrity.

---

### `tsx fmt`

Format `.forge` / `.jinja` template files (normalise indent, quotes, spacing).

```bash
# Format all template files in current directory
tsx fmt

# Format specific paths
tsx fmt templates/ src/

# Check only — exit 1 if any file needs formatting, don't write
tsx fmt --check

# Custom options
tsx fmt --indent 4 --quotes single
```

---

### `tsx lint-template`

Lint `.forge` / `.jinja` template files for common errors.

```bash
tsx lint-template
tsx lint-template ./templates
tsx lint-template ./my-feature.forge
```

---

### `tsx env`

Validate or diff `.env` files.

```bash
# Validate .env against .env.schema
tsx env check
tsx env check --schema .env.schema --env .env

# Show vars in .env.example missing from .env
tsx env diff
tsx env diff --example .env.example --env .env
```

---

### `tsx config`

Manage global tsx configuration (`~/.tsx/config.json`).

```bash
tsx config list
tsx config get registry_url
tsx config set registry_url https://registry.example.com
tsx config reset registry_url
tsx config reset                    # reset all keys to defaults
```

---

### `tsx docs`

Browse offline documentation from `.tsx/knowledge/` in a terminal UI.

```bash
# Launch interactive TUI browser
tsx docs

# Add extra directories
tsx docs ./my-docs ./project-knowledge

# Filter topics by keyword (no TUI, prints matching titles)
tsx docs --search "authentication"

# Emit topic list as JSON
tsx docs --json
```

---

### `tsx tui`

Launch the ratatui terminal dashboard (registry browser, doctor, stack editor).

```bash
tsx tui                      # default: browser view
tsx tui --view browser
tsx tui --view doctor
tsx tui --view stack
```

---

### `tsx completions`

Generate shell completion scripts.

```bash
tsx completions bash       >> ~/.bash_completion
tsx completions zsh        >> ~/.zshrc
tsx completions fish       > ~/.config/fish/completions/tsx.fish
tsx completions powershell >> $PROFILE
tsx completions elvish     >> ~/.config/elvish/rc.elv
```

---

### `tsx lsp`

Start the Language Server (LSP) for `.tsx/` config and `.forge` template files. Typically invoked by your editor plugin, not directly.

```bash
tsx lsp
```

---

### `tsx mcp`

Start the MCP (Model Context Protocol) server over stdio. Used by AI agents that speak the Model Context Protocol.

```bash
tsx mcp
```

---

## Utility Commands

### `tsx path`

Add a directory to the system PATH.

```bash
# Add current directory (session-only)
tsx path

# Add specific directory (session-only)
tsx path /usr/local/bin

# Persist permanently
#   Windows: uses setx /M PATH (requires Administrator)
#   Unix:    appends export line to ~/.zshrc or ~/.bashrc
tsx path . --permanent
tsx path /usr/local/bin --permanent

# List current PATH entries
tsx path --list
```

**Notes:**
- Without `--permanent`, the change only affects the current process session.
- On Windows with `--permanent`, setx has a 1024-character PATH limit.

---

### `tsx adb`

Android Debug Bridge management.

```bash
# Kill the ADB server
tsx adb kill

# Start the ADB server
tsx adb start

# Show ADB status and list connected devices
tsx adb status

# Reverse a port from device to host (default: 3333)
tsx adb reverse
tsx adb reverse --port 8080

# Execute arbitrary adb command
tsx adb exec devices
tsx adb exec devices -l
tsx adb exec shell ls /sdcard
```

Requires Android SDK (`adb`) installed and on PATH.

---

### `tsx flutter`

Flutter development commands.

```bash
# Run app in profile mode (default)
tsx flutter run

# Run in debug mode
tsx flutter run --mode debug

# Run on specific device
tsx flutter run --mode debug --device emulator-5554

# Run on custom port
tsx flutter run --port 8080

# Run in release mode
tsx flutter run --mode release

# Build APK
tsx flutter build --target apk

# Build app bundle
tsx flutter build --target appbundle

# Build in release mode
tsx flutter build --target apk --release

# Clean build artifacts
tsx flutter clean

# Get packages
tsx flutter pub-get
```

Requires Flutter SDK installed and on PATH.

---

### `tsx port`

Find and kill processes using a specific port.

```bash
# Find processes using port 8080
tsx port find --port 8080

# Kill all processes using port 3000
tsx port kill --port 3000
```

**Platform implementation:**
- **Windows**: uses `netstat -ano | findstr :<port>` and `taskkill /F /PID <pid>`
- **Unix/macOS**: uses `lsof -ti :<port>` and `kill -9 <pid>`
