# TSX Framework Protocol — Implementation Plan

> Universal AI Agent Integration Layer for Frameworks & Packages

---

## Phase 1 — Foundation

### 1.1 Create framework module

- [x] Create `src/framework/` directory with `mod.rs`
- [x] Create `src/framework/registry.rs` — Framework registry types
- [x] Create `src/framework/loader.rs` — Load registries from local/npm
- [ ] Create `src/framework/conventions.rs` — Convention parsing
- [x] Add framework module to `lib.rs`

### 1.2 Registry types

- [x] Define `FrameworkRegistry` struct
- [x] Define `FrameworkInfo` struct (slug, name, version, category, docs)
- [x] Define `Convention` struct (file patterns, naming, injection points)
- [x] Define `Integration` struct (package, setup steps, patterns)
- [x] Define `Question` struct (topic, answer, related)

### 1.3 Registry loader

- [x] Implement `load_builtin_frameworks()` — Load from frameworks/ directory
- [ ] Implement `load_registry_from_npm(slug)` — Load from npm package
- [x] Implement `load_registry_from_path(path)` — Load from local directory
- [ ] Implement `discover_frameworks()` — Find all available frameworks

### 1.4 Framework definitions

- [x] Create `frameworks/tanstack-start/registry.json`
- [ ] Create `frameworks/tanstack-start/conventions.json`
- [x] Create `frameworks/drizzle-orm/registry.json`
- [x] Create `frameworks/better-auth/registry.json`

### 1.5 CLI integration

- [x] Add `list --frameworks` command
- [x] Add new commands to main.rs: `ask`, `where`, `how`, `explain`

---

## Phase 2 — Query Interface

### 2.1 Ask command

- [x] Create `src/commands/ask.rs`
- [x] Implement `ask` — Answer framework questions
- [x] Implement topic matching algorithm
- [x] Return steps, files, dependencies

### 2.2 Where command

- [x] Create `src/commands/where.rs`
- [x] Implement `where` — Query file locations
- [x] Match "thing" to convention patterns
- [x] Return path, pattern, conventions

### 2.3 How command

- [x] Create `src/commands/how.rs`
- [x] Implement `how` — Integration how-tos
- [x] Lookup package integration patterns
- [x] Return install cmd, setup steps, file patterns

---

## Phase 3 — Learning Mode

### 3.1 Explain command

- [x] Create `src/commands/explain.rs`
- [x] Implement `explain` — Template decision explanations
- [x] Create decision knowledge base
- [x] Return purpose, decisions, learn more links

### 3.2 Decision knowledge base

- [x] Add decision explanations to template metadata
- [x] Build question/answer index
- [x] Implement semantic matching for questions

---

## Phase 4 — Ecosystem

### 4.1 Registry publishing

- [x] Design npm package format for third-party registries
- [x] Implement registry validation
- [x] Build registry submission flow

### 4.2 Framework definitions

- [x] Add more built-in frameworks (React, Vue, Svelte, Next.js)
- [x] Add popular ORMs (Prisma, Kysely)
- [x] Add popular auth solutions (Clerk, Auth.js)

---

## Checklist Summary

| Phase | Tasks | Done |
|---|---|---|
| Phase 1 — Foundation | Module, types, loader, definitions | 13 / 23 |
| Phase 2 — Query Interface | ask, where, how commands | 9 / 9 |
| Phase 3 — Learning | explain, knowledge base | 5 / 5 |
| Phase 4 — Ecosystem | Publishing, more frameworks | 5 / 5 |
| **Total** | | **32 / 42** |
