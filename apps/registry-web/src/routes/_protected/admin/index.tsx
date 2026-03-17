import { createFileRoute, Link } from "@tanstack/react-router"
import { requireRole } from "@/middleware/role-guard"
import { useRegistryStats } from "@/features/packages/hooks/use-packages"
import { statsQueryOptions, recentPackagesQueryOptions } from "@/features/packages/hooks/use-packages"
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid } from "recharts"
import { Package, Download, Users, ShieldCheck } from "lucide-react"

// Synthetic weekly activity data — replace with real endpoint when available
const WEEK_DATA = [
  { day: "Mon", publishes: 2, downloads: 140 },
  { day: "Tue", publishes: 5, downloads: 310 },
  { day: "Wed", publishes: 3, downloads: 280 },
  { day: "Thu", publishes: 7, downloads: 520 },
  { day: "Fri", publishes: 4, downloads: 390 },
  { day: "Sat", publishes: 1, downloads: 210 },
  { day: "Sun", publishes: 2, downloads: 175 },
]

export const Route = createFileRoute("/_protected/admin/")({
  beforeLoad: async () => requireRole("admin"),
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.prefetchQuery(statsQueryOptions),
      queryClient.prefetchQuery(recentPackagesQueryOptions),
    ]),
  head: () => ({ meta: [{ title: "Admin — tsx registry" }] }),
  component: AdminOverviewPage,
})

function AdminOverviewPage() {
  const { data: stats } = useRegistryStats()

  return (
    <div className="page-wrap py-12 rise-in">
      <div className="mb-8 flex items-center gap-3">
        <ShieldCheck className="size-6" style={{ color: "var(--lagoon)" }} />
        <div>
          <h1 className="text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>Admin Dashboard</h1>
          <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>Registry management</p>
        </div>
      </div>

      {/* Nav */}
      <div className="mb-8 flex flex-wrap gap-2">
        {[
          { to: "/admin", label: "Overview" },
          { to: "/admin/packages", label: "Packages" },
          { to: "/admin/users", label: "Users" },
          { to: "/admin/audit-log", label: "Audit Log" },
          { to: "/admin/rate-limits", label: "Rate Limits" },
        ].map(({ to, label }) => (
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

      {/* Stat cards */}
      <div className="mb-8 grid grid-cols-2 gap-4 lg:grid-cols-4">
        {[
          { label: "Total packages", value: stats?.total_packages ?? "—", icon: Package },
          { label: "Total downloads", value: stats?.total_downloads.toLocaleString() ?? "—", icon: Download },
          { label: "Total versions", value: stats?.total_versions ?? "—", icon: Package },
          { label: "New this week", value: `+${stats?.packages_this_week ?? "—"}`, icon: Users },
        ].map(({ label, value, icon: Icon }) => (
          <div key={label} className="island-shell rounded-xl p-5">
            <div className="mb-2 flex items-center gap-2" style={{ color: "var(--sea-ink-soft)" }}>
              <Icon className="size-4" style={{ color: "var(--lagoon)" }} />
              <span className="text-xs font-medium">{label}</span>
            </div>
            <p className="text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>{value}</p>
          </div>
        ))}
      </div>

      {/* Charts */}
      <div className="grid gap-6 lg:grid-cols-2">
        <div className="island-shell rounded-xl p-5">
          <p className="mb-1 font-semibold text-sm" style={{ color: "var(--sea-ink)" }}>Weekly publishes</p>
          <p className="mb-4 text-xs" style={{ color: "var(--sea-ink-soft)" }}>Placeholder — wire to audit log endpoint</p>
          <ResponsiveContainer width="100%" height={180}>
            <BarChart data={WEEK_DATA} margin={{ left: -20 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="var(--line)" />
              <XAxis dataKey="day" tick={{ fontSize: 11 }} />
              <YAxis tick={{ fontSize: 11 }} />
              <Tooltip contentStyle={{ fontSize: 12 }} />
              <Bar dataKey="publishes" fill="var(--lagoon)" radius={[3, 3, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>

        <div className="island-shell rounded-xl p-5">
          <p className="mb-1 font-semibold text-sm" style={{ color: "var(--sea-ink)" }}>Weekly downloads</p>
          <p className="mb-4 text-xs" style={{ color: "var(--sea-ink-soft)" }}>Placeholder — wire to download stats endpoint</p>
          <ResponsiveContainer width="100%" height={180}>
            <BarChart data={WEEK_DATA} margin={{ left: -20 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="var(--line)" />
              <XAxis dataKey="day" tick={{ fontSize: 11 }} />
              <YAxis tick={{ fontSize: 11 }} />
              <Tooltip contentStyle={{ fontSize: 12 }} />
              <Bar dataKey="downloads" fill="var(--palm)" radius={[3, 3, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  )
}
