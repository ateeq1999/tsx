# Drizzle ORM — SQLite

Drizzle supports SQLite via `better-sqlite3` (sync), `@libsql/client` (Turso), and Bun's built-in SQLite.

## Setup (libsql / Turso)

```ts
// db/index.ts
import { drizzle } from "drizzle-orm/libsql";
import { createClient } from "@libsql/client";

const client = createClient({ url: process.env.DATABASE_URL! });
export const db = drizzle(client);
```

## Setup (Bun)

```ts
import { drizzle } from "drizzle-orm/bun-sqlite";
import { Database } from "bun:sqlite";

const sqlite = new Database("data.db");
export const db = drizzle(sqlite);
```

## Schema definition

```ts
import { sqliteTable, integer, text } from "drizzle-orm/sqlite-core";

export const usersTable = sqliteTable("users", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  email: text("email").notNull().unique(),
  verified: integer("verified", { mode: "boolean" }).default(false),
  createdAt: integer("created_at", { mode: "timestamp" }).$defaultFn(() => new Date()),
});
```

## Queries

```ts
// list
const users = await db.select().from(usersTable);

// insert
await db.insert(usersTable).values({ email: "a@b.com" });

// update
await db.update(usersTable).set({ verified: true }).where(eq(usersTable.id, 1));

// delete
await db.delete(usersTable).where(eq(usersTable.id, 1));
```

## drizzle.config.ts

```ts
import { defineConfig } from "drizzle-kit";

export default defineConfig({
  schema: "./db/schema/*",
  out: "./drizzle",
  dialect: "sqlite",
  dbCredentials: { url: process.env.DATABASE_URL ?? "./data.db" },
});
```
