import type { Package, PackageVersion, RegistryStats, SearchResult } from "./types"

const BASE_URL = import.meta.env.VITE_REGISTRY_URL ?? "http://localhost:8080"

async function fetchJson<T>(path: string): Promise<T> {
  const res = await fetch(`${BASE_URL}${path}`)
  if (!res.ok) throw new Error(`Registry API error ${res.status}: ${await res.text()}`)
  return res.json() as Promise<T>
}

export const registryApi = {
  search: (q: string, page = 1, size = 20) =>
    fetchJson<SearchResult>(`/v1/search?q=${encodeURIComponent(q)}&page=${page}&size=${size}`),

  getPackage: (name: string) =>
    fetchJson<Package>(`/v1/packages/${encodeURIComponent(name)}`),

  getVersions: (name: string) =>
    fetchJson<Array<PackageVersion>>(`/v1/packages/${encodeURIComponent(name)}/versions`),

  getStats: () =>
    fetchJson<RegistryStats>("/v1/stats"),

  getRecent: (limit = 12) =>
    fetchJson<Array<Package>>(`/v1/packages?sort=recent&limit=${limit}`),
}
