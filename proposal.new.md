# TSX — Enhanced Proposal
### Code Generation CLI + Universal Framework Protocol

**Version 6.0 · March 2026**

> forge engine · Framework-as-Package · Token-Budgeted Knowledge · Agent-Native CLI

---

## Part 1: The Core Shift

### What Changes in v6

TSX v5 is a code generator for TanStack Start. TSX v6 becomes a **universal agent development toolkit** — any framework author can publish a tsx-compatible package that gives AI agents instant, token-efficient access to their framework's knowledge and code generation.

The key insight: **agent skills are loaded into context (expensive). TSX is called when needed (cheap).**

```
Today — Agent Skills approach:
  Load 4,000-token markdown → agent has all knowledge → always in context
  Cost per task: ~4,000 tokens of overhead

v6 — Framework Protocol CLI approach:
  tsx describe tanstack-start       →  ~80 tokens  (what's available)
  tsx ask "add auth"                →  ~120 tokens (specific answer)
  tsx generate auth --fw tanstack   →  0 tokens    (file written to disk)
  Total per task: ~200 tokens
```

Token reduction per typical agent task: **80–95%**

---

## Part 2: Engine Decision

### 2.1 Name Candidates

The template engine that powers code generation needs a name that reflects its purpose:
composing structured fragments (atoms) into complete source files.

| Name | Metaphor | Fit | Pros | Cons |
|------|----------|-----|------|------|
| **forge** | Industrial forge — raw material shaped under heat | ★★★★★ | Evokes precision manufacturing, "forging" code; short, memorable | None |
| **loom** | Weaving threads into fabric | ★★★★☆ | Maps directly to weaving atoms → molecules; elegant | Less forceful |
| **lattice** | Atomic/molecular lattice structure | ★★★★☆ | Scientific, connects to atoms tier; unique | Harder to type |
| **kiln** | Fires raw clay into finished ceramic | ★★★☆☆ | "Baking" templates into output | Obscure metaphor |
| **weave** | Weaving fragments together | ★★★☆☆ | Clear metaphor | Too generic |
| **sinter** | Joining particles under heat without melting | ★★☆☆☆ | Scientifically precise | Unknown word |

**Recommendation: `forge`**

- `forge` is the engine crate
- Framework packages become: `tsx add @tanstack/start`, `tsx add drizzle-orm`
- The action of building: "TSX forges your code from atoms"

---

### 2.2 Engine Benchmark: MiniJinja vs Tera vs atom-engine vs forge

#### What each is

| Engine | Description |
|--------|-------------|
| **MiniJinja** | Lightweight Jinja2 subset. Current TSX engine. Embeds cleanly in binaries. |
| **Tera** | Full Jinja2 for Rust. More filters, more features, more binary weight. |
| **atom-engine v5** | Built on Tera. Adds: component system, React-style context injection (Provide/Inject), async render, parallel render via Rayon, memory pooling. |
| **forge** (proposed) | Built on Tera. Adds: 4-tier system as first-class concepts, ImportCollector, `token_estimate` metadata, framework package loading. |

---

#### Feature Comparison

| Feature | MiniJinja | Tera | atom-engine | forge (proposed) |
|---------|-----------|------|-------------|-----------------|
| Jinja2 syntax | Subset | Full | Full (via Tera) | Full (via Tera) |
| Built-in filters | ~25 | 50+ | 50+ | 50+ |
| Template inheritance | ✅ | ✅ | ✅ | ✅ |
| Macros | ✅ | ✅ | ✅ | ✅ |
| Component system | ❌ | ❌ | ✅ Props + slots | ✅ Tier-aware |
| Context injection | ❌ | ❌ | ✅ Provide/Inject | ✅ |
| Import hoisting | Custom (ours) | Custom needed | Custom needed | ✅ Built-in |
| Async render | ❌ | ❌ | ✅ (Tokio) | Optional |
| Parallel render | ❌ | ❌ | ✅ (Rayon) | Optional |
| 4-tier system | Manual | Manual | Manual | ✅ Built-in |
| Token metadata | ❌ | ❌ | ❌ | ✅ Built-in |
| Framework loading | ❌ | ❌ | ❌ | ✅ Built-in |
| Binary size impact | ~350KB | ~900KB | ~1.4MB | ~1.0MB |
| Cold-start overhead | Very low | Low | Moderate | Low |

---

#### Performance Benchmark (projected — methodology below)

**Test: Render a complete feature file (schema + server fn + query + form + page)**

The benchmark simulates a realistic agent workload: `tsx add:feature products --fields name,price,qty`.
This exercises the full atom → molecule → layout → feature render pipeline.

| Engine | Simple atom render | Full feature render | Binary size delta | Init time |
|--------|-------------------|---------------------|-------------------|-----------|
| MiniJinja (current) | ~0.08ms | ~1.2ms | +350KB | <1ms |
| Tera | ~0.12ms | ~1.8ms | +900KB | ~2ms |
| atom-engine v5 | ~0.10ms* | ~1.4ms* | +1.4MB | ~3ms |
| **forge on Tera** | ~0.12ms | ~1.6ms | ~1.0MB | ~2ms |

*atom-engine parallel mode would outperform on batch operations (10+ files)

**Batch benchmark (tsx batch — 10 features in parallel):**

| Engine | 10 features sequential | 10 features parallel |
|--------|----------------------|---------------------|
| MiniJinja | ~12ms | N/A |
| Tera | ~18ms | N/A |
| atom-engine (Rayon) | ~14ms | ~4ms |
| **forge (Rayon opt-in)** | ~16ms | ~5ms |

> **Methodology:** Numbers derived from Tera and MiniJinja published benchmarks (see benches/ in each repo),
> atom-engine component overhead estimated at 15% above bare Tera based on the component resolution pass,
> forge numbers assume Tera base + ImportCollector overhead (~0.3ms per render for collector drain).
> Full empirical benchmarks to be run in Phase A.1 via `cargo bench`.

---

#### Decision Matrix

| Criterion | Weight | MiniJinja | Tera | atom-engine | forge on Tera |
|-----------|--------|-----------|------|-------------|---------------|
| Binary size | 20% | ★★★★★ | ★★★☆☆ | ★★☆☆☆ | ★★★★☆ |
| Filter richness | 15% | ★★☆☆☆ | ★★★★★ | ★★★★★ | ★★★★★ |
| 4-tier system | 25% | ★★☆☆☆ | ★★☆☆☆ | ★★★☆☆ | ★★★★★ |
| Framework authoring DX | 20% | ★★☆☆☆ | ★★★☆☆ | ★★★☆☆ | ★★★★★ |
| Import hoisting | 10% | ★★★☆☆ | ★★☆☆☆ | ★★☆☆☆ | ★★★★★ |
| Batch performance | 10% | ★★☆☆☆ | ★★☆☆☆ | ★★★★☆ | ★★★★☆ |
| **Weighted score** | | **2.55** | **3.05** | **3.25** | **4.65** |

**Verdict: Build `forge` as a new crate on top of Tera.**
- Tera gives us 50+ filters for free over MiniJinja
- atom-engine's component/slot pattern maps directly to our 4-tier system — adapt it
- Adding ImportCollector and tier-awareness natively means zero boilerplate for framework authors
- The binary size delta over MiniJinja (~650KB) is acceptable for the authoring DX gained

---

## Part 3: forge — The Engine Crate

### 3.1 What forge is

`forge` is a Rust template engine crate for **structured code generation**. It is built on Tera and adds:

1. **4-tier system as first-class concepts** — `Atom`, `Molecule`, `Layout`, `Feature` are engine-level constructs, not just directory conventions
2. **ImportCollector built-in** — import hoisting is part of the render context, not bolted on
3. **Token metadata** — every template file can declare `token_estimate` for knowledge queries
4. **Framework package loading** — load templates from a directory, npm package, or embedded bytes

```
forge (crate)
├── Engine (Tera wrapper)
│   ├── load_tier(path, Tier::Atom | Molecule | Layout | Feature)
│   ├── render(name, context) → String
│   └── render_feature(name, context) → ForgeOutput
├── ImportCollector (thread-local, drain on render_imports())
├── TierRegistry (tracks which templates are which tier)
├── TokenMetadata (reads frontmatter from knowledge .md files)
└── FrameworkLoader (loads framework packages from disk/npm/embedded)
```

### 3.2 forge API (framework author perspective)

A framework author using `forge` to test their templates locally:

```rust
use forge::{Engine, Context, Tier};

let mut engine = Engine::new();
engine.load_dir("./my-framework/generators/templates", Tier::Auto)?;

let ctx = Context::new()
    .insert("name", "users")
    .insert("fields", &fields);

let output = engine.render_feature("add-auth", &ctx)?;
println!("{}", output.code);
println!("Imports: {:?}", output.imports);
```

### 3.3 forge Template Format

Framework authors write `.forge` files (Tera syntax + forge extensions):

```jinja
{# forge:tier atom #}
{# forge:token_estimate 12 #}
{# forge:tags [drizzle, column] #}

{% if field.field_type == "text" %}
  {{ field.name | snake_case }}: text('{{ field.name }}'){{ collect_import("text", "drizzle-orm/sqlite-core") }},
{% elif field.field_type == "int" %}
  {{ field.name | snake_case }}: integer('{{ field.name }}'){{ collect_import("integer", "drizzle-orm/sqlite-core") }},
{% endif %}
```

The `{# forge:tier #}` directive registers the template at the correct tier. `collect_import` is a built-in forge filter. `render_imports()` is a built-in forge function that drains the collector.

### 3.4 forge vs atom-engine — What We Borrow

From atom-engine v5 we adopt:
- **Component slots** → mapped to Layout `{% block %}` slots
- **Provide/Inject context** → mapped to our tier context propagation (atoms inherit molecule context)
- **Parallel render** (Rayon) → opt-in for batch operations

What we add that atom-engine doesn't have:
- Tier-aware registry (know which tier a template belongs to at load time)
- ImportCollector as a first-class engine feature
- Knowledge file frontmatter parsing (`token_estimate`, `tags`, `requires`)
- Framework package loading protocol

---

## Part 4: Framework Package Standard

### 4.1 Directory Layout

A tsx-compatible framework package is published to npm as `@tsx-pkg/<name>`:

```
@tsx-pkg/tanstack-start/
  manifest.json           ← package identity, version, category, peer deps
  knowledge/
    overview.md           ← what this framework is  (≤ 150 tokens)
    concepts.md           ← key terms + glossary
    patterns.md           ← common code patterns with snippets
    faq.md                ← Q&A pairs (structured frontmatter)
    decisions.md          ← design rationale
  generators/
    manifest.json         ← available generators + JSON input schemas
    templates/
      atoms/              ← indivisible code fragments (.forge files)
      molecules/          ← composed blocks
      layouts/            ← file-level shells
      starters/           ← full project scaffold definitions
  integrations/
    better-auth.json      ← integration pattern for better-auth
    drizzle-orm.json
    shadcn-ui.json
```

### 4.2 manifest.json

```json
{
  "id": "tanstack-start",
  "name": "TanStack Start",
  "version": "1.0.0",
  "category": "framework",
  "description": "Full-stack React meta-framework",
  "docs": "https://tanstack.com/start",
  "peer_dependencies": {
    "@tanstack/start": "^1.0",
    "@tanstack/react-router": "^1.0"
  },
  "knowledge_token_budget": {
    "overview": 150,
    "concepts": 400,
    "patterns": 600,
    "faq_per_entry": 120
  },
  "generators": ["init", "add-route", "add-query", "add-form", "add-feature"],
  "starters": ["basic", "with-auth", "saas", "admin"]
}
```

### 4.3 Knowledge File Format

Every knowledge `.md` file uses frontmatter for token-efficient retrieval. The CLI uses `token_estimate` to compose responses within a requested budget without reading full file content.

```markdown
---
id: add-auth
question: How do I add authentication?
tags: [auth, security, setup]
token_estimate: 120
requires: [better-auth]
related: [add-migration, add-auth-guard]
---

## Adding Authentication

Run `tsx add:auth --provider email` to scaffold Better Auth.
This creates `lib/auth.ts` with the auth configuration...
```

### 4.4 Generator Manifest

```json
{
  "generators": [
    {
      "id": "add-feature",
      "description": "Scaffold a complete CRUD feature module",
      "input_schema": {
        "name": { "type": "string", "required": true },
        "fields": { "type": "array", "items": { "$ref": "#/FieldSchema" } },
        "auth": { "type": "boolean", "default": false }
      },
      "output": {
        "files": [
          "db/schema/{{name}}.ts",
          "server-functions/{{name}}.ts",
          "routes/{{name}}/index.tsx"
        ]
      },
      "next_steps": ["tsx add:migration {}"]
    }
  ]
}
```

### 4.5 Starter Templates

```json
{
  "id": "saas",
  "name": "SaaS Starter",
  "description": "Full-stack app with auth, DB, billing, and dashboard",
  "token_estimate": 40,
  "steps": [
    { "cmd": "init", "args": {} },
    { "cmd": "add:auth", "args": { "provider": "email" } },
    { "cmd": "add:schema", "args": { "name": "users", "timestamps": true } },
    { "cmd": "add:migration", "args": {} },
    { "cmd": "add:feature", "args": { "name": "dashboard" } }
  ]
}
```

---

## Part 5: CLI Enhancements

### 5.1 Token Budget System

Every knowledge command accepts `--depth`:

```bash
tsx ask "how to add auth" --fw tanstack-start --depth brief    # ~50 tokens
tsx ask "how to add auth" --fw tanstack-start                  # ~150 tokens (default)
tsx ask "how to add auth" --fw tanstack-start --depth full     # ~400 tokens
```

All knowledge responses include `token_estimate` in metadata:

```json
{
  "answer": "Run tsx add:auth --provider email...",
  "token_estimate": 112,
  "depth": "default",
  "related": ["add-migration", "add-auth-guard"],
  "next_command": "tsx add:auth --provider email"
}
```

### 5.2 New Commands

#### `tsx describe <framework>`

Agent's entry point for any framework. Returns what knowledge is available and its token cost — before the agent commits to loading anything.

```bash
tsx describe tanstack-start
```

```json
{
  "framework": "TanStack Start",
  "version": "1.0.0",
  "available_knowledge": {
    "overview": { "token_estimate": 150, "cmd": "tsx describe tanstack-start --section overview" },
    "concepts": { "token_estimate": 400, "cmd": "tsx describe tanstack-start --section concepts" },
    "faq_topics": 28
  },
  "generators": 8,
  "starters": ["basic", "with-auth", "saas"],
  "quick_start": "tsx create --from tanstack-start --starter basic"
}
```

#### `tsx create`

Universal scaffold — replaces `tsx init`:

```bash
tsx create --from tanstack-start                    # built-in
tsx create --from @tsx-pkg/tanstack-start           # npm package
tsx create --from github:user/my-tsx-pkg            # GitHub repo
tsx create --from tanstack-start --starter saas     # specific starter
tsx create --from tanstack-start --dry-run          # preview steps
```

#### `tsx generate <id> --fw <framework>`

Run a framework-defined generator (not just built-in TSX generators):

```bash
tsx generate add-feature products --fw tanstack-start
tsx generate add-payment-intent --fw @tsx-pkg/stripe
```

#### `tsx framework` (Author Tools)

```bash
tsx framework init <name>                 # scaffold new framework package
tsx framework validate ./my-pkg           # lint manifest + templates
tsx framework preview --template auth.forge --data '{"name":"users"}'
tsx framework publish                     # publish to npm as @tsx-pkg/<name>
tsx framework add <pkg>                   # install framework package locally
tsx framework list                        # list installed framework packages
```

### 5.3 Enhanced `tsx ask` — Multi-Framework Routing

When no `--fw` specified, routes to best-matching framework:

```bash
tsx ask "how to define a schema"
# → detects drizzle-orm in project → routes to drizzle-orm FAQ
# → returns: "Use tsx add:schema --name <table> --fields ..."
```

---

## Part 6: Architecture Overview

```
tsx (CLI binary)
│
├── commands/
│   ├── existing (add:feature, add:schema, etc.)      ← unchanged
│   ├── describe.rs      ← NEW: framework overview
│   ├── create.rs        ← NEW: replaces init, universal scaffold
│   └── generate.rs      ← NEW: framework-defined generators
│
├── framework/
│   ├── registry.rs      ← extended: supports manifest.json format
│   ├── loader.rs        ← extended: npm + GitHub + local loading
│   ├── knowledge.rs     ← NEW: markdown frontmatter parser
│   └── token_budget.rs  ← NEW: token estimate + depth system
│
└── (depends on)
    │
    forge (crate)
    ├── engine.rs         ← Tera wrapper with tier awareness
    ├── collector.rs      ← ImportCollector (moved from tsx)
    ├── tier.rs           ← Atom/Molecule/Layout/Feature types
    ├── context.rs        ← Context builder with Provide/Inject
    └── metadata.rs       ← token_estimate frontmatter reader
```

---

## Part 7: TanStack Start as Reference Implementation

The built-in `frameworks/tanstack-start/` must become the canonical example of the full format. Current state: 2 Q&A entries, minimal templates. Target state:

| Category | Current | Target |
|----------|---------|--------|
| FAQ entries | 2 | 30+ |
| Token-annotated knowledge files | 0 | 5 (overview, concepts, patterns, faq, decisions) |
| Generator templates (forge format) | 0 | All 8 generators |
| Starter templates | 0 | 4 (basic, with-auth, saas, admin) |
| Integration files | 1 (partial) | 5 (better-auth, drizzle, shadcn, react-query, router) |

This becomes the spec every other framework package author follows.

---

## Part 8: Implementation Phases

### Phase A — forge Crate (Foundation)

- [ ] A.1 Run empirical benchmarks: MiniJinja vs Tera on tsx render pipeline — `cargo bench`
- [x] A.2 Create `crates/forge/` with `Cargo.toml` + Cargo workspace setup
- [x] A.3 Wrap Tera engine with tier-aware template loader (`engine.rs`)
- [x] A.4 Move ImportCollector from tsx into forge as a first-class feature (`collector.rs`)
- [x] A.5 Add `forge:tier` and `forge:token_estimate` frontmatter parsing (`metadata.rs`, `tier.rs`)
- [ ] A.6 Add component slots (adapted from atom-engine's slot pattern)
- [ ] A.7 Add Provide/Inject context propagation
- [x] A.8 Write forge crate tests (engine, collector, tier, metadata — all pass)
- [x] A.9 tsx depends on forge via workspace path dep (`crates/forge`)

### Phase B — Framework Package Standard

- [x] B.1 Define and document the full package directory format (manifest.json + knowledge/ + integrations/ + starters/)
- [x] B.2 Refactor `frameworks/tanstack-start/` to use manifest.json + knowledge/ structure
- [x] B.3 Build `knowledge.rs` — markdown frontmatter parser with token_estimate
- [x] B.4 Build `token_budget.rs` — depth system (brief / default / full)
- [ ] B.5 Extend `framework/loader.rs` to load from npm + GitHub + local path
- [ ] B.6 Write validation rules for package manifests

### Phase C — TanStack Start Reference Implementation

- [x] C.1 Write 16 FAQ entries with token estimates (`knowledge/faq.md`)
- [x] C.2 Write overview.md, concepts.md, patterns.md, decisions.md
- [ ] C.3 Migrate all 8 generator templates to forge format
- [x] C.4 Write 3 starter template recipes (basic, with-auth, saas)
- [x] C.5 Write 5 integration files (better-auth, drizzle-orm, shadcn-ui, react-query, react-router)

### Phase D — New CLI Commands

- [x] D.1 `tsx describe` command — framework cost map, per-section retrieval
- [x] D.2 `tsx create` command (replaces `tsx init`)
- [ ] D.3 `tsx generate` command (framework-defined generators)
- [x] D.4 `--depth brief|default|full` flag on `tsx ask`
- [x] D.5 `tsx ask` multi-framework auto-routing (detects from package.json)

### Phase E — Framework Author Tools

- [x] E.1 `tsx framework init <name>` — scaffold new package
- [x] E.2 `tsx framework validate` — lint manifest + templates
- [x] E.3 `tsx framework preview` — render template with test data
- [ ] E.4 `tsx framework publish` — push to npm

### Phase F — External Package Loading

- [ ] F.1 `tsx framework add @tsx-pkg/stripe` — install from npm
- [ ] F.2 `tsx create --from github:user/repo` — load from GitHub
- [x] F.3 Local package loading: `tsx framework add ./my-pkg`
- [ ] F.4 Package caching and version management

---

## Implementation Status (v6 — March 2026)

| Phase | Description | Status |
|-------|-------------|--------|
| A | forge crate (Tera engine, ImportCollector, 4-tier system) | 7/9 ✅ |
| B | Framework Package Standard (manifest, knowledge, token budget) | 4/6 ✅ |
| C | TanStack Start reference implementation | 4/5 ✅ |
| D | New CLI commands (describe, create, --depth, auto-detect) | 4/5 ✅ |
| E | Framework author tools (init, validate, preview) | 3/4 ✅ |
| F | External package loading (local path) | 1/4 ✅ |

---

## Part 9: What Framework Developers Produce

**Summary of deliverables for a framework author:**

| File | Format | Purpose | Token cost |
|------|--------|---------|------------|
| `manifest.json` | JSON | Package identity, version, generator list, starter list | N/A |
| `knowledge/overview.md` | Markdown + frontmatter | 1-paragraph framework summary | ≤150 |
| `knowledge/concepts.md` | Markdown + frontmatter | Key terms glossary | ≤400 |
| `knowledge/patterns.md` | Markdown + frontmatter | Common code patterns | ≤600 |
| `knowledge/faq.md` | Markdown, one `---`-delimited entry per Q | Q&A pairs | ~120 per entry |
| `knowledge/decisions.md` | Markdown + frontmatter | Design rationale | ≤500 |
| `generators/manifest.json` | JSON | Generator input schemas + output file list | N/A |
| `generators/templates/**/*.forge` | Jinja2 + forge extensions | Code templates using 4-tier system | N/A |
| `generators/starters/*.json` | JSON | Full project scaffold recipes (ordered command steps) | N/A |
| `integrations/*.json` | JSON | Per-package integration patterns | N/A |

The CLI reads all of it. The agent never reads it directly — it calls the CLI.

---

## Part 10: Benchmark Methodology

To run the empirical benchmark after forge is built:

```bash
# In crates/forge/benches/render.rs
criterion_group!(benches,
  bench_atom_render,         # single atom: column definition
  bench_molecule_render,     # molecule: 5-field table body
  bench_feature_render,      # feature: full schema file (atoms → layout)
  bench_full_feature_render, # feature: all 8 files (add:feature)
  bench_batch_10_features,   # batch: 10 features parallel vs sequential
);
```

Run with:
```bash
cargo bench --bench render
```

Compare: MiniJinja baseline vs forge-on-Tera on the same templates.

---

*Version 6.0 · March 2026*
*Engine: forge (Tera-based) · Protocol: Framework-as-Package · CLI: tsx*
