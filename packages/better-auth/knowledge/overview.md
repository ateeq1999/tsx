# Better Auth — Overview

Better Auth is a TypeScript auth library with first-class Drizzle and framework adapters.
This package generates the auth config, client, schema tables, session hooks, and route guards.

## Key commands

| Command | What it generates |
|---|---|
| `add:auth-setup` | `lib/auth.ts`, `lib/auth-client.ts`, `db/schema/auth.ts` |
| `add:auth-guard` | `middleware/{{name}}-guard.ts` — redirect guard middleware |
| `add:session` | `hooks/use-session.ts` — typed useSession hook |

## Slot integration

When `drizzle-pg` is in the stack, `add:schema` calls from drizzle-pg automatically
receive a `userId` foreign key column via the `auth_fields` slot.
