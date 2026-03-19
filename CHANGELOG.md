# Changelog

All notable changes to **tsx** are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added
- Package name validation on `POST /v1/packages/publish` — rejects uppercase, path traversal, names > 214 chars, invalid scoped formats
- `.env.example` for registry-server documenting all environment variables
- `render.yaml` for one-click Render.com deployment

### Fixed
- Replace all SQLx compile-time macros (`query!`, `query_as!`, `query_scalar!`) with runtime equivalents — fixes build failures on Render where `DATABASE_URL` is injected into the Docker build context
- Split multi-statement migration SQL into individual `execute()` calls — fixes "cannot insert multiple commands into a prepared statement" startup error
- Unified admin endpoint error messages — both "no key configured" and "wrong key" now return `Unauthorized` to prevent deployment info leak
- Added `#[allow(dead_code)]` on unused DB fields to silence compiler warnings

---

## [0.1.0] — 2025-xx-xx

### Added
- **CLI binary** (`crates/cli`) — 25+ commands across scaffold, generate, integrate, framework, registry, stack, plugin, and agent groups
- **Forge engine** (`crates/forge`) — 4-tier Tera template rendering (Atom → Molecule → Layout → Feature) with import hoisting and token-budget metadata
- **Registry server** (`crates/registry-server`) — Axum 0.8 HTTP server with 16 endpoints backed by PostgreSQL (Neon via SQLx)
- **Shared types** (`crates/shared`) — Serializable API types shared between CLI and server
- Multi-platform GitHub Release pipeline (Windows x64, Linux x64/ARM64, macOS Intel/ARM64)
- `cargo-chef` multi-stage Dockerfile for efficient layer caching
- CI pipeline: tests, Clippy, rustfmt, binary size gate

[Unreleased]: https://github.com/ateeq1999/tsx/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/ateeq1999/tsx/releases/tag/v0.1.0
