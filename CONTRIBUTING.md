# Contributing to tsx

Thank you for your interest in contributing! This document covers how to set up the project, the code structure, and the PR process.

---

## Prerequisites

| Tool | Minimum version | Install |
| --- | --- | --- |
| Rust | 1.76 | `rustup update stable` |
| Cargo | ships with Rust | — |
| PostgreSQL | 14 (for registry-server) | or use Neon free tier |
| Node.js / Bun | 20+ (for npm package tooling) | optional |

---

## Development Setup

```bash
git clone https://github.com/ateeq1999/tsx.git
cd tsx

# Build all crates
cargo build

# Build only the CLI
cargo build -p tsx

# Build only the registry server
cargo build -p tsx-registry
```

### Registry Server

Copy the example env file and fill in your database URL:

```bash
cp crates/registry-server/.env.example crates/registry-server/.env
# Edit .env — add your DATABASE_URL
```

Run the server locally:

```bash
cargo run -p tsx-registry
# Server starts on http://localhost:8282
```

### Running Tests

```bash
# All tests
cargo test

# E2E CLI tests only
cargo test -p tsx --test e2e

# Forge rendering benchmarks
cargo bench -p tsx-forge
```

### Lint and Format

```bash
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

---

## Project Structure

```
tsx/
├── crates/
│   ├── cli/             CLI binary — 25+ commands via Clap 4
│   ├── forge/           Template rendering engine (Tera-based)
│   ├── registry-server/ Axum HTTP server with 16 REST endpoints
│   └── shared/          Serializable API types (used by cli + server)
├── templates/           Tera/Jinja2 templates for code generation
├── frameworks/          Framework package definitions
├── migrations/          PostgreSQL migration SQL files
└── .github/workflows/   CI and multi-platform release pipeline
```

---

## Making Changes

1. **Fork** the repository and create a branch: `git checkout -b feat/my-feature`
2. Make your changes with tests where applicable
3. Run `cargo clippy` and `cargo fmt` before committing
4. Open a pull request against `main` with a clear description of what changed and why

### Commit style

Use conventional commits:

```
feat(cli): add `tsx upgrade` self-update command
fix(registry): validate package name before upsert
docs: update CHANGELOG for 0.2.0
chore: bump sqlx to 0.8.5
```

---

## Adding a New CLI Command

1. Create `crates/cli/src/commands/<group>/<command>.rs`
2. Add a JSON schema in `crates/cli/src/schemas/<command>.rs` if the command accepts structured input
3. Register the subcommand in `crates/cli/src/main.rs` under the appropriate `Commands` variant
4. Add an E2E test in `crates/cli/tests/e2e.rs`

---

## Registry Server Endpoints

All endpoints live in `crates/registry-server/src/routes/`. Each file maps to a logical group:

| File | Endpoints |
| --- | --- |
| `health.rs` | `GET /health` |
| `stats.rs` | `GET /v1/stats` |
| `search.rs` | `GET /v1/search` |
| `packages.rs` | all `/v1/packages/*` CRUD |
| `admin.rs` | `/v1/admin/audit-log`, `/v1/admin/rate-limits` |

Database queries live in `crates/registry-server/src/db/`. All queries use runtime SQLx (no compile-time macros) so the server builds without a live database connection.

---

## Reporting Bugs

Open an issue at [github.com/ateeq1999/tsx/issues](https://github.com/ateeq1999/tsx/issues) with:

- tsx version (`tsx --version`)
- Operating system and architecture
- Steps to reproduce
- Expected vs actual behaviour
- Any relevant log output (`RUST_LOG=debug tsx ...`)
