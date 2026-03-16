# Drizzle SQLite — Conventions

## Column types mapping

SQLite has only 5 storage classes: NULL, INTEGER, REAL, TEXT, BLOB.
Drizzle maps TypeScript types as follows:

| TS type | Drizzle column |
|---------|---------------|
| `string` | `text` |
| `number` (integer) | `integer` |
| `number` (float) | `real` |
| `boolean` | `integer({ mode: "boolean" })` |
| `Date` | `integer({ mode: "timestamp" })` |
| `uuid` | `text` with `.$defaultFn(() => crypto.randomUUID())` |
| `object/array` | `text({ mode: "json" })` |

## Naming
- Table names: `snake_case` plural (e.g. `users`, `blog_posts`)
- Column names: `snake_case` (e.g. `created_at`, `user_id`)
- TypeScript exports: `PascalCase` (e.g. `User`, `NewUser`)

## Foreign keys
```ts
userId: integer("user_id").notNull().references(() => usersTable.id),
```

## Auto-timestamps
Use `.$defaultFn(() => new Date())` instead of SQL defaults for portability across SQLite drivers.

## Soft delete pattern
```ts
deletedAt: integer("deleted_at", { mode: "timestamp" }),
// query:
.where(isNull(table.deletedAt))
```

## Turso / libsql note
Use `url: "libsql://..."` for remote Turso databases and `url: "file:./data.db"` for local dev.
