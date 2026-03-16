# Axum + SeaORM — Conventions

- Entities in `src/entity/`, handlers in `src/handlers/`, services in `src/services/`
- Register migrations in `migration/src/lib.rs` MigratorTrait::migrations()
- Use `sea_orm_migration::cli::run_cli` entry point for `cargo run -- up/down`
- Inject `DatabaseConnection` via Axum state, never a global
