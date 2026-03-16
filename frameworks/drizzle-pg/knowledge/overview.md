# Drizzle ORM Overview

Drizzle ORM is a lightweight, type-safe ORM for TypeScript. It provides a SQL-like syntax that compiles to SQL queries.

## Key Concepts

- **Schema**: Type-safe table definitions using `pgTable`, `mysqlTable`, or `sqliteTable`
- **Queries**: Chainable query builder (select, insert, update, delete)
- **Migrations**: Drizzle Kit for database migrations
- **Relations**: Define and fetch related tables

## With TanStack Start

Drizzle ORM integrates with TanStack Start through:
- Server-side db client (`db` from `~/db`)
- Type-safe queries in server functions
- Zod integration for input validation

## Database Schema

Tables are defined in `db/schema/` with columns:

```typescript
import { pgTable, text, boolean, timestamp } from 'drizzle-orm/pg-core'

export const todosTable = pgTable('todos', {
  id: serial('id').primaryKey(),
  title: text('title').notNull(),
  done: boolean('done').default(false),
  createdAt: timestamp('created_at').defaultNow()
})
```
