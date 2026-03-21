---
id: overview
token_estimate: 140
tags: [tanstack-start, framework, fullstack]
---

# TanStack Start

TanStack Start is a full-stack React meta-framework built on TanStack Router. It provides file-based routing, type-safe server functions, and first-class TanStack Query integration.

## Key facts

- **Routing**: File-based via TanStack Router (`routes/` directory). Each file exports a `Route` created with `createFileRoute`.
- **Data fetching**: Server functions (`createServerFn`) + TanStack Query (`useSuspenseQuery`).
- **Forms**: TanStack Form with Zod validation schemas.
- **Auth**: Better Auth (sessions stored in DB, not JWTs by default).
- **Database**: Drizzle ORM with SQLite (default) or PostgreSQL.
- **UI**: shadcn/ui components + Tailwind CSS.

## Quick scaffold

```bash
tsx init                              # bootstrap project
tsx add:feature products --fields name:string,price:number
tsx add:migration                     # run drizzle-kit
```
