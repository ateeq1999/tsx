---
id: decisions
token_estimate: 420
tags: [tanstack-start, architecture, decisions, rationale]
---

# TanStack Start — Design Decisions

## Why server functions instead of API routes?

TanStack Start's `createServerFn` is fully type-safe end-to-end — the same TypeScript types flow from DB schema → server function → query hook → component with no manual type duplication. Traditional API routes require separate request/response type definitions.

## Why Drizzle ORM instead of Prisma?

Drizzle runs at the edge, generates no runtime code, and its table definitions *are* the types — `typeof table.$inferSelect` gives you the row type directly. Prisma's generated client adds bundle weight and an extra codegen step.

## Why Better Auth instead of NextAuth / Clerk?

Better Auth stores sessions in the database (same DB as your app data), supports custom session fields, and has no per-seat pricing. It's self-hosted by default. Clerk is excellent but adds an external service dependency and cost.

## Why SQLite as the default database?

SQLite + Turso/LiteFS covers most apps to significant scale with zero infrastructure setup. The Drizzle schema is identical for PostgreSQL — swap the driver and connection string when needed.

## Why file-based routing?

TanStack Router's file-based routing makes route discovery deterministic — an agent or developer can always find the file for a given URL. Dynamic params (`$id`) are type-safe.

## Why shadcn/ui?

Components are owned by your project (copied, not npm-installed), so they can be customised freely. The headless Radix UI primitives ensure accessibility without design lock-in.

## Why the 4-tier Atom/Molecule/Layout/Feature system?

Each tier has a single responsibility:
- Atoms change when a library API changes (e.g., new Drizzle column type)
- Molecules change when a pattern changes (e.g., new form hook API)
- Layouts change when project-wide import conventions change
- Features change when a new generator is added

Changing an atom propagates to all molecules that include it automatically — no copy-paste drift.
