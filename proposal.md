# TSX — Universal Code Pattern Registry & Generation Protocol

**Version:** 2.0 (Vision Proposal)
**Date:** 2026-03-16
**Status:** Draft — Design & Roadmap

---

## 1. The Problem We Are Solving

### How AI Agents Build Apps Today (The Broken Loop)

When a developer asks an AI agent to build a Todo web app using TanStack Start, Better Auth, TanStack Form, shadcn/ui, and Drizzle ORM with PostgreSQL, the agent does this:

1. **Guesses folder structure** — maybe `src/`, maybe `app/`, maybe flat root. Probably wrong.
2. **Hallucdinates imports** — `import { useForm } from "@tanstack/react-form"` with wrong version, wrong API surface.
3. **Writes schema from memory** — may use Drizzle v0.28 syntax when the project uses v0.36.
4. **Ignores existing conventions** — the developer has a style, a structure, patterns they use on every project. The agent ignores all of it.
5. **Burns thousands of tokens on repetitive boilerplate** — a single CRUD feature is 400–800 tokens to write. The agent writes the same patterns over and over, session after session.

**This is wasteful, inconsistent, and brittle.** The agent is doing the job of a code generator while pretending to be a thoughtful developer.

### The Root Cause

There is no contract between the developer's chosen stack and the tools the agent uses. The developer knows exactly how a `todo` feature should be built in their stack — the correct Drizzle schema syntax, the exact import path for TanStack Query, the shadcn form pattern they use. The agent has none of this.

**The fix is not a smarter agent. The fix is a better contract.**

---

## 2. The Vision

**tsx** is a universal code pattern registry and generation protocol.

It is a CLI that framework authors, library authors, and developers can extend to teach agents exactly how to write code for any framework, in any language, for any runtime.

The agent does not write boilerplate. The agent calls `tsx`.

```
Agent: "Build a todo feature"

→ tsx run add:feature --json '{"name":"todo","fields":[
    {"name":"title","type":"string","required":true},
    {"name":"done","type":"boolean","default":false}
  ]}'

← 7 files generated. Correct imports. Correct patterns. 40 tokens used.
```

tsx is not a TanStack tool. tsx is not a TypeScript tool. tsx is a protocol.

A framework author writes their patterns once. Every agent that uses tsx gets those patterns for free.

---

## 3. Core Concepts

### 3.1 Framework Package

A **Framework Package** is a self-contained bundle published to the tsx registry that contains:

- **Generator specs** — JSON Schema-driven command definitions
- **Forge templates** — The actual code patterns (language-agnostic template files)
- **Knowledge files** — Conventions, guides, integration notes in Markdown
- **Manifest** — Package metadata, dependencies, compatibility declarations
- **Slots** — Extension points that other packages can fill

Packages are named by convention: `@tsx-pkg/<framework>-<integration>` or `@tsx-pkg/<framework>`.

```
Examples:
  @tsx-pkg/tanstack-start        — TanStack Start core patterns
  @tsx-pkg/drizzle-pg            — Drizzle ORM with PostgreSQL
  @tsx-pkg/better-auth           — Better Auth integration
  @tsx-pkg/shadcn                — shadcn/ui component patterns
  @tsx-pkg/fastapi-sqlalchemy    — Python FastAPI + SQLAlchemy
  @tsx-pkg/axum-sea-orm          — Rust Axum + SeaORM
  @tsx-pkg/gin-gorm              — Go Gin + GORM
  @tsx-pkg/laravel               — PHP Laravel patterns
```

### 3.2 Stack Profile

A **Stack Profile** is the project-level configuration at `.tsx/stack.json`. It declares which packages are active for this project.

```json
{
  "version": "1",
  "lang": "typescript",
  "runtime": "node",
  "packages": [
    "tanstack-start@1.2.0",
    "drizzle-pg@0.36.0",
    "better-auth@1.0.0",
    "shadcn@2.0.0"
  ],
  "style": {
    "quotes": "double",
    "indent": 2,
    "semicolons": false
  },
  "paths": {
    "components": "src/components",
    "routes": "src/routes",
    "db": "src/db",
    "server_fns": "src/server-functions",
    "hooks": "src/hooks"
  }
}
```

When the agent runs any `tsx` command in this project, tsx reads this file and routes the command to the correct package, rendering templates with the correct path conventions and style settings.

### 3.3 Generator Spec

A **Generator Spec** is a JSON file that defines one command. It is the contract between the agent and the template engine.

```json
{
  "id": "add-feature",
  "command": "add:feature",
  "description": "Scaffold a complete CRUD feature",
  "token_estimate": 40,
  "package": "tanstack-start",
  "output_paths": [
    "db/schema/{{name}}.ts",
    "server-functions/{{name}}.ts",
    "hooks/use-{{name}}.ts",
    "components/{{name}}Form.tsx",
    "components/{{name}}Table.tsx",
    "routes/{{name}}/index.tsx",
    "routes/{{name}}/$id.tsx"
  ],
  "next_steps": [
    "Run `tsx run add:migration --json '{\"name\":\"add-{{name}}\"}'`",
    "Add `<Link to=\"/{{name}}\" />` to your navigation"
  ],
  "schema": {
    "type": "object",
    "required": ["name"],
    "properties": {
      "name": { "type": "string", "description": "Entity name (e.g. 'todo', 'product')" },
      "fields": {
        "type": "array",
        "items": {
          "type": "object",
          "required": ["name", "type"],
          "properties": {
            "name": { "type": "string" },
            "type": { "type": "string", "enum": ["string", "number", "boolean", "date", "uuid"] },
            "required": { "type": "boolean", "default": false },
            "default": {}
          }
        }
      },
      "auth": { "type": "boolean", "default": false, "description": "Protect with auth guard" }
    }
  }
}
```

The schema is the only thing the agent needs to know. tsx handles everything else.

### 3.4 Forge Template

A **Forge Template** is the actual code pattern. It uses the Tera/Jinja2 syntax with tsx-specific helpers.

```typescript
// templates/add-feature/server-functions/{{name}}.ts.forge
import { createServerFn } from "@tanstack/start";
import { db } from "~/db";
import { {{ name }}Table } from "~/db/schema/{{ name }}";
{% if auth %}
import { requireAuth } from "~/lib/auth";
{% endif %}

export const list{{ name | pascal_case }} = createServerFn({ method: "GET" })
  .handler(async ({ context }) => {
    {% if auth %}await requireAuth(context);{% endif %}
    return db.select().from({{ name }}Table);
  });

export const create{{ name | pascal_case }} = createServerFn({ method: "POST" })
  .validator((data: Create{{ name | pascal_case }}Input) => data)
  .handler(async ({ data{% if auth %}, context{% endif %} }) => {
    {% if auth %}await requireAuth(context);{% endif %}
    const [result] = await db.insert({{ name }}Table).values(data).returning();
    return result;
  });
```

Templates are entirely defined by the framework package author. The tsx engine only renders them — it does not impose any language or pattern.

### 3.5 Composition via Slots

When multiple packages are installed together, they can compose. A **Slot** is an extension point in a template that another package can fill.

```
# tanstack-start defines a slot in its schema template:
{{ slot("auth_fields") }}    ← better-auth package fills this

# better-auth's slot content:
userId: text("user_id").references(() => usersTable.id)
```

The tsx engine resolves which packages are active and fills slots at render time. This means:

- `drizzle-pg` + `better-auth` → schema automatically gets `userId` field
- `tanstack-start` + `shadcn` → form template uses shadcn Input/Button instead of raw HTML
- `drizzle-pg` + `zod` → validation schema auto-generated from Drizzle column types

The developer configures once. Every generated file reflects the full stack.

---

## 4. The Agent Protocol

### 4.1 Discovery

Before building anything, the agent queries what's available:

```bash
# What stack is configured for this project?
tsx context --json

# What commands can I call right now?
tsx list --json

# What does a specific command do?
tsx describe add:feature --json
```

**`tsx context --json`** output:

```json
{
  "stack": {
    "lang": "typescript",
    "packages": ["tanstack-start", "drizzle-pg", "better-auth", "shadcn"]
  },
  "commands": 24,
  "knowledge_topics": ["routing", "auth", "forms", "database", "deployment"],
  "summary": "TanStack Start project with Drizzle/PostgreSQL, Better Auth, and shadcn/ui"
}
```

**`tsx list --json`** output:

```json
{
  "commands": [
    {
      "id": "add-feature",
      "command": "add:feature",
      "package": "tanstack-start",
      "description": "Scaffold a complete CRUD feature module",
      "token_estimate": 40,
      "required_inputs": ["name"]
    },
    {
      "id": "add-schema",
      "command": "add:schema",
      "package": "drizzle-pg",
      "description": "Create a Drizzle ORM table schema",
      "token_estimate": 30,
      "required_inputs": ["name"]
    }
  ]
}
```

The agent now has a complete map of what it can do — without hallucinating.

### 4.2 Execution

The agent calls commands with a JSON payload:

```bash
# Full feature scaffold
tsx run add:feature --json '{
  "name": "todo",
  "fields": [
    {"name": "title", "type": "string", "required": true},
    {"name": "done", "type": "boolean", "default": false},
    {"name": "dueDate", "type": "date"}
  ],
  "auth": true
}'
```

**Response:**

```json
{
  "ok": true,
  "command": "add:feature",
  "files": [
    {"path": "db/schema/todo.ts", "action": "created", "bytes": 312},
    {"path": "server-functions/todo.ts", "action": "created", "bytes": 680},
    {"path": "hooks/use-todo.ts", "action": "created", "bytes": 445},
    {"path": "components/TodoForm.tsx", "action": "created", "bytes": 820},
    {"path": "components/TodoTable.tsx", "action": "created", "bytes": 760},
    {"path": "routes/todo/index.tsx", "action": "created", "bytes": 290},
    {"path": "routes/todo/$id.tsx", "action": "created", "bytes": 370}
  ],
  "next_steps": [
    "Run `tsx run add:migration --json '{\"name\":\"add-todo\"}'`",
    "Add `<Link to=\"/todo\" />` to your navigation"
  ],
  "tokens_used": 42
}
```

The agent does not write a single line of boilerplate. It follows the `next_steps` and moves on.

### 4.3 Dry Run (Planning Mode)

Before executing, the agent can preview what will be generated:

```bash
tsx run add:feature --json '{"name":"todo"}' --dry-run
```

```json
{
  "ok": true,
  "dry_run": true,
  "would_create": [
    "db/schema/todo.ts",
    "server-functions/todo.ts",
    "hooks/use-todo.ts",
    "components/TodoForm.tsx",
    "components/TodoTable.tsx",
    "routes/todo/index.tsx",
    "routes/todo/$id.tsx"
  ],
  "token_estimate": 42
}
```

### 4.4 Batch Execution

The agent can plan and execute an entire feature set atomically:

```bash
tsx batch --json '[
  {"command": "add:auth-setup", "args": {"provider": "github"}},
  {"command": "add:feature", "args": {"name": "todo", "auth": true}},
  {"command": "add:feature", "args": {"name": "list", "auth": true}},
  {"command": "add:migration", "args": {"name": "initial"}}
]'
```

If any command fails, all previously written files are rolled back. The project stays in a clean state.

---

## 5. The Todo App Walkthrough

**Goal:** Build a Todo web app with TanStack Start, Better Auth (GitHub OAuth), shadcn/ui, Drizzle ORM + PostgreSQL.

### Step 1: Initialize the stack

```bash
tsx init --stack tanstack-start,drizzle-pg,better-auth,shadcn
```

This installs the tsx packages for those frameworks, creates `.tsx/stack.json`, and scaffolds the project boilerplate (env files, DB connection, auth config).

**Agent tokens used: ~80** (one-time project setup)

### Step 2: Agent builds the app

The agent runs `tsx list --json` once and gets the full command map. Then:

```bash
# Wire up GitHub OAuth
tsx run add:auth-setup --json '{"provider": "github", "session_table": true}'
# → auth.ts, session schema, env template, middleware  (45 tokens)

# Build todo feature
tsx run add:feature --json '{
  "name": "todo",
  "fields": [
    {"name": "title", "type": "string", "required": true},
    {"name": "done", "type": "boolean", "default": false}
  ],
  "auth": true
}'
# → 7 files, correct drizzle schema, tanstack query hooks, shadcn form  (42 tokens)

# Create and run migration
tsx run add:migration --json '{"name": "initial-schema"}'
# → migration file, run it  (20 tokens)

# Add navigation entry
tsx run add:nav-link --json '{"label": "Todos", "path": "/todo", "icon": "CheckSquare"}'
# → updates existing nav component  (15 tokens)
```

**Total agent tokens: ~200** for a fully functional, correctly structured Todo app.

**Without tsx:** The agent writes all patterns from memory — 3,000–5,000 tokens, inconsistent structure, wrong imports, hallucinated APIs.

---

## 6. Language & Runtime Agnosticism

tsx is not a JavaScript tool. It is a code pattern protocol. The same CLI, the same `tsx run` command, the same agent protocol — for any language.

### Python / FastAPI + SQLAlchemy

```bash
# Developer installs:
tsx registry install @tsx-pkg/fastapi-sqlalchemy @tsx-pkg/pydantic-v2

# Agent runs:
tsx run add:model --json '{"name": "todo", "fields": [...]}'
# → models/todo.py  (SQLAlchemy model, Pydantic schema)

tsx run add:router --json '{"name": "todo", "auth": true}'
# → routers/todo.py  (FastAPI router with CRUD endpoints, auth dependency)
```

### Rust / Axum + SeaORM

```bash
tsx registry install @tsx-pkg/axum-sea-orm

tsx run add:entity --json '{"name": "todo", "fields": [...]}'
# → entity/todo.rs  (SeaORM entity)

tsx run add:handler --json '{"name": "todo", "auth": true}'
# → handlers/todo.rs  (Axum handler, routes, middleware)
```

### Go / Gin + GORM

```bash
tsx registry install @tsx-pkg/gin-gorm

tsx run add:model --json '{"name": "Todo", "fields": [...]}'
# → models/todo.go  (GORM model)

tsx run add:controller --json '{"name": "todo"}'
# → controllers/todo.go  (Gin controller, route registration)
```

The Framework Package Format is identical. Only the templates differ.

---

## 7. Framework Package Format (FPF v1.1)

```
my-package/
│
├── manifest.json              # Package metadata and declarations
│
├── generators/                # One .json per command
│   ├── add-model.json
│   ├── add-controller.json
│   └── add-feature.json
│
├── templates/                 # Forge templates
│   ├── add-model/
│   │   └── models/{{name}}.ts.forge
│   ├── add-controller/
│   │   └── controllers/{{name}}.ts.forge
│   └── add-feature/
│       ├── models/{{name}}.ts.forge
│       └── controllers/{{name}}.ts.forge
│
├── knowledge/                 # Markdown docs for `tsx ask`
│   ├── conventions.md
│   ├── folder-structure.md
│   └── integrations/
│       ├── with-auth.md
│       └── with-database.md
│
└── slots/                     # Extension points
    ├── schema-extra-fields.forge
    └── form-extra-imports.forge
```

### manifest.json

```json
{
  "name": "@tsx-pkg/tanstack-start",
  "version": "1.2.0",
  "description": "TanStack Start code patterns for tsx",
  "lang": ["typescript"],
  "runtime": ["node", "bun", "deno"],
  "tsx_min": "0.2.0",
  "provides": ["add:feature", "add:page", "add:server-fn", "add:query"],
  "integrates_with": {
    "drizzle-pg": "slots/with-drizzle.forge",
    "better-auth": "slots/with-auth.forge",
    "shadcn": "slots/with-shadcn.forge"
  },
  "homepage": "https://tanstack.com/start",
  "repository": "https://github.com/tanstack/tsx-package"
}
```

### Publishing

```bash
# Author tools
tsx framework init              # scaffold FPF structure
tsx framework validate          # lint manifest, validate schemas, test templates
tsx framework preview           # render all templates with example inputs
tsx framework publish           # push to registry.tsx.dev
```

---

## 8. Stack Composition Engine

When multiple packages are installed together, tsx resolves how they compose.

### Example: tanstack-start + drizzle-pg + better-auth

**Without composition** — agent runs `add:schema` and gets a basic schema:

```typescript
export const todosTable = pgTable("todos", {
  id: serial("id").primaryKey(),
  title: text("title").notNull(),
  done: boolean("done").default(false)
});
```

**With composition** — `better-auth` slot fills in auth fields:

```typescript
export const todosTable = pgTable("todos", {
  id: serial("id").primaryKey(),
  title: text("title").notNull(),
  done: boolean("done").default(false),
  // → filled by better-auth slot
  userId: text("user_id").notNull().references(() => usersTable.id),
  createdAt: timestamp("created_at").defaultNow()
});
```

**With shadcn composition** — `add:form` uses shadcn components instead of raw HTML:

```tsx
// Without shadcn: raw <input> elements
// With shadcn:
import { Input } from "~/components/ui/input"
import { Button } from "~/components/ui/button"
import { Label } from "~/components/ui/label"
```

The developer installs the packages once. Every generated file automatically reflects the full stack.

---

## 9. The Registry

### 9.1 Architecture

```
Developer/Author
    │
    │ tsx framework publish
    ▼
registry.tsx.dev
    │
    │ stores package tarballs + manifests
    │ indexes by lang, runtime, provides[], integrates_with
    ▼
tsx CLI (local)
    │
    │ tsx registry install @tsx-pkg/drizzle-pg
    │ extracts to .tsx/packages/drizzle-pg/
    ▼
Stack Profile (.tsx/stack.json)
    │
    │ declares active packages
    ▼
CommandRegistry (runtime)
    │ scans .tsx/packages/ at startup
    │ indexes all generators by id + command
    ▼
Agent calls: tsx run add:schema
```

### 9.2 Search and Discovery

```bash
tsx registry search drizzle
# → @tsx-pkg/drizzle-pg, @tsx-pkg/drizzle-mysql, @tsx-pkg/drizzle-sqlite
#   community: @myorg/drizzle-patterns

tsx registry info @tsx-pkg/drizzle-pg
# → version, commands, integrations, install count, readme

tsx registry install @tsx-pkg/drizzle-pg
# → downloads, validates, installs to .tsx/packages/

tsx registry update
# → checks all installed packages for newer versions

tsx registry list
# → all packages installed in this project
```

### 9.3 Offline Mode

tsx caches all installed package manifests and templates locally. Commands work fully offline. Registry is only needed for `install` and `update`.

---

## 10. Developer Experience

### Project Setup (30 seconds)

```bash
# New project
tsx init my-app --stack tanstack-start,drizzle-pg,better-auth,shadcn

# Existing project — detect and configure
cd my-existing-project
tsx stack detect      # reads package.json, go.mod, Cargo.toml, requirements.txt
# → detected: tanstack-start 1.2, drizzle-orm 0.36, better-auth 1.0
# → found packages: @tsx-pkg/tanstack-start, @tsx-pkg/drizzle-pg, @tsx-pkg/better-auth
# → install all? [Y/n]
```

### Agent Setup (one prompt)

At the start of an agent session, the developer gives the agent context:

```bash
tsx context
```

Output (designed to paste into agent system prompt):

```
This project uses tsx for code generation.
Stack: TypeScript / TanStack Start / Drizzle ORM (PostgreSQL) / Better Auth / shadcn/ui
Available commands (run `tsx list --json` for schemas):
  add:feature — scaffold complete CRUD module (40 tokens)
  add:schema  — Drizzle table schema (30 tokens)
  add:page    — route page component (20 tokens)
  add:auth-guard — protect a route (15 tokens)
  add:migration  — database migration (20 tokens)
  + 19 more

Always use tsx commands instead of writing boilerplate manually.
Call `tsx describe <command> --json` before using an unfamiliar command.
```

The agent now has a contract. It knows exactly what tools are available and how to call them.

---

## 11. What Does Not Change

The following parts of tsx remain as-is and are not in scope for this redesign:

- **Forge engine** (`tsx-forge` crate) — template rendering is already language-agnostic
- **CommandRegistry** — already scans framework directories dynamically
- **`tsx run` dispatcher** — already accepts JSON payload, validates schema, applies defaults
- **Batch execution with rollback** — already works
- **`tsx framework` author tools** — already scaffolds FPF structure, validates, previews
- **Agent-friendly JSON output** — already structured for machine consumption

The work is in **what surrounds** these systems: the registry infrastructure, the stack profile resolver, the composition/slot engine, and the first-party package library.

---

## 12. Implementation Roadmap

### Phase 1: Stack Profile System (Week 1–2)

**Goal:** Commands route based on installed packages, not hard-coded framework names.

- [ ] Define `.tsx/stack.json` schema and reader
- [ ] `tsx stack detect` — reads project deps and suggests packages
- [ ] `tsx stack add <package>` — adds package to stack profile
- [ ] CommandRegistry reads from `.tsx/packages/` in addition to builtin `frameworks/`
- [ ] Path conventions from `stack.json` respected in `output_paths` expansion

**Deliverable:** A project with `.tsx/stack.json` uses packages from `.tsx/packages/` automatically.

### Phase 2: Composition Engine (Week 3–4)

**Goal:** Multiple packages compose correctly in generated code.

- [ ] Slot declaration in manifests (`integrates_with`)
- [ ] Slot injection at render time based on active packages
- [ ] Style settings (`quotes`, `indent`, `semicolons`) applied to all output
- [ ] Path overrides from `stack.json` respected in all templates

**Deliverable:** Installing `better-auth` alongside `drizzle-pg` automatically adds userId fields to all schemas.

### Phase 3: Registry Infrastructure (Week 5–8)

**Goal:** Community can publish and install packages.

- [ ] `registry.tsx.dev` — hosted registry API (Rust/Axum backend)
- [ ] `tsx registry install/update/list/search` fully wired to real registry
- [ ] Package signing and verification
- [ ] `tsx framework publish` sends to registry
- [ ] CLI version compatibility check on install

**Deliverable:** A developer can publish `@myorg/my-patterns` and anyone can `tsx registry install` it.

### Phase 4: Reference Package Library (Week 9–12)

**Goal:** First-party packages for the most common stacks.

- [ ] `@tsx-pkg/tanstack-start` — complete rewrite of current builtin
- [ ] `@tsx-pkg/drizzle-pg`, `@tsx-pkg/drizzle-mysql`, `@tsx-pkg/drizzle-sqlite`
- [ ] `@tsx-pkg/better-auth`
- [ ] `@tsx-pkg/shadcn`
- [ ] `@tsx-pkg/fastapi-sqlalchemy` (Python reference implementation)
- [ ] `@tsx-pkg/axum-sea-orm` (Rust reference implementation)

**Deliverable:** The same `tsx run add:feature` workflow works for TypeScript, Python, and Rust projects.

### Phase 5: Agent Optimization (Week 13–14)

**Goal:** Agents get maximum signal with minimum tokens.

- [ ] `tsx context` — single command dumps full stack context for system prompt
- [ ] `tsx plan --json '[{"goal": "build todo feature"}]'` — agent-assisted command planning
- [ ] Token accounting in all responses
- [ ] `tsx batch --plan` — dry-run a full batch before executing

**Deliverable:** Agents can build feature-complete applications with under 500 tokens of framework overhead per session.

---

## 13. Success Metrics

| Metric | Today | Target |
| --- | --- | --- |
| Tokens to scaffold a CRUD feature | ~3,000 (agent writes it) | ~42 (tsx generates it) |
| Supported languages | 1 (TypeScript) | 5+ (TS, Python, Rust, Go, PHP) |
| Community packages | 0 | 20+ in year 1 |
| Agent adoption | Requires framework knowledge | Zero framework knowledge needed |
| Time to set up new project stack | Manual | `tsx init --stack <...>` in 30s |
| Consistency across sessions | Zero | 100% (templates don't drift) |

---

## 14. What tsx Is Not

- **tsx is not a code editor or IDE plugin.** It is a CLI.
- **tsx is not an AI model.** It generates code from deterministic templates.
- **tsx is not a package manager.** It manages code patterns, not runtime dependencies.
- **tsx is not framework-specific.** The current TanStack Start focus is a reference implementation.
- **tsx is not an npm alternative.** tsx packages contain patterns and templates, not runtime code.

tsx is the missing layer between "I know my stack" and "the agent knows my stack."

---

## Appendix: Current State

The tsx CLI already has:

- `tsx run <id> --json <payload>` — universal command dispatcher ✓
- `CommandRegistry` — dynamic generator loading from JSON files ✓
- Inline JSON Schema validation with defaults ✓
- Batch execution with rollback ✓
- `tsx framework` author tools ✓
- `tsx-forge` Tera-based template engine ✓
- Agent-friendly structured JSON output ✓

What is missing:

- Stack Profile system (`.tsx/stack.json`)
- Composition/slot engine for multi-package rendering
- Hosted registry infrastructure
- First-party packages beyond TanStack Start
- `tsx context` agent onboarding command
- Language-agnostic reference packages
