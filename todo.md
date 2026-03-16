# TSX Implementation Todo — v2.0 (Universal Code Pattern Registry)

Based on [proposal.md](proposal.md)

---

## Phase 1 — Stack Profile System

_Goal: Commands route based on installed packages, not hard-coded framework names._

- [x] `CommandRegistry` — dynamic generator loading from JSON files (`src/framework/command_registry.rs`)
- [x] `tsx run <id> --json <payload>` — universal dispatcher (`src/commands/ops/run.rs`)
- [x] Inline JSON Schema validation + defaults
- [x] **`src/stack/mod.rs`** — `StackProfile` struct with load/save/detect
  - Fields: `version`, `lang`, `runtime`, `packages[]`, `style{}`, `paths{}`
  - `StackProfile::load(dir)` — reads `.tsx/stack.json`
  - `StackProfile::save(dir)` — writes `.tsx/stack.json`
  - `StackProfile::detect(dir)` — infers from `package.json` / `Cargo.toml` / `go.mod` / `requirements.txt`
- [x] **`tsx stack` command** (`src/commands/ops/stack.rs`)
  - `tsx stack init [--lang ts] [--packages a,b,c]` — create `.tsx/stack.json`
  - `tsx stack show` — print current stack profile
  - `tsx stack add <package>` — append package to active profile
  - `tsx stack remove <package>` — remove package from active profile
  - `tsx stack detect` — auto-detect stack and print suggestions
- [x] **Update `CommandRegistry::load_all()`** — also scans `.tsx/packages/<pkg>/generators/`
- [x] **`frameworks/tanstack-start/manifest.json`** — FPF v1.1 manifest with `provides[]` + `integrates_with{}`
- [x] **Path override** — `output_paths` expansion respects `stack.json` path aliases (`components/`, `routes/`, `db/`, `server-functions/`, `hooks/`)

---

## Phase 2 — Composition Engine

_Goal: Multiple packages compose correctly in generated code._

- [x] Slot system in forge crate (`crates/forge/src/slots.rs`)
- [x] **`integrates_with` slot injection** — at render time, read `manifest.integrates_with`, check which peer packages are installed, inject slot content from each (rendered via tsx-forge)
- [x] **Style settings applied** — `stack.json` style (`quotes`, `indent`, `semicolons`) injected as `__style_*` vars into every generator's input context
- [x] **Path overrides** — `stack.json` `paths` map applied in `output_paths` expansion and `batch:plan`

---

## Phase 3 — Registry Infrastructure

_Goal: Community can publish and install packages._

- [x] **Wire `tsx registry search`** — queries npm registry for `tsx-framework-*` packages
- [x] **Wire `tsx registry install`** — FPF (`@tsx-pkg/`) packages: downloads `manifest.json` + generators to `.tsx/packages/<slug>/`; legacy packages: downloads `registry.json` to `.tsx/frameworks/<slug>/`
- [x] **Wire `tsx registry update`** — checks all installed packages against npm, reinstalls if newer version available
- [x] **`tsx registry list`** — lists both legacy registries (`.tsx/registries.json`) and FPF packages (`.tsx/packages/`)
- [ ] **Hosted registry `registry.tsx.dev`** (future — Rust/Axum backend, out of scope for CLI)
- [x] **`tsx framework publish`** — `npm publish --access public` wrapper with `@tsx-pkg/<id>` naming (was already complete)

---

## Phase 4 — Reference Package Library

_Goal: First-party packages for the most common stacks._

- [x] **`@tsx-pkg/tanstack-start`** — FPF v1.1 manifest finalized; generators list trimmed to files that exist (`frameworks/tanstack-start/`)
- [x] **`@tsx-pkg/drizzle-pg`** — Drizzle ORM PostgreSQL generators: `add:schema`, `add:migration`, `add:seed` (`frameworks/drizzle-pg/`)
- [x] **`@tsx-pkg/better-auth`** — Better Auth generators: `add:auth-setup`, `add:auth-guard`, `add:session` (`frameworks/better-auth/`)
- [x] **`@tsx-pkg/shadcn`** — shadcn/ui form/table/dialog generators (`frameworks/shadcn/`)
- [x] **`@tsx-pkg/fastapi-sqlalchemy`** — Python FastAPI + SQLAlchemy reference package (`frameworks/fastapi-sqlalchemy/`)
- [x] **`@tsx-pkg/axum-sea-orm`** — Rust Axum + SeaORM reference package (`frameworks/axum-sea-orm/`)

---

## Phase 5 — Agent Optimization

_Goal: Agents get maximum signal with minimum tokens._

- [x] **`tsx context`** — single command dumps full stack context for agent system prompt
  - Output: stack summary, active packages, available commands with token estimates, human-readable `summary` string
- [x] **`tsx plan --json '[{"goal":"..."}]'`** — translate natural-language goals into a command sequence (`src/commands/ops/plan.rs`)
- [x] **Token accounting** — `metadata.tokens_used` added to `ResponseEnvelope`; wired in `run` (per-generator estimate) and `batch`/`batch:plan` (summed)
- [x] **`tsx batch --plan`** — resolves all commands against the registry, returns `would_create` paths + token estimates per step without executing

---

---

## Phase 6 — FPF Forge Execution + Reference Templates

_Goal: `tsx run` actually generates files for every installed FPF package, not just tanstack-start._

- [x] **FPF forge execution path** — `fpf_execute()` fallback in `execute_command`: resolves via `CommandRegistry`, renders `templates/<generator-id>/N.forge` files (index-matched to `output_paths`), writes to disk (`src/commands/ops/batch.rs`)
- [x] **`frameworks/drizzle-pg/templates/`** — forge templates for `add:schema`, `add:migration`, `add:seed`
- [x] **`frameworks/better-auth/templates/`** — forge templates for `add:auth-setup` (3 files), `add:auth-guard`, `add:session`
- [x] **`frameworks/shadcn/templates/`** — forge templates for `add:ui-form`, `add:ui-data-table` (2 files), `add:ui-dialog`, `add:ui-button`, `add:ui-input`
- [x] **`frameworks/fastapi-sqlalchemy/templates/`** — forge templates for `add:model`, `add:router`, `add:schema`, `add:crud`
- [x] **`frameworks/axum-sea-orm/templates/`** — forge templates for `add:entity`, `add:handler`, `add:migration`, `add:service`
- [x] **`@tsx-pkg/gin-gorm`** — Go / Gin + GORM reference package (`frameworks/gin-gorm/`) with `add:model`, `add:controller`, `add:middleware` + templates
- [x] **Knowledge bases for new packages** — `knowledge/overview.md` + `knowledge/conventions.md` for `drizzle-pg`, `better-auth`, `shadcn`, `fastapi-sqlalchemy`, `axum-sea-orm`, `gin-gorm`
- [x] **`tsx registry info <package>`** — fetch version, description, provides[], integrates_with from npm + unpkg manifest (`src/commands/ops/registry.rs`)

---

---

## Phase 7 — Proposal Gap Closure

_Goal: Every feature described in proposal.md is implemented._

- [x] **`tsx describe <command-id>`** — per-generator description: positional `<TARGET>` arg resolves framework slug OR generator id/command, returns `id`, `command`, `framework`, `description`, `token_estimate`, `output_paths`, `schema`, `usage` (`src/commands/query/describe.rs`)
- [x] **`tsx list` agent mode** — `--kind` is now optional; omitting it returns all registry generators with `id`, `command`, `package`, `description`, `token_estimate`, `output_paths`, `required_inputs`, `usage` for agent discovery (`src/commands/ops/list.rs`)
- [x] **`tsx stack detect --install`** — `--install` flag auto-installs each detected package via `tsx registry install` (`src/commands/ops/stack.rs`)
- [x] **`tsx init --stack <packages>`** — `--stack` flag creates `.tsx/stack.json` inside the new project directory after scaffolding (`src/commands/manage/init.rs`)
- [x] **`@tsx-pkg/drizzle-mysql`** — MySQL variant: manifest, generators (`add:schema`, `add:migration`, `add:seed`), forge templates, knowledge (`frameworks/drizzle-mysql/`)
- [x] **`@tsx-pkg/drizzle-sqlite`** — SQLite/Turso/Bun variant: manifest, generators (`add:schema`, `add:migration`, `add:seed`), forge templates, knowledge (`frameworks/drizzle-sqlite/`)

---

## Completed (prior sessions)

- [x] `tsx run <id> --json` universal dispatcher
- [x] `CommandRegistry` scanning builtin `frameworks/` dirs
- [x] Inline JSON Schema validator + default applicator
- [x] Batch execution with atomic rollback
- [x] `tsx framework init/validate/preview/add/publish` author tools
- [x] Forge (Tera) engine crate (`tsx-forge`)
- [x] Agent-friendly structured JSON output with `ResponseEnvelope`
- [x] `tsx-forge` + `tsx` published to crates.io
- [x] Comprehensive README.md
