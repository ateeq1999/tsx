---
id: patterns
token_estimate: 560
tags: [tanstack-start, patterns, code]
---

# TanStack Start — Common Patterns

## 1. CRUD Route pair (list + detail)

```
routes/products/index.tsx   → list page with table
routes/products/$id.tsx     → detail/edit page
```

Generate both with: `tsx add:feature products --fields name:string,price:number`

## 2. Server function with auth guard

```ts
export const getSecureData = createServerFn()
  .validator(z.object({}))
  .handler(async ({ data }) => {
    const session = await auth.api.getSession({ headers: getHeaders() })
    if (!session) throw new Error('Unauthorized')
    return db.select().from(table)
  })
```

## 3. Optimistic mutation

```ts
const mutation = useMutation({
  mutationFn: (data: NewProduct) => createProduct({ data }),
  onSuccess: () => queryClient.invalidateQueries({ queryKey: productsQueryKey() }),
})
```

## 4. Drizzle relation

```ts
export const productsRelations = relations(products, ({ one }) => ({
  category: one(categories, { fields: [products.categoryId], references: [categories.id] }),
}))
```

## 5. TanStack Form with Zod

```ts
const form = useForm({
  defaultValues: { name: '', price: 0 },
  validators: { onChange: productSchema },
  onSubmit: async ({ value }) => mutation.mutate(value),
})
```

## 6. Route loader with prefetch

```ts
export const Route = createFileRoute('/products')({
  loader: ({ context: { queryClient } }) =>
    queryClient.ensureQueryData({ queryKey: productsQueryKey(), queryFn: () => getProducts({ data: {} }) }),
  component: ProductsPage,
})
```

## 7. Barrel export (index.ts)

Each directory exports via `index.ts`:
```ts
export { ProductsTable } from './products-table'
export { ProductForm } from './product-form'
```

## 8. Auth session in route context

```ts
// routes/__root.tsx
export const Route = createRootRouteWithContext<{ session: Session | null }>()({
  component: RootLayout,
})
```
