# with-shadcn

**Packages:** `@tsx-pkg/tanstack-start` + `@tsx-pkg/drizzle-pg` + `@tsx-pkg/better-auth` + `@tsx-pkg/shadcn`

Adds shadcn/ui form primitives and a full `DataTable` (sorting, pagination, column visibility) on top of `with-auth`.

## Stack Profile

```json
{
  "packages": ["tanstack-start", "drizzle-pg", "better-auth", "shadcn"]
}
```

## Key Differences from `with-auth`

- `components/ui/` — shadcn primitive components (button, input, label, data-table)
- `PostsForm.tsx` — uses shadcn `Input`, `Label`, `Button` imports
- `PostsTable.tsx` — wraps the shadcn `DataTable` with full column visibility toggle

## Reproduce

```bash
bash scripts/generate.sh
```
