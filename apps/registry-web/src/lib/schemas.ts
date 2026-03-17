import { z } from "zod"

export const PackageSchema = z.object({
  name: z.string(),
  version: z.string(),
  description: z.string(),
  author: z.string(),
  license: z.string(),
  tags: z.array(z.string()),
  tsx_min: z.string(),
  created_at: z.string(),
  updated_at: z.string(),
  download_count: z.number(),
  lang: z.string().optional(),
  runtime: z.string().optional(),
  provides: z.array(z.string()).optional(),
  integrates_with: z.array(z.string()).optional(),
})

export const PackageVersionSchema = z.object({
  version: z.string(),
  published_at: z.string(),
  download_count: z.number(),
})

export const SearchResultSchema = z.object({
  packages: z.array(PackageSchema),
  total: z.number(),
  page: z.number(),
  per_page: z.number(),
})

export const RegistryStatsSchema = z.object({
  total_packages: z.number(),
  total_downloads: z.number(),
  total_versions: z.number(),
  packages_this_week: z.number(),
})

export const DailyDownloadsSchema = z.object({
  date: z.string(),
  downloads: z.number(),
})
