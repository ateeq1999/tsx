import { createFileRoute, Link } from "@tanstack/react-router"
import { useState } from "react"
import { requireRole } from "@/middleware/role-guard"
import { Shield, Activity, Ban, RefreshCw } from "lucide-react"
import { Button } from "@/components/ui/button"
import { toast } from "sonner"

const ADMIN_NAV = [
  { to: "/admin", label: "Overview" },
  { to: "/admin/packages", label: "Packages" },
  { to: "/admin/users", label: "Users" },
  { to: "/admin/audit-log", label: "Audit Log" },
  { to: "/admin/rate-limits", label: "Rate Limits" },
]

// Stub data — replace with a real backend endpoint (GET /v1/admin/rate-limits)
const STUB_RATE_DATA = [
  { ip: "198.51.100.42", requests: 47, limit: 60, blocked: false, lastSeen: "2 min ago" },
  { ip: "203.0.113.77", requests: 61, limit: 60, blocked: true, lastSeen: "5 min ago" },
  { ip: "192.0.2.15", requests: 12, limit: 60, blocked: false, lastSeen: "12 min ago" },
  { ip: "198.51.100.8", requests: 59, limit: 60, blocked: false, lastSeen: "1 min ago" },
  { ip: "203.0.113.101", requests: 60, limit: 60, blocked: true, lastSeen: "just now" },
]

export const Route = createFileRoute("/_protected/admin/rate-limits")({
  beforeLoad: async () => requireRole("admin"),
  head: () => ({ meta: [{ title: "Admin: Rate Limits — tsx registry" }] }),
  component: AdminRateLimitsPage,
})

function AdminRateLimitsPage() {
  const [data, setData] = useState(STUB_RATE_DATA)

  function unblock(ip: string) {
    setData((prev) => prev.map((r) => (r.ip === ip ? { ...r, blocked: false, requests: 0 } : r)))
    toast.success(`Unblocked ${ip}`)
  }

  function blockIp(ip: string) {
    setData((prev) => prev.map((r) => (r.ip === ip ? { ...r, blocked: true } : r)))
    toast.success(`Blocked ${ip}`)
  }

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
              Publish-rate per IP (60 req / hr window)
            </p>
          </div>
        </div>
        <Button
          size="sm"
          variant="outline"
          onClick={() => { setData(STUB_RATE_DATA); toast.info("Refreshed (stub data)") }}
        >
          <RefreshCw className="mr-1.5 size-3.5" />
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
      <div className="island-shell rounded-xl overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr style={{ borderBottom: "1px solid var(--line)", background: "var(--code-bg)" }}>
              {["IP Address", "Requests / limit", "Usage", "Last seen", "Status", "Actions"].map((h) => (
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
                  <td className="px-4 py-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>{row.lastSeen}</td>
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
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-2">
                      {row.blocked ? (
                        <button
                          onClick={() => unblock(row.ip)}
                          className="text-xs hover:underline"
                          style={{ color: "var(--lagoon-deep)" }}
                        >
                          Unblock
                        </button>
                      ) : (
                        <button
                          onClick={() => blockIp(row.ip)}
                          className="text-xs hover:underline"
                          style={{ color: "#ef4444" }}
                        >
                          Block
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>

      <p className="mt-4 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
        Live rate-limit data requires a backend endpoint (<code>GET /v1/admin/rate-limits</code>).
        Currently showing stub data for UI demonstration.
      </p>
    </div>
  )
}
