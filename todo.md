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
- [ ] **`integrates_with` slot injection** — at render time, read `manifest.integrates_with`, check which peer packages are installed, inject slot content from each
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

- [ ] **`@tsx-pkg/tanstack-start`** — extract + rewrite current `frameworks/tanstack-start/` as installable package
- [x] **`@tsx-pkg/drizzle-pg`** — Drizzle ORM PostgreSQL generators: `add:schema`, `add:migration`, `add:seed` (`frameworks/drizzle-pg/`)
- [x] **`@tsx-pkg/better-auth`** — Better Auth generators: `add:auth-setup`, `add:auth-guard`, `add:session` (`frameworks/better-auth/`)
- [ ] **`@tsx-pkg/shadcn`** — shadcn/ui form/table/dialog generators
- [ ] **`@tsx-pkg/fastapi-sqlalchemy`** — Python FastAPI + SQLAlchemy reference package
- [ ] **`@tsx-pkg/axum-sea-orm`** — Rust Axum + SeaORM reference package

---

## Phase 5 — Agent Optimization

_Goal: Agents get maximum signal with minimum tokens._

- [x] **`tsx context`** — single command dumps full stack context for agent system prompt
  - Output: stack summary, active packages, available commands with token estimates, human-readable `summary` string
- [ ] **`tsx plan --json '[{"goal":"..."}]'`** — translate natural-language goals into a command sequence
- [ ] **Token accounting** — all responses include `tokens_used` (already on run, extend to all commands)
- [x] **`tsx batch --plan`** — resolves all commands against the registry, returns `would_create` paths + token estimates per step without executing

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
