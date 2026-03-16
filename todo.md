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
- [x] **Hosted registry `registry.tsx.dev`** — Rust/Axum backend server (`crates/registry-server/`)
  - `GET  /health` — health check
  - `GET  /v1/search?q=&lang=&size=` — full-text search across name, description, provides[]
  - `GET  /v1/packages/:name` — full package metadata + version history + parsed manifest
  - `GET  /v1/packages/:name/:version/tarball` — stream .tar.gz, increment download counter
  - `POST /v1/packages/publish` — multipart upload (name, version, manifest JSON, tarball); `Authorization: Bearer <TSX_REGISTRY_API_KEY>`
  - SQLite storage via rusqlite (packages + versions tables, WAL mode)
  - `PORT`, `DATA_DIR`, `TSX_REGISTRY_API_KEY` env config
  - Run: `cargo run -p tsx-registry`
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

---

## Phase 8 — Registry Hardening & CLI-Server Integration

_Goal: The CLI actually talks to `registry.tsx.dev`, search works correctly, and the server is production-ready._

- [x] **Fix `iso_now()`** — correct leap-year and month calendar math in both `crates/registry-server/src/db.rs` and `src/commands/ops/registry.rs`
- [x] **`TSX_REGISTRY_URL` env var** — all `tsx registry *` commands prefer `$TSX_REGISTRY_URL/v1/...` over hardcoded npm/unpkg; falls back to npm when unset
- [x] **Tarball extraction on install** — `tsx registry install` downloads `.tar.gz` from registry server and extracts to `.tsx/packages/<slug>/`
- [x] **`tsx_min` version compat check** — `registry_install` reads `manifest.tsx_min`, rejects install if CLI version is older
- [x] **Fix search `latest_version` stub** — `GET /v1/search` returns the real latest semver version via `search_with_latest()` DB method
- [x] **Sort versions by semver** — `get_versions()` sorts by parsed semver DESC, not `published_at`
- [x] **Fix lang filter** — parse `lang` JSON column properly in Rust so `?lang=rust` never false-matches `trust`
- [x] **WAL + busy_timeout** — `Mutex<Db>` + `PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;` in registry server migrations
- [x] **Rate limiting on publish** — `POST /v1/packages/publish` returns HTTP 429 after 10 req/min per IP (per-IP token bucket in `AppState`)
- [x] **`tsx framework publish --registry <url> --api-key <key>`** — multipart-upload to custom registry instead of only `npm publish`

---

## Phase 9 — Web Platform & Examples

_Goal: A complete web presence — registry dashboard, docs site, and installable example projects._

- [x] **`apps/registry-web/`** — TanStack Start registry web app (port 3000)
  - Landing page: hero, install command, stats cards, recent packages grid
  - `/browse` — searchable package index with lazy loading
  - `/packages/$name` — package detail with versions table, install command, meta sidebar
  - `/_protected/dashboard/` — admin stats dashboard (Better Auth protected)
  - `src/lib/api.ts` — typed fetch client for Rust registry server
  - `src/lib/types.ts` — `Package`, `SearchResult`, `RegistryStats` types
  - `src/features/packages/hooks/` — react-query options for all registry endpoints
  - `src/integrations/tanstack-query/` — `getContext()` singleton + QueryClientProvider
  - sea-ink/lagoon CSS palette, `nav-link` + `island-shell` utility classes
  - Header/Footer/ThemeToggle with THEME_INIT_SCRIPT for flicker-free dark mode
- [x] **`apps/docs/`** — TanStack Start documentation site (port 3001)
  - Landing page with quick-nav cards (Getting Started / CLI / Registry)
  - Sidebar layout (`docs.tsx`) with nested routes
  - `Getting Started`, `CLI Reference`, `Registry API` doc pages
  - Same sea-ink/lagoon palette, ThemeProvider, Header/Footer
- [x] **`examples/basic-crud/`** — complete CRUD example (products, drizzle-pg, react-query)
- [x] **`examples/with-auth/`** — complete Better Auth example (auth server, client, middleware, dashboard)
- [x] **`examples/with-shadcn/`** — DataTable + feature-based hooks example (items)
- [x] **`examples/full-saas/`** — multi-org SaaS example (org + billing feature hooks, dashboard)

### Phase 9 — Pending / Next Up

- [ ] **`apps/registry-web/` — auth publish flow** — logged-in users can publish packages from the UI
- [ ] **`apps/registry-web/` — package README rendering** — fetch + render markdown from tarball
- [ ] **`apps/docs/` — FPF format docs** — `stack.json` reference, `output_paths`, slot system
- [ ] **`apps/docs/` — Examples gallery** — link to all examples with generated preview screenshots
- [ ] **Registry server: `GET /v1/stats`** — implement the stats endpoint in Axum (currently stubbed)
- [ ] **Registry server: `GET /v1/packages?sort=recent`** — recent packages endpoint for landing page
- [ ] **CI/CD** — GitHub Actions workflow: `cargo test`, `cargo build --release`, `bun install && bun run build` for both apps
- [ ] **Deploy** — Dockerfile for registry server + Fly.io config; Vercel/Netlify config for apps

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
