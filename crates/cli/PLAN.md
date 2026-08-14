# CLI Architecture Rewrite Plan

## 1. Why

`crates/cli/src` grew to ~25,000 lines across ~110 files by appending, not
by design. Three concrete problems, found by reading the tree:

### 1a. `main.rs` is a 2,016-line god file
It holds the *entire* clap surface (55+ top-level commands, ~30 subcommand
enums, ~1,200 lines) **and** the entire dispatch match statement (~800
lines) **and**, for four commands (`Docs`, `Fmt`, `Watch`, `Tui`), the full
business logic inline (main.rs:1704-1843) instead of delegating to
`commands/`. Nobody can find "the docs command" without reading main.rs
top to bottom.

### 1b. `commands/` is not self-explanatory
Two different organizing principles fight each other:

- `commands/mod.rs` groups files into `generate/`, `manage/`, `ops/`,
  `query/`, `package/` — categories that don't correspond to anything a
  user or contributor thinks in. `ops/` alone contains 40 unrelated
  files: `pattern.rs`, `registry.rs`, `adb.rs`, `flutter.rs`, `port.rs`,
  `mcp.rs`, `repl.rs`, `migrate.rs`, `audit.rs`... there's no shared
  concept named "ops."
- **19 dead files sit directly under `commands/`**, duplicating names
  that already exist under the grouped folders, and are wired into
  **nothing** — `commands/mod.rs` never declares `mod add_auth;` etc. at
  the top level, only `pub mod generate/manage/ops/query/package;`.
  Confirmed via grep — none of these are referenced by any `mod`
  declaration anywhere in the crate:

  ```
  commands/add_auth.rs        commands/add_schema.rs      commands/ask.rs
  commands/add_auth_guard.rs  commands/add_seed.rs        commands/batch.rs
  commands/add_feature.rs     commands/add_server_fn.rs   commands/explain.rs
  commands/add_form.rs        commands/add_table.rs       commands/how.rs
  commands/add_migration.rs   commands/add_page.rs        commands/init.rs
  commands/add_query.rs       commands/inspect.rs         commands/list.rs
                                                            commands/where_cmd.rs
  ```

  This is the single biggest source of "which file do I edit?" confusion
  — e.g. `add_auth.rs` (50 lines, dead) sits next to `manage/add_auth.rs`
  (54 lines, live) with no signal which one runs.

### 1c. God files mixing types + orchestration + helpers
Several single files each hold an entire subcommand tree's types, every
action handler, *and* their private helpers with no internal boundary:

| File | Lines | Contains |
|---|---|---|
| `commands/ops/pattern.rs` | 1,712 | 4 structs + 14 public actions (`pattern_new`, `pattern_run`, `pattern_install`, `pattern_lint`, `pattern_add`, `pattern_record_*`, `pattern_list*`, `pattern_show`, `pattern_remove`, `pattern_publish`, `pattern_search`, `pattern_share`, `pattern_eject`, `pattern_update`) + 15 private helpers (github/registry download, tar bundling, snapshotting, hashing) |
| `commands/ops/registry.rs` | 1,165 | search/install/list/website/update/info |
| `commands/manage/framework_cmd.rs` | 955 | init/validate/preview/add/list/publish |
| `commands/ops/codegen.rs` | 681 | rust-to-ts / openapi-to-zod / drizzle-to-zod |
| `commands/ops/batch.rs` | 555 | batch execution + batch planning |
| `commands/ops/pkg.rs` | 523 | install/info/upgrade/publish |
| `stack/mod.rs` | 522 | init/show/add/remove/detect |

None of these have a `types.rs`/`utils.rs` split — a bug fix to one
action requires scrolling past thirteen unrelated ones.

## 2. Goals

1. **One command → one folder.** Given `tsx pattern install ...`, a
   contributor should be able to guess the path is
   `commands/pattern/install.rs` without grepping.
2. **`main.rs` does one thing: bootstrap.** Argument parsing lives in
   `cli/`, dispatch lives in `cli/dispatch.rs`, `main.rs` shrinks to
   ~15 lines.
3. **No file mixes types, orchestration, and helpers.** Every command
   folder gets the same three-way split, so the split itself becomes
   unsurprising.
4. **Zero behavior change.** This is a structural rewrite, not a
   rewrite of what the CLI does. Flags, JSON output shape
   (`ResponseEnvelope`), and command names are unchanged. Verified via
   the existing `tests/e2e.rs` and the CLI's own `tsx snapshot diff`
   generator-regression tool.

## 3. SOLID mapping (what each principle means concretely here)

| Principle | Current violation | Fix |
|---|---|---|
| **S**RP | `pattern.rs` does parsing, git/tar/http I/O, hashing, and 14 command actions in one file | Split into `commands/pattern/{types,utils,new,run,install,...}.rs` |
| **O**CP | Adding a command means editing the 800-line match in `main.rs` in the correct one of 55 arms | Dispatch stays a match (see §6 — a trait registry is rejected), but each arm becomes one line: `pattern::install::run(args, &ctx)`. New commands add a folder + one match arm, never touch another command's file |
| **L**SP | N/A directly (no polymorphic command types today) | If/when a `Command` trait is introduced (§6, deferred), every impl must honor "returns `ResponseEnvelope`, never panics on user input" — already true by convention, made explicit in the trait's doc comment |
| **I**SP | Handlers take 4-6 loose params (`cli.overwrite, cli.dry_run, cli.verbose, cli.diff`) — every handler depends on the full `Cli` shape even when it uses one field | Introduce `ExecutionContext { overwrite, dry_run, verbose, diff, json_input }`, pass by reference |
| **D**IP | Commands call concrete `framework::registry`, `packages::PackageStore`, filesystem functions directly, making them slow to test in isolation | Not a full DI rewrite (would be over-engineering for a CLI) — but shared infra (`render/`, `framework/`, `packages/`, `json/`) stays as the dependency-inversion seam commands already call *through*; §6 explains why we don't go further |

## 4. Target structure

```
crates/cli/src/
├── main.rs                  # ~15 lines: read stdin/--file, call cli::run()
├── cli/
│   ├── mod.rs                # pub fn run() -> reads Cli::parse(), calls dispatch
│   ├── args.rs                # the Cli struct + Command enum + all subcommand enums
│   │                           # (pure clap definitions, ported verbatim from main.rs, no logic)
│   ├── context.rs              # ExecutionContext { overwrite, dry_run, verbose, diff, json_input }
│   └── dispatch.rs              # the match statement — one line per arm, delegates only
│
├── commands/
│   ├── mod.rs                  # pub mod <one per command, flat>
│   │
│   ├── init/                   # `tsx init`
│   │   ├── mod.rs                # pub use handler::run
│   │   ├── handler.rs
│   │   ├── types.rs
│   │   └── utils.rs
│   │
│   ├── dev/                    # `tsx dev`
│   ├── list/                   # `tsx list`
│   ├── create/                 # `tsx create`
│   ├── inspect/                 # `tsx inspect`
│   ├── batch/                   # `tsx batch` (execute + plan submodes)
│   ├── run/                     # `tsx run` (universal generator dispatcher)
│   ├── context/                  # `tsx context`
│   ├── doctor/                    # `tsx doctor`
│   ├── analyze/                    # `tsx analyze`
│   ├── audit/                       # `tsx audit`
│   ├── build/                        # `tsx build`
│   ├── test_run/                      # `tsx test`
│   ├── migrate/                        # `tsx migrate`
│   ├── lint_template/                   # `tsx lint-template`
│   ├── repl/                             # `tsx repl`
│   ├── subscribe/                         # `tsx subscribe`
│   ├── plan/                               # `tsx plan`
│   ├── docs/                                # `tsx docs`   (moved out of main.rs)
│   ├── fmt/                                  # `tsx fmt`    (moved out of main.rs)
│   ├── watch/                                 # `tsx watch`  (moved out of main.rs)
│   ├── tui/                                    # `tsx tui`    (moved out of main.rs)
│   ├── lsp/                                     # `tsx lsp`
│   ├── mcp/                                      # `tsx mcp`
│   │
│   ├── generate/                # `tsx generate <sub>` — thin per-action files,
│   │   ├── mod.rs                  #   these are ~20-35 lines each already, no further split needed
│   │   ├── types.rs                 #   shared GenerateRequest-ish types, if any duplication is found
│   │   ├── feature.rs
│   │   ├── schema.rs
│   │   ├── server_fn.rs
│   │   ├── query.rs
│   │   ├── form.rs
│   │   ├── table.rs
│   │   ├── page.rs
│   │   └── seed.rs
│   │
│   ├── add/                     # `tsx add <sub>`
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── auth_guard.rs
│   │   └── migration.rs
│   │
│   ├── framework/                # `tsx framework <sub>` (was manage/framework_cmd.rs, 955 lines)
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   ├── utils.rs                # shared: manifest read/write, tarball, npm publish helpers
│   │   ├── init.rs
│   │   ├── validate.rs
│   │   ├── preview.rs
│   │   ├── add.rs
│   │   ├── list.rs
│   │   └── publish.rs
│   │
│   ├── pattern/                  # `tsx pattern <sub>` (was ops/pattern.rs, 1,712 lines)
│   │   ├── mod.rs
│   │   ├── types.rs                # PatternArg, PatternOutput, PatternSlot, PatternDefinition, RecordSession
│   │   ├── utils.rs                # snapshot_dir, hex_first64, templatize_path, copy_dir_all, tempfile_dir,
│   │   │                            # parse_args_spec, lint_schema_directive, bundle_pack_dir, urlencoding_simple
│   │   ├── new.rs
│   │   ├── run.rs
│   │   ├── install.rs               # install_local / install_github / install_registry / install_builtin
│   │   ├── lint.rs
│   │   ├── add.rs
│   │   ├── record.rs                # start + stop
│   │   ├── list.rs                  # list + list_builtin
│   │   ├── show.rs
│   │   ├── remove.rs
│   │   ├── publish.rs
│   │   ├── search.rs
│   │   ├── share.rs
│   │   ├── eject.rs
│   │   └── update.rs
│   │
│   ├── registry/                  # `tsx registry <sub>` (was ops/registry.rs, 1,165 lines)
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   ├── utils.rs
│   │   ├── search.rs
│   │   ├── install.rs
│   │   ├── list.rs
│   │   ├── website.rs
│   │   ├── update.rs
│   │   └── info.rs
│   │
│   ├── pkg/                        # `tsx pkg <sub>` (was ops/pkg.rs, 523 lines)
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   ├── utils.rs
│   │   ├── install.rs
│   │   ├── info.rs
│   │   ├── upgrade.rs
│   │   └── publish.rs
│   │
│   ├── template/                    # `tsx template <sub>`
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   ├── list.rs
│   │   ├── info.rs
│   │   ├── init.rs
│   │   ├── install.rs
│   │   ├── uninstall.rs
│   │   ├── schema.rs
│   │   ├── lint.rs
│   │   ├── config.rs                 # show/set/init (small enough to stay one file)
│   │   ├── login.rs
│   │   ├── logout.rs
│   │   └── publish.rs
│   │
│   ├── package/                       # `tsx package <sub>` (registry package authoring)
│   │   ├── mod.rs
│   │   ├── new.rs
│   │   ├── validate.rs
│   │   ├── pack.rs
│   │   ├── publish.rs
│   │   └── install.rs
│   │
│   ├── stack/                          # `tsx stack <sub>` (was top-level stack/mod.rs, 522 lines)
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   ├── init.rs
│   │   ├── show.rs
│   │   ├── add.rs
│   │   ├── remove.rs
│   │   └── detect.rs
│   │
│   ├── codegen/                         # `tsx codegen <sub>` (was ops/codegen.rs, 681 lines)
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   ├── rust_to_ts.rs
│   │   ├── openapi_to_zod.rs
│   │   └── drizzle_to_zod.rs
│   │
│   ├── snapshot/                         # `tsx snapshot <sub>`
│   ├── replay/                            # `tsx replay <sub>`
│   ├── atoms/                              # `tsx atoms <sub>`
│   ├── config/                              # `tsx config <sub>`
│   ├── env/                                  # `tsx env <sub>`
│   ├── plugin/                                # `tsx plugin <sub>` (commands layer — distinct from src/plugin/ infra)
│   ├── publish/                                # `tsx publish <sub>`
│   ├── upgrade/                                 # `tsx upgrade <sub>` (atoms + cli)
│   ├── auth/                                     # `tsx login/logout/whoami`
│   ├── adb/                                       # `tsx adb <sub>`
│   ├── flutter/                                    # `tsx flutter <sub>`
│   ├── port/                                        # `tsx port <sub>`
│   ├── path/                                         # `tsx path`
│   └── query/                                         # `tsx ask/where/how/explain/describe`
│       ├── mod.rs                                       #   (kept as one folder — these are 5 independent
│       ├── ask.rs                                         #    verb commands, not subcommands of one noun,
│       ├── where_cmd.rs                                     #   but small enough not to need types/utils split)
│       ├── how.rs
│       ├── explain.rs
│       └── describe.rs
│
├── framework/    # unchanged — shared infra: detection, registry loading, knowledge base
├── render/       # unchanged — template engine
├── packages/     # unchanged — registry package install/store
├── plugin/       # unchanged — plugin manifest/validator (infra, not the `commands/plugin` folder)
├── schemas/      # unchanged — shared request schema types used by generate/*
├── json/         # unchanged — ResponseEnvelope / ErrorResponse contract
├── output.rs     # unchanged
└── utils/        # unchanged — shared low-level helpers (paths, imports, barrel, write, format, validate)
```

**Per-command file contract** (what goes where, so the split stays
boring and predictable):

- `types.rs` — request/response/domain structs and enums for this
  command only. If a type is used by exactly one action file, it can
  live in that file instead — don't create `types.rs` for a single
  struct.
- `utils.rs` — private helpers used by ≥2 action files in this command.
  A helper used by only one action stays in that action's file.
- `<action>.rs` — one public `pub fn run(...) -> ResponseEnvelope`
  (or the existing naming the file already used, e.g. `pattern_install`)
  per subcommand action, plus any truly-private helpers only it needs.
- `mod.rs` — `pub mod` declarations + re-exports only. No logic.

Single-action commands (`init`, `dev`, `list`, `build`, `doctor`, ...)
don't get the full four-file treatment — `mod.rs` + `handler.rs` is
enough, and `types.rs`/`utils.rs` are added only when the handler
actually needs them (most are 20-90 lines today and are fine as
`mod.rs` + `handler.rs`).

## 5. `commands/mod.rs` after the rewrite

Flat, one line per command, no grouping re-exports, no ambiguity:

```rust
pub mod add;
pub mod adb;
pub mod atoms;
pub mod auth;
pub mod batch;
// ... (55 lines, alphabetical, one per top-level noun)
pub mod watch;
```

No file lives directly under `commands/` except `mod.rs` — anything
that isn't a folder is either not a command, or is dead code we deleted
in Phase 0.

## 6. Explicitly rejected / deferred

- **A `Command` trait + registry** (`trait CliCommand { fn run(&self,
  ctx) -> ResponseEnvelope }` with a `HashMap<&str, Box<dyn
  CliCommand>>`) was considered for OCP. Rejected for now: clap's
  `#[derive(Subcommand)]` enum is already the extension point — adding
  a command means adding one enum variant and one match arm, which is
  exactly as open/closed as a registry would be, without the
  boxed-trait-object indirection or the loss of clap's compile-time
  exhaustiveness checking on the match. Revisit only if a real second
  consumer of "list of commands as data" shows up (e.g. a plugin system
  that registers commands at runtime).
- **Full dependency-injection of `render/`, `framework/`, `packages/`**
  behind traits. These are already single well-scoped modules called
  the same way from every command; adding trait indirection now would
  be speculative (no second implementation exists or is planned).
- **Renaming CLI-facing flags/commands.** Out of scope — this plan is
  file-tree-only. `Cli`/`Command` enum variant names, doc comments (=
  `--help` text), and JSON payload shapes are ported verbatim.

## 7. Migration phases

Each phase ends with `cargo build`, `cargo test -p tsx`, and (for
generator-touching phases) `tsx snapshot diff` to catch any accidental
output change, before moving to the next phase. Commit per phase.

1. **Phase 0 — Delete dead weight.** Remove the 19 orphaned files listed
   in §1b. Zero risk: nothing references them, `cargo build` is the
   verification. Also delete the leftover `commands/package.rs` if it
   turns out to duplicate `commands/manage`... *(verify at execution
   time — package.rs is the one root-level file that IS wired in, via
   `pub mod package;`, so it stays, just gets folderized in Phase 4)*.

2. **Phase 1 — Extract `cli/`.** Move the `Cli`/`Command`/subcommand
   enums out of `main.rs` into `cli/args.rs` verbatim. Move the match
   statement into `cli/dispatch.rs` verbatim. `main.rs` becomes the
   ~15-line bootstrap. Pure move, no logic change — lowest-risk, highest
   immediate readability win.

3. **Phase 2 — Introduce `ExecutionContext`.** Add the struct, thread it
   through `dispatch.rs` instead of four loose `cli.x` fields. Update
   every handler signature. Mechanical, compiler-guided (won't compile
   until every call site is fixed).

4. **Phase 3 — Move `Docs`/`Fmt`/`Watch`/`Tui` logic out of
   `dispatch.rs`** into `commands/docs/`, `commands/fmt/`,
   `commands/watch/`, `commands/tui/`. These are the only commands with
   business logic outside `commands/` today.

5. **Phase 4 — Folderize one command group at a time**, smallest/lowest-risk
   first so the pattern gets validated before tackling the big files:
   `path` → `port` → `adb` → `config` → `env` → `atoms` → `auth` →
   `upgrade` → `publish` → `snapshot` → `replay` → `plugin` → `package`
   → `add` → `generate` → `stack` → `codegen` → `pkg` → `template` →
   `framework` → `registry` → `pattern` (last: biggest file, most
   subcommands, most I/O helpers to carve into `utils.rs`).

6. **Phase 5 — Flatten `commands/mod.rs`**, dropping the
   `generate/manage/ops/query` umbrella re-exports now that every
   command is already its own top-level folder.

## 8. Non-goals (confirm before starting)

- Not changing what any command does, its flags, or its `--help` text.
- Not touching `framework/`, `render/`, `packages/`, `plugin/`,
  `schemas/`, `json/`, `utils/`, `output.rs` — these are shared infra,
  already reasonably scoped, and out of the "commands" complaint this
  plan addresses.
- Not a rewrite of business logic inside handlers — code moves between
  files, but a handler's internal logic changes only where lifting a
  helper into `utils.rs` requires it (e.g. taking `&Path` instead of a
  captured local).

## 9. Open decisions for you

1. **Phase granularity** — do this as one long-lived branch merged at
   the end, or land each phase as its own PR? Given 25k lines and a
   solo-maintained repo, landing per-phase (§7) is recommended so a
   regression is bisectable to one command group.
2. **`ops` name** — this plan deletes the `ops` umbrella entirely
   (§4/§5). If there's a reason it exists (e.g. an external tool
   greps for `commands::ops::`), say so before Phase 5.
3. **Where to start** — I'd suggest doing Phase 0 + Phase 1 right now
   (low-risk, immediately makes `main.rs` navigable) and pausing there
   for you to review before touching the 1,700-line `pattern.rs`.
