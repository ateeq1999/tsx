# Drizzle ORM — MySQL

Drizzle is a headless TypeScript ORM with first-class MySQL support via `mysql2` and `@planetscale/database`.

## Setup

```ts
// db/index.ts
import { drizzle } from "drizzle-orm/mysql2";
import mysql from "mysql2/promise";

const pool = mysql.createPool({ uri: process.env.DATABASE_URL! });
export const db = drizzle(pool);
```

## Schema definition

```ts
import { mysqlTable, serial, varchar, boolean, datetime } from "drizzle-orm/mysql-core";

export const usersTable = mysqlTable("users", {
  id: serial("id").primaryKey().autoincrement(),
  email: varchar("email", { length: 255 }).notNull().unique(),
  verified: boolean("verified").default(false),
  createdAt: datetime("created_at").$defaultFn(() => new Date()),
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
  dialect: "mysql",
  dbCredentials: { url: process.env.DATABASE_URL! },
});
```
