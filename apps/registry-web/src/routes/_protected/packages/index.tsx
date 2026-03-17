import { Link, createFileRoute } from "@tanstack/react-router"
import { queryOptions, useQuery } from "@tanstack/react-query"
import { Package, ExternalLink } from "lucide-react"
import { requireAuth } from "@/middleware/auth-guard"
import { registryApi } from "@/lib/api"
import { Button } from "@/components/ui/button"

const myPackagesQueryOptions = (authorName: string) =>
  queryOptions({
    queryKey: ["packages", "by-author", authorName],
    queryFn: async () => {
      const result = await registryApi.search(authorName)
      return result.packages.filter(
        (p) => p.author.toLowerCase() === authorName.toLowerCase()
      )
    },
    enabled: !!authorName,
  })

export const Route = createFileRoute("/_protected/packages/")({
  beforeLoad: async () => requireAuth(),
  head: () => ({ meta: [{ title: "My Packages — tsx registry" }] }),
  component: MyPackagesPage,
})

function MyPackagesPage() {
  const { user } = Route.useRouteContext()
  const { data: packages, isLoading } = useQuery(myPackagesQueryOptions(user.name ?? ""))

  return (
    <div className="page-wrap py-12 rise-in">
      <div className="mb-8 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>My Packages</h1>
          <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>
            Packages published under <strong>{user.name}</strong>
          </p>
        </div>
        <Button asChild>
          <Link to="/publish">Publish new</Link>
        </Button>
      </div>

      {isLoading ? (
        <div className="space-y-3">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="island-shell h-16 animate-pulse rounded-xl" />
          ))}
        </div>
      ) : !packages || packages.length === 0 ? (
        <div className="island-shell rounded-xl p-12 text-center">
          <Package className="mx-auto mb-3 size-10 opacity-30" style={{ color: "var(--lagoon)" }} />
          <p className="font-semibold" style={{ color: "var(--sea-ink)" }}>No packages yet</p>
          <p className="mt-1 text-sm" style={{ color: "var(--sea-ink-soft)" }}>
            Publish your first package to see it here.
          </p>
          <Button asChild className="mt-4">
            <Link to="/publish">Publish a package</Link>
          </Button>
        </div>
      ) : (
        <div className="island-shell rounded-xl overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr style={{ borderBottom: "1px solid var(--line)", background: "var(--code-bg)" }}>
                {["Package", "Version", "Downloads", "Updated", ""].map((h) => (
                  <th
                    key={h}
                    className="px-4 py-3 text-left text-xs font-semibold"
                    style={{ color: "var(--sea-ink-soft)" }}
                  >
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {packages.map((pkg) => (
                <tr
                  key={pkg.name}
                  style={{ borderBottom: "1px solid var(--line)" }}
                  className="hover:bg-black/[0.02] dark:hover:bg-white/[0.02]"
                >
                  <td className="px-4 py-3">
                    <span className="font-mono font-bold text-sm" style={{ color: "var(--sea-ink)" }}>
                      {pkg.name}
                    </span>
                    {pkg.description && (
                      <p className="mt-0.5 text-xs line-clamp-1" style={{ color: "var(--sea-ink-soft)" }}>
                        {pkg.description}
                      </p>
                    )}
                  </td>
                  <td className="px-4 py-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                    v{pkg.version}
                  </td>
                  <td className="px-4 py-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                    {pkg.download_count.toLocaleString()}
                  </td>
                  <td className="px-4 py-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                    {new Date(pkg.updated_at).toLocaleDateString()}
                  </td>
                  <td className="px-4 py-3">
                    <Link
                      to="/packages/$name"
                      params={{ name: pkg.name }}
                      className="flex items-center gap-1 text-xs hover:underline"
                      style={{ color: "var(--lagoon-deep)" }}
                    >
                      View <ExternalLink className="size-3" />
                    </Link>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
