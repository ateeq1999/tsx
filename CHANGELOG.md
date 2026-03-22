# Changelog

All notable changes to **tsx** are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [0.2.0] — 2026-03-23

### Added

- **Registry-driven architecture** — templates and generator specs no longer baked into the binary; loaded at runtime from installed packages in `~/.tsx/packages/`, `.tsx/packages/`, or `packages/` in the source tree
- **`PackageStore`** — multi-root package discovery (`project-local → global cache → bundled`); scans all installed packages on startup
- **`PackageInstaller`** — download, extract, and install `.tgz` tarballs from the registry into `~/.tsx/packages/`
- **`tsx package new/validate/pack/publish/install`** — full author workflow for creating and publishing registry packages
- **`GET /v1/discovery`** — registry endpoint: maps npm package names → tsx packages using `manifest.jsonb` column
- **`GET /v1/commands`** — registry endpoint: lists all commands across all packages, filterable by package id or command id
- **`tsx stack init`** — now stores canonical npm package names (e.g. `@tanstack/start`) instead of tsx slugs; `style` and `paths` always written with defaults
- **Migration 0006** — `versions.manifest JSONB` column + GIN index for discovery queries
- **`scripts/install.sh`** — platform-aware one-line installer (Linux x64/ARM64, macOS Intel/ARM64)
- **`render.yaml`** — persistent disk mount at `/data`, `SQLX_OFFLINE=true`, starter plan

### Changed

- `frameworks/` directory renamed to `packages/` — aligns with registry-driven naming
- `get_frameworks_dir()` now prefers `packages/` with `frameworks/` as legacy fallback
- `FrameworkLoader` and `PackageStore` resolve `packages/` before `frameworks/`
- TUI browser reads from `PackageStore::list()` instead of hardcoded items
- `Generate` and `Add` clap subcommands are thin wrappers that delegate to `run::run()`

### Removed

- Out-of-scope backend frameworks: `axum-sea-orm`, `fastapi-sqlalchemy`, `gin-gorm`
- Root-level `templates/` directory — templates now live in `packages/<id>/templates/`
- All `include_str!()` embedded templates — `embedded.rs` returns an empty map

### Fixed

- `tsx stack init` storing tsx slugs instead of npm package names for detected deps
- `PathConfig` / `StyleConfig` fields defaulting to empty instead of canonical values
- Scoped npm package `base_name()` returning empty string for `@scope/pkg@version`

---

## [0.1.0] — 2025-12-01

### Added

- **CLI binary** (`crates/cli`) — 25+ commands across scaffold, generate, integrate, framework, registry, stack, plugin, and agent groups
- **Forge engine** (`crates/forge`) — 4-tier Tera template rendering (Atom → Molecule → Layout → Feature) with import hoisting and token-budget metadata
- **Registry server** (`crates/registry-server`) — Axum 0.8 HTTP server with 16 endpoints backed by PostgreSQL (Neon via SQLx)
- **Shared types** (`crates/shared`) — Serializable API types shared between CLI and server
- Multi-platform GitHub Release pipeline (Windows x64, Linux x64/ARM64, macOS Intel/ARM64)
- `cargo-chef` multi-stage Dockerfile for efficient layer caching
- CI pipeline: tests, Clippy, rustfmt, binary size gate

[Unreleased]: https://github.com/ateeq1999/tsx/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/ateeq1999/tsx/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ateeq1999/tsx/releases/tag/v0.1.0
