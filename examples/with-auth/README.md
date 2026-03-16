# with-auth

**Packages:** `@tsx-pkg/tanstack-start` + `@tsx-pkg/drizzle-pg` + `@tsx-pkg/better-auth`

Adds email + GitHub OAuth authentication on top of `basic-crud`. Protected routes use an auth middleware guard.

## Stack Profile

```json
{
  "packages": ["tanstack-start", "drizzle-pg", "better-auth"]
}
```

## Generated Files

```
lib/auth.ts                    tsx run add:auth-setup
lib/auth-client.ts             tsx run add:auth-setup (file 2)
routes/api/auth/$.ts           tsx run add:auth-setup (file 3)
middleware/authGuard.ts        tsx run add:auth-guard
db/schema/posts.ts             tsx run add:schema
server-functions/posts.ts      tsx run add:server-fn
hooks/use-posts.ts             tsx run add:query
components/PostsTable.tsx      tsx run add:table
routes/dashboard/index.tsx     (auth-guarded page)
```

## Reproduce

```bash
bash scripts/generate.sh
```
