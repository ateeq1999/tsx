/**
 * Re-export all API types from the shared @tsx/api-types workspace package.
 *
 * Existing imports of `@/lib/types` continue to work unchanged.
 * The canonical source of truth is packages/api-types/src/index.ts.
 */
export type {
  Package,
  PackageVersion,
  SearchResult,
  RegistryStats,
  DailyDownloads,
  AuditEntry,
  RateLimitEntry,
  ApiError,
  PublishResult,
} from "@tsx/api-types"
