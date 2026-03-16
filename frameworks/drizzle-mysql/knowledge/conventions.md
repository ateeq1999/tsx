# Drizzle MySQL — Conventions

## Column types mapping

| TS type | Drizzle column |
|---------|---------------|
| `string` (short) | `varchar(255)` |
| `string` (long) | `text` |
| `number` | `int` |
| `boolean` | `boolean` |
| `Date` | `datetime` |
| `uuid` | `varchar(36)` with `DEFAULT (UUID())` |
| `object` | `json` |

## Naming
- Table names: `snake_case` plural (e.g. `users`, `blog_posts`)
- Column names: `snake_case` (e.g. `created_at`, `user_id`)
- TypeScript exports: `PascalCase` (e.g. `User`, `NewUser`)

## Foreign keys
```ts
userId: int("user_id").notNull().references(() => usersTable.id, { onDelete: "cascade" }),
```

## Auto-timestamps
Use `.$defaultFn(() => new Date())` rather than `DEFAULT CURRENT_TIMESTAMP` for portability.

## Soft delete pattern
```ts
deletedAt: datetime("deleted_at"),
// query:
.where(isNull(table.deletedAt))
```

## Indexes
```ts
import { index } from "drizzle-orm/mysql-core";

export const postsTable = mysqlTable("posts", { ... }, (t) => ({
  emailIdx: index("email_idx").on(t.email),
}));
```
