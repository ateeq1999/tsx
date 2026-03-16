# Drizzle ORM (PostgreSQL) — Conventions

- Schema files live in `db/schema/<name>.ts`
- Export both `select` type and `insert` type from every schema file
- Use `defaultNow()` for timestamps, never `new Date()` in schema
- Run `npx drizzle-kit push` for development, generate SQL migrations for production
