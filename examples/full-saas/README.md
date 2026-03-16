# full-saas

**Packages:** `@tsx-pkg/tanstack-start` + `@tsx-pkg/drizzle-pg` + `@tsx-pkg/better-auth` + `@tsx-pkg/shadcn`

Multi-tenant SaaS starter with organizations, memberships, and users. All auth-guarded.

## Stack Profile

```json
{
  "packages": ["tanstack-start", "drizzle-pg", "better-auth", "shadcn"]
}
```

## Generated Files

```
lib/auth.ts                         tsx run add:auth-setup
lib/auth-client.ts
routes/api/auth/$.ts
middleware/authGuard.ts             tsx run add:auth-guard

db/schema/organizations.ts         tsx run add:schema (organizations)
db/schema/memberships.ts           tsx run add:schema (memberships)
db/schema/users.ts                 tsx run add:schema (users)

server-functions/organizations.ts  tsx run add:server-fn x3
server-functions/memberships.ts
server-functions/users.ts

hooks/use-organizations.ts         tsx run add:query x3
hooks/use-memberships.ts
hooks/use-users.ts

components/OrganizationsForm.tsx   tsx run add:ui-form x2
components/UsersForm.tsx
components/OrganizationsTable.tsx  tsx run add:ui-data-table x2
components/UsersTable.tsx

routes/dashboard/index.tsx
routes/organizations/index.tsx
routes/users/index.tsx
routes/users/$id.tsx
```

## Reproduce

```bash
bash scripts/generate.sh
```
