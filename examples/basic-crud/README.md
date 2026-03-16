# basic-crud

**Packages:** `@tsx-pkg/tanstack-start` + `@tsx-pkg/drizzle-pg`

A minimal Products CRUD — schema, server functions, React Query hooks, form, and table — all generated in one pass.

## Stack Profile

```json
{
  "lang": "typescript",
  "runtime": "node",
  "packages": ["tanstack-start", "drizzle-pg"]
}
```

## Generated Files

```
db/schema/products.ts          tsx run add:schema
server-functions/products.ts   tsx run add:server-fn
hooks/use-products.ts          tsx run add:query
components/ProductsForm.tsx    tsx run add:form
components/ProductsTable.tsx   tsx run add:table
routes/products/index.tsx      tsx run add:page
```

## Reproduce

```bash
bash scripts/generate.sh
```
