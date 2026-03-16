---
id: faq
token_estimate: 30
tags: [tanstack-start, faq, how-to]
---

# TanStack Start — FAQ

Each entry below is a self-contained Q&A. token_estimate per entry: ~80-130 tokens.

---
id: faq-add-route
question: How do I add a new route?
token_estimate: 90
tags: [routing, route]
related: [faq-add-feature]
---

Create a file in `routes/`. For a list page: `routes/products/index.tsx`. For a detail page: `routes/products/$id.tsx`. Or generate both at once:

```bash
tsx add:feature products --fields name:string,price:number
```

The route file must export `export const Route = createFileRoute('/products')({...})`.

---
id: faq-add-auth
question: How do I add authentication?
token_estimate: 110
tags: [auth, better-auth, security]
requires: [better-auth]
related: [faq-add-auth-guard, faq-add-migration]
---

```bash
tsx add:auth --provider email
tsx add:migration
```

This creates `lib/auth.ts` with Better Auth configured. To protect a route:

```bash
tsx add:auth-guard --route routes/dashboard/index.tsx
```

---
id: faq-add-schema
question: How do I add a new database table?
token_estimate: 100
tags: [drizzle, database, schema]
requires: [drizzle-orm]
related: [faq-add-migration]
---

```bash
tsx add:schema products --fields name:string,price:number --timestamps
tsx add:migration
```

Creates `db/schema/products.ts` with Drizzle table definition and TypeScript types. Always run `add:migration` after adding a schema.

---
id: faq-add-migration
question: How do I run database migrations?
token_estimate: 70
tags: [drizzle, migration, database]
---

```bash
tsx add:migration
```

Runs `npx drizzle-kit generate` then `npx drizzle-kit migrate`. Requires a valid `drizzle.config.ts` in the project root.

---
id: faq-add-query
question: How do I add a TanStack Query hook?
token_estimate: 100
tags: [react-query, query, data-fetching]
requires: [@tanstack/react-query]
---

```bash
tsx add:query products --operations list,get,create,update,delete
```

Creates `queries/products.ts` with `useSuspenseQuery` hooks and `useMutation` hooks. Wraps the corresponding server functions.

---
id: faq-add-form
question: How do I add a form?
token_estimate: 95
tags: [form, tanstack-form, validation]
requires: [@tanstack/react-form]
---

```bash
tsx add:form product --fields name:string,price:number
```

Creates `components/product/product-form.tsx` with TanStack Form + Zod validation. Each field uses the appropriate atom (input, select, switch, datepicker).

---
id: faq-add-table
question: How do I add a data table?
token_estimate: 85
tags: [table, tanstack-table]
requires: [@tanstack/react-table]
---

```bash
tsx add:table products --fields name:string,price:number
```

Creates `components/products/products-table.tsx` with TanStack Table columns, sorting, and pagination.

---
id: faq-add-server-fn
question: How do I add a server function?
token_estimate: 100
tags: [server-function, rpc, api]
---

```bash
tsx add:server-fn products --operations list,create,update,delete
```

Creates `server-functions/products.ts` with typed `createServerFn` handlers, Zod validation, and optional auth guard checks.

---
id: faq-file-structure
question: Where do files go?
token_estimate: 120
tags: [structure, conventions, files]
---

| File type | Location |
|-----------|----------|
| Routes | `routes/` |
| Server functions | `server-functions/` |
| Query hooks | `queries/` |
| DB schemas | `db/schema/` |
| Components | `components/<feature>/` |
| Auth config | `lib/auth.ts` |
| Drizzle config | `drizzle.config.ts` |

Use `tsx where --thing <type>` to query file locations for any kind.

---
id: faq-add-feature
question: How do I scaffold a complete CRUD feature?
token_estimate: 110
tags: [feature, crud, scaffold]
---

```bash
tsx add:feature products --fields name:string,price:number,qty:integer --auth
```

Generates 8 files: schema, server-fn, query hooks, list page, detail page, form component, table component, delete dialog. Pass `--auth` to add session guards to all server functions.

---
id: faq-env
question: What environment variables are required?
token_estimate: 80
tags: [env, configuration, setup]
---

Copy `.env.example` to `.env`:

```
DATABASE_URL=file:local.db
BETTER_AUTH_SECRET=change-me-in-production
BETTER_AUTH_URL=http://localhost:3000
```

---
id: faq-shadcn
question: How do I add shadcn/ui components?
token_estimate: 70
tags: [shadcn, ui, components]
requires: [shadcn-ui]
---

```bash
npx shadcn@latest add button card input label select
```

Components are copied into `components/ui/`. TSX generators automatically import from this directory.

---
id: faq-pagination
question: How do I add pagination to a feature?
token_estimate: 85
tags: [pagination, table, query]
---

```bash
tsx add:feature products --fields name:string --paginated
```

Adds `page` and `pageSize` params to the server function, `totalCount` to the response, and pagination controls to the table component.

---
id: faq-batch
question: How do I run multiple TSX commands at once?
token_estimate: 80
tags: [batch, agent, cli]
---

```bash
tsx batch --json '[
  {"command": "add:schema", "args": {"name": "orders", "fields": [...]}},
  {"command": "add:server-fn", "args": {"name": "orders"}},
  {"command": "add:migration", "args": {}}
]'
```

---
id: faq-dry-run
question: How do I preview what files will be created without writing them?
token_estimate: 60
tags: [dry-run, preview, agent]
---

Add `--dry-run` to any command:

```bash
tsx add:feature products --fields name:string --dry-run
```

Returns `files_created` in the JSON response without touching disk.
