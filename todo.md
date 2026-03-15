# TSX — Implementation Plan

> Rust project already initialised with `cargo new tsx --bin`. Work top-to-bottom. Each task is a discrete, testable unit of work.

---

## Phase 1 — Foundation

### 1.1 Cargo.toml — Add dependencies

- [x] Add `clap = { version = "4", features = ["derive"] }`
- [x] Add `minijinja = { version = "2", features = ["loader"] }`
- [x] Add `serde = { version = "1", features = ["derive"] }`
- [x] Add `serde_json = "1"`
- [x] Add `anyhow = "1"`
- [x] Add `walkdir = "2"`
- [x] Add `heck = "0.5"`
- [x] Run `cargo build` — confirm clean compile with zero feature errors

### 1.2 Project structure — create module skeleton

- [x] Create `src/commands/` directory with `mod.rs`
- [x] Create `src/schemas/` directory with `mod.rs`
- [x] Create `src/render/` directory with `mod.rs`
- [x] Create `src/utils/` directory with `mod.rs`
- [x] Create `src/json/` directory with `mod.rs` — for JSON input/output handling
- [x] Create `src/output.rs` — stub `CommandResult` struct
- [x] Declare all modules in `src/main.rs`
- [x] Run `cargo check` — confirm all modules resolve

### 1.3 CLI skeleton — `clap` app

- [x] Define `Cli` struct in `main.rs` with `#[derive(Parser)]`
- [x] Define `Command` enum with `#[derive(Subcommand)]` — all 12 subcommands as stubs
- [x] Wire `match cli.command { }` in `main` — each arm prints `"not yet implemented"` and exits `0`
- [x] Add `--overwrite`, `--dry-run`, and `--verbose` as global flags on `Cli`
- [x] Add `--json`, `--stdin`, and `--file` flags for JSON input mode
- [x] Run `cargo run -- --help` — confirm all subcommands appear in help output
- [x] Run `cargo run -- add:feature --help` — confirm flag appears

### 1.4 JSON input/output — core infrastructure

- [x] Create `src/json/payload.rs` — command payload structures for JSON input
- [x] Create `src/json/response.rs` — structured response envelope with metadata
- [x] Create `src/json/error.rs` — error types with codes (VALIDATION_ERROR, FILE_EXISTS, etc.)
- [x] Implement `ResponseEnvelope::success()` builder
- [x] Implement `ResponseEnvelope::error()` builder
- [x] Implement JSON input parsing from `--json`, `--stdin`, and `--file` sources
- [x] Write unit tests: parse various JSON payloads, assert correct deserialisation
- [x] Run `cargo test json` — passes

### 1.5 Payload schemas — `serde` structs

- [x] Create `src/schemas/field.rs` — `FieldSchema` struct + `FieldType` enum (all 11 variants) + `Operation` enum
- [x] Create `src/schemas/feature.rs` — `AddFeatureArgs` with `name`, `fields`, `auth`, `paginated`, `operations`
- [x] Create `src/schemas/schema.rs` — `AddSchemaArgs` with `name`, `fields`, `timestamps`, `soft_delete`
- [x] Create `src/schemas/server_fn.rs` — `AddServerFnArgs`
- [x] Create `src/schemas/query.rs` — `AddQueryArgs`
- [x] Create `src/schemas/form.rs` — `AddFormArgs`
- [x] Create `src/schemas/page.rs` — `AddPageArgs`
- [x] Create `src/schemas/auth.rs` — `AddAuthArgs`, `AddAuthGuardArgs`
- [x] Create `src/schemas/seed.rs` — `AddSeedArgs`
- [x] Re-export all from `src/schemas/mod.rs`
- [x] Write unit tests in each schema file: deserialise a valid JSON fixture, assert field values
- [x] Run `cargo test schemas` — all pass

### 1.6 Output contract

- [x] Define `CommandResult` in `src/output.rs` with `success`, `command`, `files_created`, `warnings`, `next_steps`, `metadata`
- [x] Implement `CommandResult::ok(command, files)` and `CommandResult::err(command, msg)` constructors
- [x] Implement `CommandResult::print(&self)` — serialises to JSON and writes to stdout
- [x] Add `metadata` field with `timestamp` and `duration_ms`
- [x] Write unit test: serialise a result, deserialise, assert round-trip
- [x] Run `cargo test output` — passes

### 1.7 Path utilities

- [x] Create `src/utils/paths.rs`
- [x] Implement `find_project_root() -> anyhow::Result<PathBuf>` — walks up from `std::env::current_dir()` looking for `package.json` using `walkdir`
- [x] Implement `resolve_output_path(root: &Path, relative: &str) -> PathBuf`
- [x] Write unit test: create a temp dir with a nested `package.json`, confirm root is found from a child dir
- [x] Run `cargo test paths` — passes

### 1.8 Atomic file writer

- [x] Create `src/utils/write.rs`
- [x] Implement `write_file(path: &Path, content: &str, overwrite: bool) -> anyhow::Result<WriteOutcome>` — returns `Created`, `Skipped` (file exists, overwrite=false), or `Overwritten`
- [x] Ensure parent directories are created with `fs::create_dir_all`
- [x] Write unit tests: create file, skip existing, overwrite with flag
- [x] Run `cargo test write` — passes

### 1.9 MiniJinja engine bootstrap

- [x] Create `src/render/engine.rs`
- [x] Implement `build_engine(templates_dir: &Path) -> minijinja::Environment` — loads all `.jinja` files from `templates/` recursively
- [x] Register custom filters: `snake_case`, `pascal_case`, `camel_case`, `kebab_case` using `heck`
- [x] Implement `ImportCollector` using `thread_local!` + `RefCell<BTreeSet<String>>`
- [x] Register `collect_import` as a MiniJinja filter (pushes to collector, returns empty string)
- [x] Register `collect_import_priority` as a MiniJinja filter (pushes to priority vec, returns empty string)
- [x] Register `render_imports` as a MiniJinja global function (drains both collectors, returns joined string)
- [x] Implement `reset_import_collector()` — clears both thread-locals; call this before every render
- [x] Create `src/render/context.rs` — `RenderContext` builder that converts payload structs to `minijinja::Value` maps
- [x] Write unit test: build engine, render a trivial inline template with `{{ name | snake_case }}`, assert output
- [x] Run `cargo test render` — passes

### 1.10 Template directory — atoms skeleton

- [x] Create `templates/` directory at crate root
- [x] Create `templates/atoms/drizzle/`, `templates/atoms/zod/`, `templates/atoms/form/`, `templates/atoms/query/`, `templates/atoms/imports/`
- [x] Create `templates/molecules/drizzle/`, `templates/molecules/zod/`, `templates/molecules/server_fn/`, `templates/molecules/form/`, `templates/molecules/table/`, `templates/molecules/query/`, `templates/molecules/auth/`
- [x] Create `templates/layouts/`
- [x] Create `templates/features/`
- [x] Create `templates/metadata.json` — template metadata for introspection
- [x] Add placeholder `_keep` files so git tracks empty dirs (remove once real files are added)

---

## Phase 2 — Atoms, Molecules, Layouts

### 2.1 Core atoms — Drizzle

- [x] Write `templates/atoms/drizzle/column.jinja` — full `{% if/elif %}` block for all 11 field types; calls `collect_import` for the correct drizzle-orm import
- [x] Write `templates/atoms/drizzle/timestamp_cols.jinja` — `createdAt` + `updatedAt` integer timestamp columns
- [x] Write `templates/atoms/drizzle/soft_delete_col.jinja` — `deletedAt` nullable timestamp column
- [x] Write `templates/atoms/drizzle/relation.jinja` — `relations()` export for a FK field
- [x] Write atom unit tests in `tests/atoms/drizzle.rs`: render each atom with a fixture context, assert output string contains expected column definition
- [x] Run `cargo test atoms::drizzle` — all pass

### 2.2 Core atoms — Zod

- [x] Write `templates/atoms/zod/field_rule.jinja` — maps `FieldType` to `z.*()` rule with optional `.min()`, `.email()`, `.url()` chaining
- [x] Write `templates/atoms/zod/object_wrapper.jinja` — `export const <name>Schema = z.object({ ... })`
- [x] Write atom unit tests in `tests/atoms/zod.rs`
- [x] Run `cargo test atoms::zod` — all pass

### 2.3 Core atoms — Form fields

- [x] Write `templates/atoms/form/field_input.jinja`
- [x] Write `templates/atoms/form/field_select.jinja`
- [x] Write `templates/atoms/form/field_switch.jinja`
- [x] Write `templates/atoms/form/field_datepicker.jinja`
- [x] Write `templates/atoms/form/field_textarea.jinja`
- [x] Write atom unit tests in `tests/atoms/form.rs`
- [x] Run `cargo test atoms::form` — all pass

### 2.4 Core atoms — Query

- [x] Write `templates/atoms/query/query_key.jinja`
- [x] Write `templates/atoms/query/suspense_query.jinja`
- [x] Write `templates/atoms/query/mutation.jinja`
- [x] Write atom unit tests in `tests/atoms/query.rs`
- [x] Run `cargo test atoms::query` — all pass

### 2.5 Molecules — Drizzle

- [x] Write `templates/molecules/drizzle/table_body.jinja` — `sqliteTable(...)` block iterating over fields via `{% for field in fields %}{% include atom %}{% endfor %}`; includes timestamp + soft-delete atoms conditionally; emits type exports
- [x] Write `templates/molecules/drizzle/schema_shared.jinja` — shared service types molecule
- [x] Write molecule integration test in `tests/molecules/drizzle.rs`: render `table_body` with a 3-field fixture, parse output, assert `sqliteTable` call present, assert type exports present, assert `ImportCollector` drained correct imports
- [x] Run `cargo test molecules::drizzle` — passes

### 2.6 Molecules — Zod

- [x] Write `templates/molecules/zod/schema_block.jinja` — full `z.object({})` wrapping field rule atoms
- [x] Write molecule integration test
- [x] Run `cargo test molecules::zod` — passes

### 2.7 Molecules — Server function

- [x] Write `templates/molecules/server_fn/handler.jinja` — `createServerFn().validator(schema).handler(async ({ data }) => { ... })` for list / create / update / delete operations with auth guard branch
- [x] Write molecule integration test
- [x] Run `cargo test molecules::server_fn` — passes

### 2.8 Molecules — Query hooks

- [x] Write `templates/molecules/query/hooks_block.jinja` — `useSuspenseQuery`, `useMutation` exports per operation
- [x] Write molecule integration test
- [x] Run `cargo test molecules::query` — passes

### 2.9 Molecules — Form component

- [x] Write `templates/molecules/form/form_component.jinja` — `useForm` hook, JSX field loop dispatching to form field atoms, submit button
- [x] Write molecule integration test
- [x] Run `cargo test molecules::form` — passes

### 2.10 Molecules — Table component

- [x] Write `templates/molecules/table/data_table.jinja` — `useReactTable` column defs, thead/tbody render, pagination controls
- [x] Write molecule integration test
- [x] Run `cargo test molecules::table` — passes

### 2.11 Molecules — Auth

- [x] Write `templates/molecules/auth/config_block.jinja` — `betterAuth({})` config with provider and session field slots
- [x] Write molecule integration test
- [x] Run `cargo test molecules::auth` — passes

### 2.12 Layouts

- [x] Write `templates/layouts/base.jinja` — `{{ render_imports() }}` drain + `{% block body %}` slot
- [x] Write `templates/layouts/component.jinja` — priority React import + drain + body block
- [x] Write `templates/layouts/route.jinja` — priority router imports + drain + Route export + `{% block loader %}` + `{% block body %}`
- [x] Write layout integration tests: render a layout with a simple molecule injected, assert import block appears at top of output, assert no duplicate imports
- [x] Run `cargo test layouts` — passes

### 2.13 Feature templates

- [x] Write `templates/features/schema.jinja`
- [x] Write `templates/features/server_fn.jinja`
- [x] Write `templates/features/query.jinja`
- [x] Write `templates/features/form.jinja`
- [x] Write `templates/features/table.jinja`
- [x] Write `templates/features/page.jinja`
- [x] Write `templates/features/seed.jinja`
- [x] Write `templates/features/auth_config.jinja`
- [x] Write feature end-to-end render tests in `tests/features/`: render each feature with a representative fixture, assert output compiles (pipe through `tsc --noEmit` in test), assert no duplicate imports
- [x] Run `cargo test features` — all pass

---

## Phase 3 — Agent-Friendly JSON API

### 3.1 Structured error handling

- [x] Implement error code enum in `src/json/error.rs` — all error codes (INVALID_PAYLOAD, VALIDATION_ERROR, FILE_EXISTS, etc.)
- [x] Implement `ErrorResponse::new()` builder with error details
- [x] Wire structured error output in `main.rs` catch block
- [x] Write unit tests: trigger various errors, assert JSON error format
- [x] Run `cargo test error` — passes

### 3.2 Introspection — list command

- [x] Create `src/commands/list.rs`
- [x] Implement `list templates` — reads `templates/metadata.json`, returns template list
- [x] Implement `list generators` — returns all available commands with option schemas
- [x] Implement `list components` — returns available shadcn components with props
- [x] Write unit tests: call list with each kind, assert JSON response
- [x] Run `cargo test list` — passes

### 3.3 Project inspection

- [x] Create `src/commands/inspect.rs`
- [x] Implement `inspect` command — scans project structure, returns schemas, routes, queries, forms, tables
- [x] Detect database provider and migration status
- [x] Detect auth configuration
- [x] Write unit tests: run inspect on fixture project, assert structure returned
- [x] Run `cargo test inspect` — passes

### 3.4 Batch operations

- [x] Create `src/commands/batch.rs`
- [x] Implement `batch` command — accepts array of commands, executes sequentially
- [x] Implement result aggregation with per-command success/failure
- [x] Implement early termination option on failure
- [x] Write unit tests: batch 3 commands, assert aggregated results
- [x] Run `cargo test batch` — passes

### 3.5 Dry-run mode

- [x] Implement `dryRun: true` handling in all command handlers
- [x] Return would-be-created files without writing to disk
- [x] Add `dryRun` field to response envelope
- [x] Write integration tests: dry-run across commands, assert no files created
- [x] Run `cargo test dry_run` — passes

### 3.6 Verbose mode

- [x] Implement `--verbose` flag handling
- [x] Add `warnings` array to metadata
- [x] Add `context` object to response with project_root and tsx_version
- [x] Write tests: run with --verbose, assert extended response
- [x] Run `cargo test verbose` — passes

---

## Phase 4 — Command Handlers

### 4.1 Single-file command handlers

> Each handler follows the same pattern: deserialise payload → resolve path → reset ImportCollector → render feature template → format output → write file → return `CommandResult`.

- [x] Implement `src/commands/add_schema.rs` — renders `features/schema.jinja`, writes to `db/schema/<name>.ts`
- [x] Implement `src/commands/add_server_fn.rs` — renders `features/server_fn.jinja`, writes to `server-functions/<name>.ts`
- [x] Implement `src/commands/add_query.rs` — renders `features/query.jinja`, writes to `queries/<name>.ts`
- [x] Implement `src/commands/add_form.rs` — renders `features/form.jinja`, writes to `components/<name>/<name>-form.tsx`
- [x] Implement `src/commands/add_table.rs` — renders `features/table.jinja`, writes to `components/<name>/<name>-table.tsx`
- [x] Implement `src/commands/add_page.rs` — renders `features/page.jinja`, writes to `routes/<path>.tsx`
- [x] Implement `src/commands/add_seed.rs` — renders `features/seed.jinja`, writes to `db/seeds/<name>.ts`
- [x] Wire each handler into the `main.rs` match arm
- [x] Smoke-test each command against a real TanStack Start project fixture: run command, open generated file, confirm it compiles

### 4.2 `add:feature` — compound command

- [x] Implement `src/commands/add_feature.rs`
- [x] Orchestrate calls to `add_schema`, `add_server_fn`, `add_query`, `add_page` (index + $id), `add_form`, `add_table`, plus a delete-dialog render
- [x] Collect all `files_created` from sub-commands into one `CommandResult`
- [x] Add `next_steps: ["Run: tsx add:migration {}"]` to result
- [x] Integration test: run `add:feature` with a 4-field fixture against a real project, assert all 8 files are created and compile

### 4.3 `add:auth` and `add:auth-guard`

- [x] Implement `src/commands/add_auth.rs` — renders `features/auth_config.jinja`, writes to `lib/auth.ts`; shells out `npx @better-auth/cli generate` if requested
- [x] Implement `src/commands/add_auth_guard.rs` — injects a `beforeLoad` guard into an existing route file using string manipulation
- [x] Integration tests for both commands

### 4.4 `add:migration`

- [x] Implement `src/commands/add_migration.rs` — shells out `npx drizzle-kit generate` then `npx drizzle-kit migrate` using `std::process::Command`; streams stdout to the terminal; surfaces exit codes as structured errors

### 4.5 `init`

- [x] Implement `src/commands/init.rs`
- [x] Shell out `npm create tanstack@latest` with `--template start` flag
- [x] Run `npx shadcn@latest init` in the new project dir
- [x] Write `drizzle.config.ts`, `.env.example`, and initial Better Auth config
- [x] Print a `CommandResult` with all created file paths

### 4.6 Import injection utility

- [x] Implement `src/utils/imports.rs` — `inject_import(file_path: &Path, import_line: &str) -> anyhow::Result<()>` reads the file, checks if import already present, inserts after the last existing import line if not
- [x] Write unit tests: inject into a file with existing imports, inject into empty file, skip if already present

### 4.7 Barrel file utility

- [x] Implement `src/utils/barrel.rs` — `update_barrel(dir: &Path, export_line: &str) -> anyhow::Result<()>` appends or creates `index.ts` in the given directory
- [x] Write unit tests

---

## Phase 5 — Hardening & Distribution

### 5.1 Prettier integration

- [x] Implement `src/utils/format.rs` — `format_typescript(content: &str) -> anyhow::Result<String>` spawns `npx prettier --parser typescript --stdin-filepath file.ts`, pipes content to stdin, reads formatted output from stdout
- [x] Gracefully degrade: if Prettier is not available, return content unchanged and add a warning to `CommandResult`
- [x] Wire into the render pipeline in all command handlers (after render, before write)

### 5.2 Embed templates in binary

- [x] Use `include_dir` crate or `build.rs` + `include_str!` macros to embed the entire `templates/` directory into the binary at compile time
- [x] Update `build_engine()` to load templates from the embedded bytes when no external `templates/` directory is found alongside the binary
- [x] Verify binary runs correctly with no `templates/` directory present on disk

### 5.3 End-to-end agent stress test

- [x] Create `tests/e2e/` directory with a minimal TanStack Start project fixture (committed to repo)
- [x] Write a test script that runs the full agent workflow: `init` → `add:feature` (×3 resources) → `add:migration` → `add:auth`
- [x] Assert all generated files compile: run `npx tsc --noEmit` in the fixture project after generation
- [x] Measure token count of equivalent manual agent generation (baseline) vs CLI workflow — document reduction

### 5.4 Flags and edge cases

- [x] Test `--dry-run` across all commands — confirm no files are written, JSON result lists what *would* be created
- [x] Test `--overwrite` — confirm existing files are replaced
- [x] Test missing `package.json` (not in a project) — confirm clear error message, non-zero exit code
- [x] Test malformed JSON payload — confirm `serde_json` error is surfaced as structured output, not a panic
- [x] Test `--stdin` input mode
- [x] Test `--file` input mode

### 5.5 Cross-compilation & release

- [x] Add `[profile.release]` to `Cargo.toml`: `opt-level = 3`, `lto = true`, `codegen-units = 1`, `strip = true`
- [x] Set up GitHub Actions workflow: build matrix for `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-pc-windows-gnu`
- [x] Verify binary size ≤ 10MB on all targets
- [x] Verify cold-start time ≤ 10ms: `time tsx --help`
- [x] Publish binaries as GitHub Release assets with checksums

---

## Phase 6 — Template Versioning

### 6.1 Atom version pinning

- [ ] Add version field to atom metadata
- [ ] Implement `tsx upgrade` command to update atom versions
- [ ] Support pinning specific atom versions per project in `package.json`
- [ ] Add version compatibility checking

### 6.2 Breaking change detection

- [ ] Add deprecation warnings to atom templates
- [ ] Implement breaking change detection between versions
- [ ] Generate migration guide for version upgrades

---

## Phase 7 — Custom Template Plugins

### 7.1 Plugin system

- [ ] Add `--plugin` flag to load custom template packages
- [ ] Implement plugin discovery from npm
- [ ] Support template overrides for specific generators
- [ ] Implement plugin sandboxing for security

### 7.2 Plugin API

- [ ] Define plugin manifest format (`plugin.json`)
- [ ] Implement plugin validation
- [ ] Add plugin commands: `list`, `install`, `remove`

---

## Phase 8 — WebSocket Dev Server

### 8.1 Watch mode

- [ ] Add `--watch` flag for file regeneration
- [ ] Implement file system watcher for template changes
- [ ] Support selective regeneration based on changed files

### 8.2 WebSocket events

- [ ] Implement WebSocket server for real-time events
- [ ] Add event types: file_created, file_modified, error, ready
- [ ] Support hot module replacement integration

---

## Phase 9 — Enhanced Learning Mode

### 9.1 Semantic search

- [ ] Implement fuzzy matching for question topics
- [ ] Add relevance scoring to answers
- [ ] Support multi-framework question routing

### 9.2 Enhanced explain

- [ ] Add decision versioning with changelog
- [ ] Implement learn-more URL resolution
- [ ] Add visual decision tree rendering

---

## Phase 10 — Registry Publishing

### 10.1 Publish command

- [ ] Implement `tsx publish` for sharing custom registries
- [ ] Add registry validation and testing
- [ ] Implement registry versioning

### 10.2 Registry ecosystem

- [ ] Create framework registry website
- [ ] Implement registry discovery service
- [ ] Add community template sharing

---

## Phase 11 — Additional Framework Support

### 11.1 New framework registries

- [ ] Add Solid.js framework registry
- [ ] Add Kysely ORM registry
- [ ] Add Zustand state management registry
- [ ] Add Jotai state management registry

### 11.2 Integration patterns

- [ ] Add Tailwind CSS integration patterns
- [ ] Add payment integration (Stripe)
- [ ] Add analytics integration patterns

---

## Checklist Summary

| Phase | Tasks | Done |
|---|---|---|
| Phase 1 — Foundation | Cargo setup, modules, CLI skeleton, JSON I/O, schemas, output, paths, writer, engine | 42 / 42 |
| Phase 2 — Atoms, Molecules, Layouts | All template tiers + tests | 37 / 37 |
| Phase 3 — Agent-Friendly JSON API | Errors, list, inspect, batch, dry-run, verbose | 18 / 18 |
| Phase 4 — Command Handlers | All 12 commands + utilities | 20 / 20 |
| Phase 5 — Hardening | Prettier, embedding, e2e, flags, release | 14 / 14 |
| Phase 6 — Template Versioning | Atom versioning, breaking change detection | 0 / 7 |
| Phase 7 — Custom Template Plugins | Plugin system, plugin API | 0 / 7 |
| Phase 8 — WebSocket Dev Server | Watch mode, WebSocket events | 0 / 6 |
| Phase 9 — Enhanced Learning | Semantic search, enhanced explain | 0 / 6 |
| Phase 10 — Registry Publishing | Publish command, registry ecosystem | 0 / 6 |
| Phase 11 — Additional Frameworks | New registries, integration patterns | 0 / 7 |
| **Total** | | **131 / 210** |
