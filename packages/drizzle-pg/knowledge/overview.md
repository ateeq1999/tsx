# Drizzle ORM (PostgreSQL) — Overview

Drizzle ORM is a TypeScript-first ORM with a SQL-like query builder and schema-as-code.
This package generates schema definitions, migration scaffolds, and seed files for PostgreSQL.

## Key commands

| Command | What it generates |
|---|---|
| `add:schema` | `db/schema/{{name}}.ts` — pgTable definition with typed columns |
| `add:migration` | `db/migrations/{{name}}.sql` — migration file scaffold |
| `add:seed` | `db/seed/{{name}}.ts` — seed script that inserts test records |

## Slot integration

When `better-auth` is in the stack, `add:schema` automatically injects a `userId` foreign key.
