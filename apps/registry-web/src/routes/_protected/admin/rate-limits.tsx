import { createFileRoute, Link } from "@tanstack/react-router"
import { useQuery, queryOptions } from "@tanstack/react-query"
import { requireRole } from "@/middleware/role-guard"
import { getAdminRateLimits } from "@/server/admin/queries"
import { Shield, Activity, Ban, RefreshCw } from "lucide-react"
import { Button } from "@/components/ui/button"

const ADMIN_NAV = [
  { to: "/admin", label: "Overview" },
  { to: "/admin/packages", label: "Packages" },
  { to: "/admin/users", label: "Users" },
  { to: "/admin/audit-log", label: "Audit Log" },
  { to: "/admin/rate-limits", label: "Rate Limits" },
]

const rateLimitsQueryOptions = queryOptions({
  queryKey: ["admin", "rate-limits"],
  queryFn: () => getAdminRateLimits(),
  staleTime: 15_000,
  refetchInterval: 30_000,
})

export const Route = createFileRoute("/_protected/admin/rate-limits")({
  beforeLoad: async () => requireRole("admin"),
  loader: ({ context: { queryClient } }) => queryClient.prefetchQuery(rateLimitsQueryOptions),
  head: () => ({ meta: [{ title: "Admin: Rate Limits — tsx registry" }] }),
  component: AdminRateLimitsPage,
})

function AdminRateLimitsPage() {
  const { data = [], isLoading, refetch, isFetching } = useQuery(rateLimitsQueryOptions)

  const blocked = data.filter((r) => r.blocked)
  const near = data.filter((r) => !r.blocked && r.requests >= r.limit * 0.8)

  return (
    <div className="page-wrap py-12 rise-in">
      <div className="mb-8 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Shield className="size-5" style={{ color: "var(--lagoon)" }} />
          <div>
            <h1 className="text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>Rate Limit Monitor</h1>
            <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>
              Publish-rate per IP · live from registry-server
            </p>
          </div>
        </div>
        <Button size="sm" variant="outline" onClick={() => refetch()} disabled={isFetching}>
          <RefreshCw className={`mr-1.5 size-3.5 ${isFetching ? "animate-spin" : ""}`} />
          Refresh
        </Button>
      </div>

      {/* Nav */}
      <div className="mb-6 flex flex-wrap gap-2">
        {ADMIN_NAV.map(({ to, label }) => (
          <Link
            key={to}
            to={to as "/admin"}
            className="rounded-lg border px-4 py-2 text-sm font-medium transition-colors hover:no-underline"
            activeProps={{ style: { background: "var(--lagoon)", color: "#fff", borderColor: "var(--lagoon)" } }}
            style={{ borderColor: "var(--line)", color: "var(--sea-ink)" }}
          >
            {label}
          </Link>
        ))}
      </div>

      {/* Summary cards */}
      <div className="mb-6 grid gap-4 sm:grid-cols-3">
        {[
          { label: "IPs tracked", value: data.length, icon: Activity, color: "var(--lagoon)" },
          { label: "Currently blocked", value: blocked.length, icon: Ban, color: "#ef4444" },
          { label: "Near limit (≥80%)", value: near.length, icon: Activity, color: "#f59e0b" },
        ].map(({ label, value, icon: Icon, color }) => (
          <div key={label} className="island-shell rounded-xl p-4">
            <div className="flex items-center gap-2 mb-1">
              <Icon className="size-4" style={{ color }} />
              <span className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>{label}</span>
            </div>
            <p className="text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>{value}</p>
          </div>
        ))}
      </div>

      {/* IP table */}
      {isLoading ? (
        <div className="space-y-2">
          {Array.from({ length: 5 }).map((_, i) => (
            <div key={i} className="island-shell h-12 animate-pulse rounded-xl" />
          ))}
        </div>
      ) : (
        <div className="island-shell rounded-xl overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr style={{ borderBottom: "1px solid var(--line)", background: "var(--code-bg)" }}>
                {["IP Address", "Requests / limit", "Usage", "Window remaining", "Status"].map((h) => (
                  <th key={h} className="px-4 py-3 text-left text-xs font-semibold" style={{ color: "var(--sea-ink-soft)" }}>
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {data.map((row) => {
                const pct = Math.min(100, (row.requests / row.limit) * 100)
                const barColor = row.blocked ? "#ef4444" : pct >= 80 ? "#f59e0b" : "var(--lagoon)"
                const windowLabel = row.window_secs_remaining > 0
                  ? `${row.window_secs_remaining}s`
                  : "—"
                return (
                  <tr key={row.ip} style={{ borderBottom: "1px solid var(--line)" }} className="hover:bg-black/[0.02]">
                    <td className="px-4 py-3 font-mono text-xs" style={{ color: "var(--sea-ink)" }}>{row.ip}</td>
                    <td className="px-4 py-3 text-xs tabular-nums" style={{ color: "var(--sea-ink-soft)" }}>
                      {row.requests} / {row.limit}
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-2">
                        <div className="h-1.5 w-24 rounded-full overflow-hidden" style={{ background: "var(--line)" }}>
                          <div
                            className="h-full rounded-full transition-all"
                            style={{ width: `${pct}%`, background: barColor }}
                          />
                        </div>
                        <span className="text-xs tabular-nums" style={{ color: "var(--sea-ink-soft)" }}>
                          {Math.round(pct)}%
                        </span>
                      </div>
                    </td>
                    <td className="px-4 py-3 font-mono text-xs tabular-nums" style={{ color: "var(--sea-ink-soft)" }}>
                      {windowLabel}
                    </td>
                    <td className="px-4 py-3">
                      <span
                        className="rounded-full px-2 py-0.5 text-[10px] font-semibold"
                        style={
                          row.blocked
                            ? { background: "#fecaca", color: "#dc2626" }
                            : pct >= 80
                            ? { background: "#fef3c7", color: "#d97706" }
                            : { background: "#d1fae5", color: "#059669" }
                        }
                      >
                        {row.blocked ? "blocked" : pct >= 80 ? "near limit" : "ok"}
                      </span>
                    </td>
                  </tr>
                )
              })}
              {data.length === 0 && (
                <tr>
                  <td colSpan={5} className="px-4 py-10 text-center text-sm" style={{ color: "var(--sea-ink-soft)" }}>
                    <Activity className="mx-auto mb-2 size-6 opacity-40" />
                    No active rate-limit entries.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
