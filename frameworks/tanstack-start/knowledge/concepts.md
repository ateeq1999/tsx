---
id: concepts
token_estimate: 380
tags: [tanstack-start, concepts, architecture]
---

# TanStack Start — Core Concepts

## Route

A file in `routes/` that exports a `Route` object. Created with `createFileRoute('/<path>')`. Each route can have a `loader` (server-side data fetch) and a component.

```ts
export const Route = createFileRoute('/products')({
  loader: () => fetchProducts(),
  component: ProductsPage,
})
```

## Server Function

Type-safe RPC callable from client code. Created with `createServerFn()`. Validated with Zod. Lives in `server-functions/`.

```ts
export const getProducts = createServerFn()
  .validator(z.object({ page: z.number() }))
  .handler(async ({ data }) => db.select().from(products))
```

## TanStack Query Hook

Wraps a server function call with caching and suspense. Lives in `queries/`.

```ts
export const useProducts = () =>
  useSuspenseQuery({ queryKey: productsQueryKey(), queryFn: () => getProducts({ data: {} }) })
```

## Drizzle Schema

Type-safe table definition. Lives in `db/schema/`. Exported types are used directly in server functions.

```ts
export const products = sqliteTable('products', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  name: text('name').notNull(),
})
export type Product = typeof products.$inferSelect
export type NewProduct = typeof products.$inferInsert
```

## Atom / Molecule / Layout / Feature (TSX tiers)

- **Atom**: indivisible template fragment (a Drizzle column, a Zod field rule)
- **Molecule**: atoms composed into a logical block (full Drizzle table, Zod schema object)
- **Layout**: file-level shell that hoists imports to the top
- **Feature**: orchestrates all molecules for one complete CRUD module (8+ files)

## Auth Guard

A `beforeLoad` hook on a route that redirects unauthenticated users:

```ts
beforeLoad: async ({ context }) => {
  if (!context.session) throw redirect({ to: '/login' })
}
```
