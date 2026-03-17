import { z } from "zod"
import {
  DailyDownloadsSchema,
  PackageSchema,
  PackageVersionSchema,
  RegistryStatsSchema,
  SearchResultSchema,
} from "./schemas"
import type { Package, PackageVersion, RegistryStats, SearchResult, DailyDownloads } from "./types"

const BASE_URL = import.meta.env.VITE_REGISTRY_URL ?? "http://localhost:8080"

async function fetchJson<T>(path: string, schema: z.ZodType<T>): Promise<T> {
  const res = await fetch(`${BASE_URL}${path}`)
  if (!res.ok) throw new Error(`Registry API error ${res.status}: ${await res.text()}`)
  return schema.parse(await res.json())
}

export const registryApi = {
  search: (q: string, page = 1, size = 20) =>
    fetchJson<SearchResult>(
      `/v1/search?q=${encodeURIComponent(q)}&page=${page}&size=${size}`,
      SearchResultSchema,
    ),

  getPackage: (name: string) =>
    fetchJson<Package>(`/v1/packages/${encodeURIComponent(name)}`, PackageSchema),

  getVersions: (name: string) =>
    fetchJson<Array<PackageVersion>>(
      `/v1/packages/${encodeURIComponent(name)}/versions`,
      z.array(PackageVersionSchema),
    ),

  getStats: () =>
    fetchJson<RegistryStats>("/v1/stats", RegistryStatsSchema),

  getRecent: (limit = 12) =>
    fetchJson<Array<Package>>(`/v1/packages?sort=recent&limit=${limit}`, z.array(PackageSchema)),

  getReadme: async (name: string): Promise<string | null> => {
    const res = await fetch(`${BASE_URL}/v1/packages/${encodeURIComponent(name)}/readme`)
    if (!res.ok) return null
    return res.text()
  },

  getDownloadStats: (name: string) =>
    fetchJson<Array<DailyDownloads>>(
      `/v1/packages/${encodeURIComponent(name)}/stats/downloads`,
      z.array(DailyDownloadsSchema),
    ),
}
