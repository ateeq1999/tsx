import { queryOptions, useQuery } from "@tanstack/react-query"
import { registryApi } from "@/lib/api"

export const packagesQueryOptions = (q = "", page = 1) =>
  queryOptions({
    queryKey: ["packages", "search", q, page],
    queryFn: () => registryApi.search(q, page),
  })

export const packageQueryOptions = (name: string) =>
  queryOptions({
    queryKey: ["packages", name],
    queryFn: () => registryApi.getPackage(name),
    enabled: !!name,
  })

export const packageVersionsQueryOptions = (name: string) =>
  queryOptions({
    queryKey: ["packages", name, "versions"],
    queryFn: () => registryApi.getVersions(name),
    enabled: !!name,
  })

export const statsQueryOptions = queryOptions({
  queryKey: ["registry", "stats"],
  queryFn: () => registryApi.getStats(),
  staleTime: 60_000,
})

export const recentPackagesQueryOptions = queryOptions({
  queryKey: ["packages", "recent"],
  queryFn: () => registryApi.getRecent(12),
  staleTime: 60_000,
})

export function usePackages(q = "", page = 1) {
  return useQuery(packagesQueryOptions(q, page))
}

export function usePackage(name: string) {
  return useQuery(packageQueryOptions(name))
}

export function useRegistryStats() {
  return useQuery(statsQueryOptions)
}

export function useRecentPackages() {
  return useQuery(recentPackagesQueryOptions)
}

export const packageReadmeQueryOptions = (name: string) =>
  queryOptions({
    queryKey: ["packages", name, "readme"],
    queryFn: () => registryApi.getReadme(name),
    enabled: !!name,
    staleTime: 5 * 60_000,
  })

export const packageDownloadStatsQueryOptions = (name: string) =>
  queryOptions({
    queryKey: ["packages", name, "download-stats"],
    queryFn: () => registryApi.getDownloadStats(name),
    enabled: !!name,
    staleTime: 5 * 60_000,
  })
