# Better Auth Conventions

## File Structure

```
src/
├── lib/
│   └── auth.ts          # Auth configuration
├── components/
│   └── auth/            # Auth components (SignIn, SignOut)
└── routes/
    └── auth/            # Auth routes (/sign-in, /sign-up)
```

## Naming Conventions

- Auth config file: `auth.ts` in `src/lib/`
- Session hook: `useSession` (from `better-auth/react`)
- Auth functions: `signIn()`, `signOut()`, `requireAuth()`

## Server Functions

Use `requireAuth(context)` to protect server functions:

```typescript
import { requireAuth } from "~/lib/auth"

export const getProtectedData = createServerFn({ method: "GET" })
  .handler(async ({ context }) => {
    await requireAuth(context)
    return { secret: "data" }
  })
```

## Route Protection

Wrap routes with auth guard in route definition:

```typescript
defineRoute({
  beforeLoad: async ({ context }) => {
    await requireAuth(context)
  }
})
```
