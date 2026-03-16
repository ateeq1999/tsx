# Axum + SeaORM — Overview

Axum is a Rust web framework built on Tokio/Tower. SeaORM provides async ORM for multiple databases.
This package generates entities, Axum handlers, SeaORM migrations, and service layers.

## Key commands

| Command | What it generates |
|---|---|
| `add:entity` | `src/entity/{{name}}.rs` — SeaORM DeriveEntityModel |
| `add:handler` | `src/handlers/{{name}}.rs` — Axum route handlers |
| `add:migration` | `migration/src/{{name}}.rs` — SeaORM migration scaffold |
| `add:service` | `src/services/{{name}}.rs` — service layer over SeaORM |

## State injection

Handlers receive `State<DatabaseConnection>` — inject via `axum::Router::with_state(db)`.
