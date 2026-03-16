import { createFileRoute, notFound } from "@tanstack/react-router"
import { useQuery } from "@tanstack/react-query"
import { Clock, Download, Tag, User } from "lucide-react"
import { packageQueryOptions, packageVersionsQueryOptions, usePackage } from "@/features/packages/hooks/use-packages"
import { Badge } from "@/components/ui/badge"

export const Route = createFileRoute("/packages/$name")({
  loader: async ({ context: { queryClient }, params: { name } }) => {
    try {
      await queryClient.ensureQueryData(packageQueryOptions(name))
    } catch {
      throw notFound()
    }
    queryClient.prefetchQuery(packageVersionsQueryOptions(name))
  },
  notFoundComponent: () => (
    <div className="page-wrap py-24 text-center" style={{ color: "var(--sea-ink-soft)" }}>
      Package not found.
    </div>
  ),
  component: PackageDetailPage,
})

function PackageDetailPage() {
  const { name } = Route.useParams()
  const { data: pkg } = usePackage(name)
  const { data: versions } = useQuery(packageVersionsQueryOptions(name))

  if (!pkg) return null

  return (
    <div className="page-wrap py-12 rise-in">
      <div className="mb-8">
        <div className="mb-2 flex items-center gap-3">
          <h1 className="font-mono text-3xl font-bold" style={{ color: "var(--sea-ink)" }}>
            {pkg.name}
          </h1>
          <Badge variant="secondary">v{pkg.version}</Badge>
        </div>
        <p className="mb-4 text-sm leading-relaxed" style={{ color: "var(--sea-ink-soft)" }}>
          {pkg.description}
        </p>
        <div className="flex flex-wrap gap-1">
          {pkg.tags.map((tag) => (
            <span key={tag} className="pkg-tag">{tag}</span>
          ))}
        </div>
      </div>

      {/* Install command */}
      <div className="island-shell mb-8 rounded-xl p-4">
        <p className="island-kicker mb-2">Install</p>
        <div className="flex items-center gap-3 rounded-lg bg-black/5 px-4 py-2 dark:bg-white/5">
          <span style={{ color: "var(--lagoon)" }} className="font-bold">$</span>
          <code className="flex-1 text-sm" style={{ color: "var(--sea-ink)" }}>
            tsx install {pkg.name}
          </code>
          <button
            onClick={() => navigator.clipboard.writeText(`tsx install ${pkg.name}`)}
            className="text-xs opacity-60 hover:opacity-100"
            style={{ color: "var(--lagoon-deep)" }}
          >
            copy
          </button>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-[1fr_280px]">
        {/* Versions */}
        <div className="island-shell rounded-xl p-6">
          <h2 className="mb-4 font-bold" style={{ color: "var(--sea-ink)" }}>Versions</h2>
          {versions ? (
            <div className="space-y-2">
              {versions.map((v) => (
                <div
                  key={v.version}
                  className="flex items-center justify-between text-sm"
                  style={{ borderBottom: "1px solid var(--line)", paddingBottom: "8px" }}
                >
                  <span className="font-mono font-bold" style={{ color: "var(--sea-ink)" }}>
                    v{v.version}
                  </span>
                  <div className="flex items-center gap-4" style={{ color: "var(--sea-ink-soft)" }}>
                    <span>{v.download_count.toLocaleString()} downloads</span>
                    <span>{new Date(v.published_at).toLocaleDateString()}</span>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="space-y-2">
              {Array.from({ length: 4 }).map((_, i) => (
                <div key={i} className="h-8 animate-pulse rounded" style={{ background: "var(--line)" }} />
              ))}
            </div>
          )}
        </div>

        {/* Meta sidebar */}
        <div className="space-y-4">
          <div className="island-shell rounded-xl p-4">
            <h2 className="island-kicker mb-4">Details</h2>
            <div className="space-y-3 text-sm">
              <div className="flex items-center gap-2" style={{ color: "var(--sea-ink-soft)" }}>
                <User className="size-4" />
                <span>{pkg.author}</span>
              </div>
              <div className="flex items-center gap-2" style={{ color: "var(--sea-ink-soft)" }}>
                <Tag className="size-4" />
                <span>{pkg.license}</span>
              </div>
              <div className="flex items-center gap-2" style={{ color: "var(--sea-ink-soft)" }}>
                <Download className="size-4" />
                <span>{pkg.download_count.toLocaleString()} installs</span>
              </div>
              <div className="flex items-center gap-2" style={{ color: "var(--sea-ink-soft)" }}>
                <Clock className="size-4" />
                <span>Updated {new Date(pkg.updated_at).toLocaleDateString()}</span>
              </div>
            </div>
          </div>

          <div className="island-shell rounded-xl p-4">
            <h2 className="island-kicker mb-2">Requires tsx</h2>
            <p className="font-mono text-sm" style={{ color: "var(--sea-ink)" }}>&gt;= {pkg.tsx_min}</p>
          </div>
        </div>
      </div>
    </div>
  )
}
