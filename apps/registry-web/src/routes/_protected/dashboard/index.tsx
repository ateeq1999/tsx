import { Link, createFileRoute  } from "@tanstack/react-router"
import { ArrowRight, Download, Layers, Package, Zap } from "lucide-react"
import { requireAuth } from "@/middleware/auth-guard"
import { recentPackagesQueryOptions, statsQueryOptions, useRecentPackages, useRegistryStats } from "@/features/packages/hooks/use-packages"

import { Button } from "@/components/ui/button"

export const Route = createFileRoute("/_protected/dashboard/")({
  beforeLoad: async () => requireAuth(),
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.prefetchQuery(statsQueryOptions),
      queryClient.prefetchQuery(recentPackagesQueryOptions),
    ]),
  component: DashboardPage,
})

function DashboardPage() {
  const { user } = Route.useRouteContext()
  const { data: stats } = useRegistryStats()
  const { data: recent } = useRecentPackages()

  return (
    <div className="page-wrap py-12 rise-in">
      <div className="mb-8 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>
            Registry Dashboard
          </h1>
          <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>
            Welcome back, {user.name}
          </p>
        </div>
        <Button asChild>
          <Link to="/browse">
            Browse packages <ArrowRight className="ml-1 size-4" />
          </Link>
        </Button>
      </div>

      {/* Stats */}
      <div className="mb-8 grid grid-cols-2 gap-4 lg:grid-cols-4">
        {[
          { label: "Total packages", value: stats?.total_packages ?? "—", icon: Package },
          { label: "Total downloads", value: stats?.total_downloads.toLocaleString() ?? "—", icon: Download },
          { label: "Total versions", value: stats?.total_versions ?? "—", icon: Layers },
          { label: "This week", value: `+${stats?.packages_this_week ?? "—"}`, icon: Zap },
        ].map(({ label, value, icon: Icon }) => (
          <div key={label} className="island-shell rounded-xl p-5">
            <div className="mb-3 flex items-center gap-2" style={{ color: "var(--sea-ink-soft)" }}>
              <Icon className="size-4" style={{ color: "var(--lagoon)" }} />
              <span className="text-xs font-medium">{label}</span>
            </div>
            <p className="text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>{value}</p>
          </div>
        ))}
      </div>

      {/* Recent packages table */}
      <div className="island-shell rounded-xl p-6">
        <div className="mb-4 flex items-center justify-between">
          <h2 className="font-bold" style={{ color: "var(--sea-ink)" }}>Recent packages</h2>
          <Link to="/browse" className="nav-link text-xs">View all</Link>
        </div>

        {recent && recent.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr style={{ borderBottom: "1px solid var(--line)" }}>
                  {["Package", "Version", "Downloads", "Author", "Published"].map((h) => (
                    <th
                      key={h}
                      className="pb-3 text-left text-xs font-semibold"
                      style={{ color: "var(--sea-ink-soft)" }}
                    >
                      {h}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {recent.map((pkg) => (
                  <tr
                    key={pkg.name}
                    style={{ borderBottom: "1px solid var(--line)" }}
                    className="hover:bg-black/[0.02] dark:hover:bg-white/[0.02]"
                  >
                    <td className="py-3">
                      <Link
                        to="/packages/$name"
                        params={{ name: pkg.name }}
                        className="font-mono font-bold hover:underline"
                        style={{ color: "var(--lagoon-deep)" }}
                      >
                        {pkg.name}
                      </Link>
                    </td>
                    <td className="py-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                      v{pkg.version}
                    </td>
                    <td className="py-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                      {pkg.download_count.toLocaleString()}
                    </td>
                    <td className="py-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                      {pkg.author}
                    </td>
                    <td className="py-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                      {new Date(pkg.created_at).toLocaleDateString()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="py-8 text-center text-sm" style={{ color: "var(--sea-ink-soft)" }}>
            No packages yet
          </div>
        )}
      </div>
    </div>
  )
}
