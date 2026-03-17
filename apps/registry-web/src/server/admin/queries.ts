import { createServerFn } from "@tanstack/react-start"
import { desc, count, sum, sql } from "drizzle-orm"
import { db } from "@/db"
import { user } from "@/db/schema/auth"
import { packages, versions, auditLog } from "@/db/schema/packages"
import { requireRole } from "@/middleware/role-guard"

// ── Users ─────────────────────────────────────────────────────────────────────

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

// ── Packages ──────────────────────────────────────────────────────────────────

export const getAdminPackages = createServerFn({ method: "GET" }).handler(async () => {
  await requireRole("admin")
  return db
    .select({
      id: packages.id,
      name: packages.name,
      description: packages.description,
      authorName: packages.authorName,
      downloads: packages.downloads,
      publishedAt: packages.publishedAt,
      updatedAt: packages.updatedAt,
    })
    .from(packages)
    .orderBy(desc(packages.updatedAt))
    .limit(200)
})

export const getAdminPackageVersions = createServerFn({ method: "GET" })
  .inputValidator((data: { packageId: number }) => data)
  .handler(async ({ data }) => {
    await requireRole("admin")
    return db
      .select({
        id: versions.id,
        version: versions.version,
        downloadCount: versions.downloadCount,
        yanked: versions.yanked,
        publishedAt: versions.publishedAt,
      })
      .from(versions)
      .where(sql`${versions.packageId} = ${data.packageId}`)
      .orderBy(desc(versions.publishedAt))
  })

// ── Audit log ─────────────────────────────────────────────────────────────────

export const getAdminAuditLog = createServerFn({ method: "GET" }).handler(async () => {
  await requireRole("admin")
  return db
    .select({
      id: auditLog.id,
      action: auditLog.action,
      packageName: auditLog.packageName,
      version: auditLog.version,
      authorName: auditLog.authorName,
      ipAddress: auditLog.ipAddress,
      createdAt: auditLog.createdAt,
    })
    .from(auditLog)
    .orderBy(desc(auditLog.createdAt))
    .limit(500)
})

// ── Registry stats (direct DB query as fallback) ──────────────────────────────

export const getAdminStats = createServerFn({ method: "GET" }).handler(async () => {
  await requireRole("admin")
  const [pkgStats] = await db
    .select({
      totalPackages: count(packages.id),
      totalDownloads: sum(packages.downloads),
    })
    .from(packages)

  const [verStats] = await db
    .select({ totalVersions: count(versions.id) })
    .from(versions)

  const thisWeek = await db
    .select({ count: count(packages.id) })
    .from(packages)
    .where(sql`${packages.publishedAt} >= NOW() - INTERVAL '7 days'`)

  return {
    totalPackages: pkgStats?.totalPackages ?? 0,
    totalDownloads: Number(pkgStats?.totalDownloads ?? 0),
    totalVersions: verStats?.totalVersions ?? 0,
    packagesThisWeek: thisWeek[0]?.count ?? 0,
  }
})
