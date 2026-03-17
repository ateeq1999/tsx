/**
 * Drizzle schema for registry package tables.
 * These tables are owned by the registry-server (Rust/Axum) but mapped here
 * so TanStack Start server functions can query them directly via Drizzle ORM
 * (e.g. admin queries, "my packages" page, audit log).
 *
 * DATABASE: postgresql://postgres:@localhost:5432/tsx_db (same DB as better-auth)
 */

import { relations } from "drizzle-orm"
import {
  bigint,
  bigserial,
  boolean,
  index,
  jsonb,
  pgTable,
  text,
  timestamp,
} from "drizzle-orm/pg-core"
import { user } from "./auth"

// ── packages ──────────────────────────────────────────────────────────────────

export const packages = pgTable(
  "packages",
  {
    id: bigserial("id", { mode: "number" }).primaryKey(),
    name: text("name").notNull().unique(),        // @tsx-pkg/drizzle-pg
    slug: text("slug").notNull().unique(),         // drizzle-pg
    description: text("description").notNull().default(""),
    authorId: text("author_id").references(() => user.id, { onDelete: "set null" }),
    authorName: text("author_name").notNull().default(""),
    license: text("license").notNull().default("MIT"),
    tsxMin: text("tsx_min").notNull().default("0.1.0"),
    // PostgreSQL TEXT[] arrays — stored as text[] column
    tags: text("tags").array().notNull().default([]),
    lang: text("lang").array().notNull().default([]),
    runtime: text("runtime").array().notNull().default([]),
    provides: text("provides").array().notNull().default([]),
    integrates: text("integrates").array().notNull().default([]),
    readme: text("readme"),
    downloads: bigint("downloads", { mode: "number" }).notNull().default(0),
    publishedAt: timestamp("published_at", { withTimezone: true }).defaultNow().notNull(),
    updatedAt: timestamp("updated_at", { withTimezone: true }).defaultNow().notNull(),
  },
  (t) => [
    index("idx_packages_downloads").on(t.downloads),
    index("idx_packages_updated").on(t.updatedAt),
    index("idx_packages_author").on(t.authorId),
  ],
)

// ── versions ──────────────────────────────────────────────────────────────────

export const versions = pgTable(
  "versions",
  {
    id: bigserial("id", { mode: "number" }).primaryKey(),
    packageId: bigint("package_id", { mode: "number" })
      .notNull()
      .references(() => packages.id, { onDelete: "cascade" }),
    version: text("version").notNull(),
    manifest: jsonb("manifest").notNull().default({}),
    checksum: text("checksum").notNull().default(""),
    sizeBytes: bigint("size_bytes", { mode: "number" }).notNull().default(0),
    tarballPath: text("tarball_path").notNull().default(""),
    downloadCount: bigint("download_count", { mode: "number" }).notNull().default(0),
    yanked: boolean("yanked").notNull().default(false),
    publishedAt: timestamp("published_at", { withTimezone: true }).defaultNow().notNull(),
  },
  (t) => [index("idx_versions_package").on(t.packageId)],
)

// ── download_logs ─────────────────────────────────────────────────────────────

export const downloadLogs = pgTable(
  "download_logs",
  {
    id: bigserial("id", { mode: "number" }).primaryKey(),
    packageId: bigint("package_id", { mode: "number" })
      .notNull()
      .references(() => packages.id, { onDelete: "cascade" }),
    versionId: bigint("version_id", { mode: "number" }).references(
      () => versions.id,
      { onDelete: "set null" },
    ),
    ipAddress: text("ip_address"),
    userAgent: text("user_agent"),
    downloadedAt: timestamp("downloaded_at", { withTimezone: true }).defaultNow().notNull(),
  },
  (t) => [
    index("idx_download_logs_package").on(t.packageId),
    index("idx_download_logs_time").on(t.downloadedAt),
  ],
)

// ── audit_log ─────────────────────────────────────────────────────────────────

export const auditLog = pgTable(
  "audit_log",
  {
    id: bigserial("id", { mode: "number" }).primaryKey(),
    /** 'publish' | 'yank' | 'delete' | 'update_readme' | 'update_meta' */
    action: text("action").notNull(),
    packageName: text("package_name").notNull(),
    version: text("version"),
    userId: text("user_id").references(() => user.id, { onDelete: "set null" }),
    authorName: text("author_name"),
    ipAddress: text("ip_address"),
    detail: jsonb("detail"),
    createdAt: timestamp("created_at", { withTimezone: true }).defaultNow().notNull(),
  },
  (t) => [index("idx_audit_log_time").on(t.createdAt)],
)

// ── Relations ─────────────────────────────────────────────────────────────────

export const packagesRelations = relations(packages, ({ one, many }) => ({
  author: one(user, { fields: [packages.authorId], references: [user.id] }),
  versions: many(versions),
  downloadLogs: many(downloadLogs),
}))

export const versionsRelations = relations(versions, ({ one, many }) => ({
  package: one(packages, { fields: [versions.packageId], references: [packages.id] }),
  downloadLogs: many(downloadLogs),
}))

export const downloadLogsRelations = relations(downloadLogs, ({ one }) => ({
  package: one(packages, { fields: [downloadLogs.packageId], references: [packages.id] }),
  version: one(versions, { fields: [downloadLogs.versionId], references: [versions.id] }),
}))

export const auditLogRelations = relations(auditLog, ({ one }) => ({
  user: one(user, { fields: [auditLog.userId], references: [user.id] }),
}))
