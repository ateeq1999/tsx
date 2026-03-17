import { createServerFn } from "@tanstack/react-start"
import { desc } from "drizzle-orm"
import { db } from "@/db"
import { user } from "@/db/schema/auth"
import { requireRole } from "@/middleware/role-guard"
import type { AuditEntry, RateLimitEntry } from "@tsx/api-types"

const REGISTRY_URL = process.env.VITE_REGISTRY_URL ?? "http://localhost:8080"

// ── Users (better-auth table — stays in Drizzle) ──────────────────────────────

export const getAdminUsers = createServerFn({ method: "GET" }).handler(async () => {
  await requireRole("admin")
  return db
    .select({
      id: user.id,
      name: user.name,
      email: user.email,
      emailVerified: user.emailVerified,
      createdAt: user.createdAt,
    })
    .from(user)
    .orderBy(desc(user.createdAt))
})

// ── Audit log (Rust registry-server) ─────────────────────────────────────────

export const getAdminAuditLog = createServerFn({ method: "GET" }).handler(async () => {
  await requireRole("admin")
  const apiKey = process.env.TSX_REGISTRY_API_KEY
  const res = await fetch(`${REGISTRY_URL}/v1/admin/audit-log?limit=500`, {
    headers: apiKey ? { Authorization: `Bearer ${apiKey}` } : {},
  })
  if (!res.ok) throw new Error(`Registry API error ${res.status}: ${await res.text()}`)
  return (await res.json()) as AuditEntry[]
})

// ── Rate limits (Rust registry-server) ───────────────────────────────────────

export const getAdminRateLimits = createServerFn({ method: "GET" }).handler(async () => {
  await requireRole("admin")
  const apiKey = process.env.TSX_REGISTRY_API_KEY
  const res = await fetch(`${REGISTRY_URL}/v1/admin/rate-limits`, {
    headers: apiKey ? { Authorization: `Bearer ${apiKey}` } : {},
  })
  if (!res.ok) throw new Error(`Registry API error ${res.status}: ${await res.text()}`)
  return (await res.json()) as RateLimitEntry[]
})
