# Drizzle ORM Conventions

## File Structure

```
src/
├── db/
│   ├── index.ts         # Database client export
│   ├── schema.ts       # Table definitions
│   └── migrations/      # Migration files
└── server-functions/   # Use db here
```

## Naming Conventions

- Table name: `plural` (e.g., `usersTable`, `todosTable`)
- Column names: `snake_case` in SQL, `camelCase` in TypeScript
- Schema file: `db/schema/{{name}}.ts` per entity
- Index file: `db/index.ts` exports db client

## Column Types (PostgreSQL)

- `text` - String/varchar
- `boolean` - Boolean
- `integer` / `serial` - Integer
- `timestamp` / `timestampz` - Date/time
- `uuid` - UUID
- `jsonb` - JSON

## Query Patterns

### Select
```typescript
const todos = await db.select().from(todosTable)
```

### Insert
```typescript
const [todo] = await db.insert(todosTable).values({
  title: "New todo"
}).returning()
```

### Update
```typescript
const [todo] = await db.update(todosTable)
  .set({ done: true })
  .where(eq(todosTable.id, id))
  .returning()
```

### Delete
```typescript
await db.delete(todosTable).where(eq(todosTable.id, id))
```

## With Better Auth

When using Drizzle with Better Auth, add userId foreign key:

```typescript
import { usersTable } from './user'

export const todosTable = pgTable('todos', {
  // ... columns
  userId: text('user_id').references(() => usersTable.id)
})
```
