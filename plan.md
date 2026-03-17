# tsx — Architecture & Scalability Plan

> Written 2026-03-17. Software architecture + system design review across all
> languages, frameworks, and layers of the monorepo.

---

## 1. Current State Inventory

| Layer | Location | Language / Stack | Lines |
|-------|----------|-----------------|-------|
| CLI binary | `src/` (root) | Rust (Clap, Tokio, Reqwest) | ~100 files |
| Code-gen engine | `crates/forge/` | Rust (Tera templates) | ~10 files |
| Registry API server | `crates/registry-server/` | Rust (Axum, SQLx, PostgreSQL) | ~14 files |
| Registry web dashboard | `apps/registry-web/` | TypeScript (TanStack Start, Drizzle, React) | ~119 files |
| Documentation site | `apps/docs/` | TypeScript (TanStack Start, MDX, React) | ~20 files |
| Shared UI components | `packages/ui/` | TypeScript (React) | 2 files |
| CI/CD | `.github/workflows/ci.yml` | GitHub Actions | 5 jobs |
| Deployment | `fly.toml` + Dockerfile | Fly.io (registry-server) | — |

---

## 2. Architectural Problems (Current)

### 2.1 Two backends hitting the same database (Critical SoC violation)

`apps/registry-web/src/db/` contains a **Drizzle ORM schema** and connects to
the same PostgreSQL database that `crates/registry-server` (SQLx) also manages.
This means two separate ORMs, two separate migration systems, and two separate
query layers own the same tables.

```
❌ Current
Browser → TanStack Start server fn → Drizzle ORM → PostgreSQL
Browser → Rust Axum API → SQLx → PostgreSQL  (same DB!)
```

This creates schema drift risk: a migration applied via Drizzle may not be
reflected in the SQLx models, and vice versa. There is no single source of
truth for the database schema.

### 2.2 CLI source lives at workspace root

The main CLI (`src/`) is the root Cargo package, not a dedicated crate. This
means `Cargo.toml` serves double duty as both the workspace manifest and the
CLI package manifest, making it hard to reason about workspace dependencies
vs. binary dependencies.

### 2.3 No shared TypeScript types for the Rust API

`apps/registry-web/src/lib/types.ts` manually hand-codes TypeScript interfaces
that mirror the JSON shapes returned by `crates/registry-server`. When the Rust
models change, the TypeScript side silently drifts.

### 2.4 `packages/ui` is minimal and the Header is duplicated

Both apps have their own `Header.tsx` with near-identical structure. The shared
package only extracts `ThemeToggle` and `Footer`. Header navigation is
app-specific, but the shell (sticky positioning, blur, logo slot, right slot)
is identical and could be a shared `BaseHeader` render prop.

### 2.5 CI does not use Bun workspaces

Although a root `package.json` with `workspaces` now exists, the CI workflow
still `cd apps/registry-web && bun install` + `cd apps/docs && bun install`
separately, bypassing the workspace and causing duplicated installs.

### 2.6 `data/`, `templates/`, `examples/`, `frameworks/` at root

Runtime data (`data/`) and CLI content (`templates/`, `examples/`,
`frameworks/`) are mixed with source code at root. There is no clear boundary
between repository source and generated/runtime artefacts.

### 2.7 No schema migration ownership

Both `drizzle-kit` (in registry-web) and SQLx migrations (in registry-server)
claim to manage the schema. Running `drizzle-kit push` from the frontend app
silently becomes the de facto migration tool.

---

## 3. Proposed Target Architecture

```
tsx/
├── Cargo.toml                  ← workspace-only manifest (no [package])
├── package.json                ← Bun workspace root
│
├── crates/
│   ├── cli/                    ← moved from src/ — the tsx binary
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── forge/                  ← code-gen engine library (keep as-is)
│   ├── registry-server/        ← Axum API server (keep, minor changes)
│   └── shared/                 ← NEW: shared Rust types across crates
│       ├── Cargo.toml
│       └── src/
│           └── models.rs       ← serde Serialize/Deserialize models shared
│                               ← between cli and registry-server
│
├── apps/
│   ├── registry-web/           ← pure BFF+SPA (see §3.1)
│   └── docs/                   ← documentation site (keep as-is)
│
├── packages/
│   ├── ui/                     ← shared React components (expand)
│   └── api-types/              ← NEW: TypeScript types that match Rust API
│       ├── package.json
│       └── src/
│           └── index.ts        ← generated or hand-maintained API types
│
├── migrations/                 ← NEW: single place for all DB migrations
│   └── *.sql                   ← plain SQL, applied by registry-server
│
├── scripts/                    ← NEW: dev tooling scripts
│   ├── seed.ts
│   └── migrate.sh
│
└── .runtime/                   ← gitignored, replaces data/ at root
    └── data/
```

---

## 3.1 Backend Strategy — Pick One, Own It

The most important decision. Three options:

### Option A — Rust owns everything (Recommended)

`crates/registry-server` is the **only** backend. It owns all DB access,
migrations, and business logic. The TanStack Start server functions in
`apps/registry-web/src/server/` are **deleted** — they become thin HTTP proxy
calls to the Rust API.

```
✅ Option A
Browser → TanStack Start server fn → fetch() → Rust Axum API → SQLx → PostgreSQL
```

**Pros:** One language for backend, SQLx migrations are authoritative, no
Drizzle schema duplication, Rust binary stays the canonical artefact.

**Cons:** Server functions become network-bound (one extra hop). Need to
wire TanStack Server Functions as API proxies, not direct DB calls.

**Migration path:**
1. Replace each `apps/registry-web/src/server/*` mutation with a `fetch()` call
   to the corresponding `GET/POST /v1/...` endpoint on the Rust server.
2. Delete `apps/registry-web/src/db/` entirely.
3. Remove `drizzle-*` dependencies from `registry-web/package.json`.
4. Move all migrations to `migrations/` at repo root, applied only by the
   Rust server at startup (already done via `sqlx::migrate!`).

### Option B — TypeScript owns everything

Remove `crates/registry-server`. The TanStack Start server functions with
Drizzle become the only backend. The Rust CLI calls the `/v1/...` REST
endpoints emitted by TanStack Start's Nitro server.

**Pros:** One language end-to-end for web, simpler deployment.
**Cons:** Loses the Rust performance + type safety for the registry layer.
Large surface area refactor. Counter to the project's Rust-first identity.

### Option C — Strict BFF boundary (middle ground, not recommended)

Keep both, but enforce: Drizzle is **only** used for auth session tables
(owned by `better-auth`), and all other queries go through the Rust API.

**Cons:** Two ORMs, two migration systems, still fragile. Not recommended.

---

## 4. Rust Workspace Restructuring

### 4.1 Separate the CLI into its own crate

Move `src/` → `crates/cli/src/` and update `Cargo.toml`:

```toml
# Cargo.toml (root) — workspace manifest only
[workspace]
members = [
  "crates/cli",
  "crates/forge",
  "crates/registry-server",
  "crates/shared",
]
resolver = "2"

[workspace.dependencies]
# shared deps here
```

```toml
# crates/cli/Cargo.toml
[package]
name = "tsx"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "tsx"
path = "src/main.rs"

[dependencies]
tsx-shared = { path = "../shared" }
tsx-forge  = { path = "../forge" }
# clap, reqwest, etc.
```

This makes `cargo build -p tsx` (CLI) and `cargo build -p tsx-registry` (server)
completely independent build targets with zero accidental coupling.

### 4.2 Extract shared models into `crates/shared`

Both the CLI (deserialises API responses) and the registry-server (serialises
them) currently duplicate model structs. Extract into one crate:

```
crates/shared/src/
├── lib.rs
├── package.rs      ← Package, VersionRow, SearchResult
├── manifest.rs     ← ManifestJson (FPF spec)
└── error.rs        ← shared error types
```

Both crates add `tsx-shared = { path = "../shared" }`.

---

## 5. TypeScript Monorepo Restructuring

### 5.1 `packages/api-types` — single source of truth for API contracts

```ts
// packages/api-types/src/index.ts
export interface Package { name: string; version: string; description: string; ... }
export interface SearchResult { packages: Package[]; total: number; page: number; ... }
export interface RegistryStats { total_packages: number; total_downloads: number; ... }
// ... mirrors crates/shared models exactly
```

Both `apps/registry-web` and `apps/docs` add `"@tsx/api-types": "workspace:*"`.

When the Rust models change, update `packages/api-types/src/index.ts` in the
same PR. This creates a conscious, reviewable API contract boundary.

**Future:** Auto-generate from an OpenAPI spec emitted by the Rust server
(`utoipa` crate) → `openapi-typescript` CLI → `packages/api-types/src/index.ts`.

### 5.2 `packages/ui` — expand to a proper component library

Current state: 2 files. Expand to cover the shared chrome:

```
packages/ui/src/
├── ThemeToggle.tsx         ← done
├── Footer.tsx              ← done
├── BaseHeader.tsx          ← NEW: shell with logo/nav/right slots (render props)
├── NavLink.tsx             ← NEW: active-aware anchor used by both Headers
├── CodeBlock.tsx           ← NEW: shared hljs/shiki wrapper (docs + registry-web)
└── index.ts                ← barrel
```

`BaseHeader` uses render props / children so each app fills in its own nav
links and right-side buttons, while the layout (sticky, backdrop-blur, height,
border) is shared:

```tsx
// apps/docs/src/components/Header.tsx
import { BaseHeader } from "@tsx/ui"
export function Header() {
  return (
    <BaseHeader
      logo={<span>tsx docs</span>}
      nav={<DocsNav />}
      right={<ThemeToggle />}
    />
  )
}
```

### 5.3 Fix CI to use Bun workspaces

Replace the current "install each app separately" CI pattern:

```yaml
# .github/workflows/ci.yml  — web job
- name: Install all workspace deps
  run: bun install          # runs at repo root, resolves all workspaces

- name: Build registry-web
  run: bun --cwd apps/registry-web build

- name: Build docs
  run: bun --cwd apps/docs build
```

Single `bun install` at root installs shared `node_modules` once, symlinks
workspace packages, and is significantly faster.

---

## 6. Database Migration Ownership

**Rule:** Migrations live in `migrations/` at repo root. Only
`crates/registry-server` applies them (via `sqlx::migrate!("../../migrations")`).
`apps/registry-web` has no migration tooling — its Drizzle config is
**deleted**.

```
migrations/
├── 0001_initial_schema.sql
├── 0002_add_downloads_table.sql
├── 0003_add_audit_log.sql
└── 0004_add_better_auth_tables.sql
```

`drizzle-kit` and the `db:*` scripts are removed from `registry-web`. The
`apps/registry-web/src/db/` directory (Drizzle schema) is deleted when
Option A is adopted.

---

## 7. Deployment Architecture

```
                    ┌──────────────────────────────────┐
                    │         Fly.io                   │
                    │  tsx-registry (Rust binary)       │
                    │  Port 8080 / HTTPS               │
                    │  Volume: /data (tarballs)         │
                    │  Env: DATABASE_URL, API_KEY       │
                    └──────────────┬───────────────────┘
                                   │ REST API /v1/*
                    ┌──────────────▼───────────────────┐
                    │         Vercel / Fly.io           │
                    │  registry-web (Nitro SSR)         │
                    │  TanStack Start server fns        │
                    │  → proxy to Rust API             │
                    └──────────────┬───────────────────┘
                                   │
                    ┌──────────────▼───────────────────┐
                    │         Vercel                   │
                    │  docs (Nitro SSR)                │
                    │  Static MDX pages                │
                    └──────────────────────────────────┘

                    ┌──────────────────────────────────┐
                    │  Neon / Supabase PostgreSQL       │
                    │  Single DB, single owner         │
                    │  Migrated by registry-server     │
                    └──────────────────────────────────┘
```

---

## 8. Separation of Concerns Matrix

| Concern | Owns It | Does NOT own it |
|---------|---------|----------------|
| DB schema + migrations | `crates/registry-server` | `apps/registry-web` |
| Auth session storage | `crates/registry-server` (better-auth tables) | `apps/registry-web/src/db` |
| Package metadata queries | `crates/registry-server` REST API | `apps/registry-web` (direct DB) |
| API types contract | `packages/api-types` | Inline in each app |
| Code generation logic | `crates/forge` | `crates/cli` (calls forge) |
| CLI binary | `crates/cli` | `src/` root |
| UI chrome (Header/Footer/ThemeToggle) | `packages/ui` | Both apps individually |
| Docs content | `apps/docs/src/content/*.mdx` | Route files |
| E2E tests | `apps/registry-web/e2e/` | `apps/docs/` (no E2E needed) |
| Storybook | `apps/registry-web` | Elsewhere |

---

## 9. Implementation Phases

### Phase 1 — Structural cleanup (no behaviour change) ✅ COMPLETE

| Task | Status | Notes |
|------|--------|-------|
| Move `src/` → `crates/cli/src/` | ✅ Done | `git mv`; Cargo alias `forge = { package = "tsx-forge" }` preserved |
| Create `crates/shared/` with shared models | ✅ Done | `tsx-shared` crate; registry-server re-exports as `pub use tsx_shared as models` |
| Create `packages/api-types/` with TypeScript interfaces | ✅ Done | `@tsx/api-types`; `registry-web/lib/types.ts` now re-exports from it |
| Move `data/` to `.gitignore`, use `DATA_DIR` env only | ✅ Done | Already gitignored; added `.runtime/` entry |
| Move migrations to root `migrations/` dir | ✅ Done | `migrations/0001_initial_schema.sql`; inline SQL in db/mod.rs kept for self-contained binary |
| Fix CI to use single `bun install` at root | ✅ Done | Workspace cache shared across e2e/lighthouse/storybook jobs |

### Phase 2 — Backend consolidation (Option A) ✅ COMPLETE

| Task | Status | Notes |
|------|--------|-------|
| Replace `server/admin/queries.ts` Drizzle queries with `fetch()` to Rust API | ✅ Done | `getAdminAuditLog` → `GET /v1/admin/audit-log`; `getAdminUsers` kept in Drizzle (better-auth `user` table) |
| Delete `apps/registry-web/src/db/schema/packages.ts` | ✅ Done | Drizzle schema duplicating Rust tables removed |
| Update `db/schema/index.ts` to remove packages export | ✅ Done | Only exports auth schema |
| Update `audit-log.tsx` field names to snake_case | ✅ Done | Matches Rust API contract (`package_name`, `author_name`, `ip_address`, `created_at`) |
| Remove `drizzle-seed` from registry-web devDependencies | ✅ Done | No longer seeding package tables from TS side |

### Phase 3 — Shared package expansion ✅ COMPLETE

| Task | Status | Notes |
|------|--------|-------|
| Extract `BaseHeader` into `packages/ui` | ✅ Done | `packages/ui/src/BaseHeader.tsx` — logo/nav/right slots; both app Headers refactored to use it |
| Wire both apps to `@tsx/api-types` | ✅ Done | Done in Phase 1; `registry-web/lib/types.ts` re-exports from `@tsx/api-types` |
| `CodeBlock` / hljs wrapper | ⏭ Skipped | Both apps use hljs as a DOM side-effect in `useEffect`, not a component; extracting would require refactoring markdown rendering — deferred to Phase 4 |
| OpenAPI spec from Rust server (`utoipa`) | ⏭ Deferred | Phase 4 |
| Auto-generate `packages/api-types` from OpenAPI spec | ⏭ Deferred | Phase 4 |

### Phase 4 — Observability & ops ✅ COMPLETE

| Task | Status | Notes |
|------|--------|-------|
| Storybook CI job | ✅ Done | Added in Phase 1 (`bun run build-storybook`) |
| Structured JSON logging in Rust server | ✅ Done | `LOG_FORMAT=json` env var enables JSON output via `tracing-subscriber` json feature |
| Wire rate-limit dashboard to real endpoint | ✅ Done | `getAdminRateLimits` server fn → `GET /v1/admin/rate-limits`; page auto-refreshes every 30s |
| Metrics endpoint (`/metrics` Prometheus) | ⏭ Deferred | Requires `axum-prometheus` or `metrics` crate — post-stabilisation task |
| `cargo test -p tsx-registry` integration tests | ⏭ Deferred | Needs test database setup in CI — post-stabilisation task |

---

## 10. File & Naming Conventions

### Rust
- Crate names: `tsx-*` (e.g. `tsx-cli`, `tsx-forge`, `tsx-registry`, `tsx-shared`)
- Module files: snake_case (`db/packages.rs`, `routes/search.rs`)
- Public re-exports in `mod.rs` so call sites never import sub-module paths directly

### TypeScript
- Packages: `@tsx/*` (e.g. `@tsx/ui`, `@tsx/api-types`)
- Components: PascalCase files (`ThemeToggle.tsx`)
- Hooks: `use-*` files (`use-session.ts`)
- Server functions: `apps/*/src/server/` — mutations only, no DB access post-Phase 2
- Stories: co-located `*.stories.tsx` next to component

### SQL
- Migration files: `NNNN_description.sql` (zero-padded 4-digit number)
- Applied ascending by `sqlx::migrate!` at server startup
- Never edited after merge — only forward migrations

---

## 11. What NOT to Do

- **Do not add another ORM** to the Rust side (SQLx `query!` macros are
  sufficient and checked at compile time).
- **Do not split `apps/registry-web` into separate frontend/backend packages**
  — TanStack Start's colocation of server functions and UI is the framework's
  core value. The fix is to make server functions thin HTTP proxies, not to
  restructure the app.
- **Do not add a message queue or microservices** at this stage. The registry
  is a single-purpose service; prematurely splitting it would add operational
  complexity with no benefit.
- **Do not generate TypeScript from Rust types automatically yet** — manual
  sync via `packages/api-types` is fine until the API surface stabilises.
  Add codegen (utoipa → openapi-typescript) in Phase 4.
