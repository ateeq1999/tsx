import { Link, createFileRoute } from "@tanstack/react-router"
import { useState } from "react"
import { Search } from "lucide-react"
import { packagesQueryOptions, usePackages } from "@/features/packages/hooks/use-packages"
import { Input } from "@/components/ui/input"

export const Route = createFileRoute("/browse/")({
  loader: ({ context: { queryClient } }) =>
    queryClient.prefetchQuery(packagesQueryOptions("", 1)),
  component: BrowsePage,
})

function BrowsePage() {
  const [q, setQ] = useState("")
  const { data, isLoading } = usePackages(q)

  return (
    <div className="page-wrap py-12 rise-in">
      <h1 className="mb-2 text-3xl font-bold" style={{ color: "var(--sea-ink)" }}>Browse</h1>
      <p className="mb-8 text-sm" style={{ color: "var(--sea-ink-soft)" }}>
        {data?.total ?? "..."} packages available
      </p>

      <div className="relative mb-8 max-w-md">
        <Search className="absolute left-3 top-1/2 size-4 -translate-y-1/2" style={{ color: "var(--sea-ink-soft)" }} />
        <Input
          className="pl-9"
          placeholder="Search packages..."
          value={q}
          onChange={(e) => setQ(e.target.value)}
        />
      </div>

      {isLoading ? (
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 9 }).map((_, i) => (
            <div key={i} className="island-shell h-28 animate-pulse rounded-xl" />
          ))}
        </div>
      ) : data?.packages.length === 0 ? (
        <div className="py-16 text-center" style={{ color: "var(--sea-ink-soft)" }}>
          No packages found for &ldquo;{q}&rdquo;
        </div>
      ) : (
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {data?.packages.map((pkg) => (
            <Link
              key={pkg.name}
              to="/packages/$name"
              params={{ name: pkg.name }}
              className="island-shell rounded-xl p-4 hover:no-underline"
            >
              <div className="mb-1 flex items-start justify-between">
                <span className="font-mono font-bold text-sm" style={{ color: "var(--sea-ink)" }}>
                  {pkg.name}
                </span>
                <span className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>v{pkg.version}</span>
              </div>
              <p className="mb-3 text-xs leading-relaxed line-clamp-2" style={{ color: "var(--sea-ink-soft)" }}>
                {pkg.description}
              </p>
              <div className="flex flex-wrap gap-1">
                {pkg.tags.slice(0, 3).map((tag) => (
                  <span key={tag} className="pkg-tag">{tag}</span>
                ))}
              </div>
              <div className="mt-3 flex items-center gap-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                <span>{pkg.download_count.toLocaleString()} installs</span>
                <span>{pkg.license}</span>
              </div>
            </Link>
          ))}
        </div>
      )}
    </div>
  )
}
