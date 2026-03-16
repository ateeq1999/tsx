# TSX CLI — Bug & Improvement Tracker

> One checkbox per issue. Fix in order. Commit after each. Update `[x]` when done.

---

## 🔴 Critical — Correctness

- [x] **BUG-01** `init`: silent failure — `npm install` and `shadcn init` errors are swallowed; user gets success with missing files
- [x] **BUG-02** `add_auth`: silent failure — `npm install better-auth` failure returns empty vec, no warning surfaced to user
- [x] **BUG-03** `add_migration`: returns `success: true` even when drizzle-kit generate/migrate fail (failures demoted to warnings)
- [x] **BUG-04** `add_auth_guard`: fragile string replacement — `content.replace("export const Route = createFileRoute", ...)` inserts guard code in wrong position; guard must be inside the route options object, not appended to the function call
- [x] **BUG-05** `add_table`: wrong schema type — accepts `AddFormArgs` (includes `submit_fn`, `layout`) instead of its own `AddTableArgs`; table-specific columns/sorting/pagination options are absent

---

## 🟠 High — Quality

- [x] **BUG-06** Input validation missing on all generate commands — no check that `name` is a valid TypeScript identifier, `fields` is non-empty, `path` has no double slashes or traversal
- [x] **BUG-07** Plugin overrides never applied — `plugin install` copies files to `.tsx/plugins/` but `render_and_write()` never checks for plugin template overrides before rendering built-in templates
- [x] **BUG-08** `upgrade atoms` only pins version — does not copy updated atom template files into the project; the command name implies an actual upgrade but only writes `tsx.atomsVersion` to package.json
- [x] **BUG-09** Inconsistent error representation — commands use three different error formats: `CommandResult` string errors, `ResponseEnvelope + ErrorResponse`, and `BatchError`; consumers must handle all three

---

## 🟡 Medium — Robustness

- [x] **BUG-10** `dev --ws-port` thread leak — `JoinHandle` from `start_ws_server()` is immediately dropped, which detaches the thread silently; should be stored and awaited on shutdown
- [x] **BUG-11** `batch` has no rollback — if command N of M fails mid-scaffold, commands 1..N-1 already wrote files; no cleanup, no atomicity, no record of partial state
- [x] **BUG-12** No structured logging — errors go to `eprintln!` with no context about which command, which template, or which file was being processed

---

## 🟢 Improve — Elevation

- [ ] **BUG-13** `explain` knowledge base is hardcoded Rust — 400-line match expression baked into source; adding/editing topics requires recompile; should live in `data/explain.json`
- [ ] **BUG-14** Stub framework registries — `vue`, `svelte`, `clerk`, `react` registries are minimal placeholders; `tsx ask --framework vue` returns empty results
- [ ] **BUG-15** Zero command unit tests — 44 command files, 0 unit tests; only the render engine and file writer are tested

---

## Progress

| ID | Description | Status |
|----|-------------|--------|
| BUG-01 | init silent failures | ✅ fixed |
| BUG-02 | add_auth silent npm failure | ✅ fixed |
| BUG-03 | add_migration returns success on failure | ✅ fixed |
| BUG-04 | add_auth_guard fragile string replace | ✅ fixed |
| BUG-05 | add_table wrong schema type | ✅ fixed |
| BUG-06 | Missing input validation | ✅ fixed |
| BUG-07 | Plugin overrides not wired | ✅ fixed |
| BUG-08 | upgrade only pins, doesn't upgrade | ✅ fixed |
| BUG-09 | Inconsistent error formats | ✅ fixed |
| BUG-10 | WebSocket thread leak | ✅ fixed |
| BUG-11 | Batch no rollback | ✅ fixed |
| BUG-12 | No structured logging | ✅ fixed |
| BUG-13 | explain hardcoded knowledge base | ⬜ pending |
| BUG-14 | Stub framework registries | ⬜ pending |
| BUG-15 | Zero command unit tests | ⬜ pending |
