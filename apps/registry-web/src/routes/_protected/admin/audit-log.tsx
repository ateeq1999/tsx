import { createFileRoute, Link } from "@tanstack/react-router"
import { queryOptions, useQuery } from "@tanstack/react-query"
import { requireRole } from "@/middleware/role-guard"
import { getAdminAuditLog } from "@/server/admin/queries"
import { ClipboardList, Shield } from "lucide-react"

const ADMIN_NAV = [
  { to: "/admin", label: "Overview" },
  { to: "/admin/packages", label: "Packages" },
  { to: "/admin/users", label: "Users" },
  { to: "/admin/audit-log", label: "Audit Log" },
  { to: "/admin/rate-limits", label: "Rate Limits" },
]

const auditQueryOptions = queryOptions({
  queryKey: ["admin", "audit-log"],
  queryFn: () => getAdminAuditLog(),
  staleTime: 30_000,
})

export const Route = createFileRoute("/_protected/admin/audit-log")({
  beforeLoad: async () => requireRole("admin"),
  loader: ({ context: { queryClient } }) => queryClient.prefetchQuery(auditQueryOptions),
  head: () => ({ meta: [{ title: "Admin: Audit Log — tsx registry" }] }),
  component: AdminAuditLogPage,
})

function AdminAuditLogPage() {
  const { data, isLoading } = useQuery(auditQueryOptions)

  const ACTION_COLORS: Record<string, { bg: string; color: string }> = {
    publish:       { bg: "#d1fae5", color: "#059669" },
    yank:          { bg: "#fef3c7", color: "#d97706" },
    delete:        { bg: "#fecaca", color: "#dc2626" },
    update_readme: { bg: "#dbeafe", color: "#2563eb" },
    update_meta:   { bg: "#ede9fe", color: "#7c3aed" },
  }

  return (
    <div className="page-wrap py-12 rise-in">
      <div className="mb-8 flex items-center gap-3">
        <Shield className="size-5" style={{ color: "var(--lagoon)" }} />
        <div>
          <h1 className="text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>Publish Audit Log</h1>
          <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>
            All package events, most recent first.
          </p>
        </div>
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

      {isLoading ? (
        <div className="space-y-2">
          {Array.from({ length: 8 }).map((_, i) => (
            <div key={i} className="island-shell h-12 animate-pulse rounded-lg" />
          ))}
        </div>
      ) : (
        <div className="island-shell rounded-xl overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr style={{ borderBottom: "1px solid var(--line)", background: "var(--code-bg)" }}>
                {["Action", "Package", "Version", "Author", "IP", "Time"].map((h) => (
                  <th key={h} className="px-4 py-3 text-left text-xs font-semibold" style={{ color: "var(--sea-ink-soft)" }}>
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {(data ?? []).map((entry) => {
                const colors = ACTION_COLORS[entry.action] ?? { bg: "var(--line)", color: "var(--sea-ink-soft)" }
                return (
                  <tr key={entry.id} style={{ borderBottom: "1px solid var(--line)" }} className="hover:bg-black/[0.02]">
                    <td className="px-4 py-3">
                      <span
                        className="rounded-full px-2 py-0.5 text-[10px] font-semibold"
                        style={{ background: colors.bg, color: colors.color }}
                      >
                        {entry.action}
                      </span>
                    </td>
                    <td className="px-4 py-3">
                      <Link
                        to="/packages/$name"
                        params={{ name: entry.package_name }}
                        className="font-mono font-bold hover:underline text-xs"
                        style={{ color: "var(--lagoon-deep)" }}
                      >
                        {entry.package_name}
                      </Link>
                    </td>
                    <td className="px-4 py-3 font-mono text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                      {entry.version ?? "—"}
                    </td>
                    <td className="px-4 py-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                      {entry.author_name ?? "—"}
                    </td>
                    <td className="px-4 py-3 font-mono text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                      {entry.ip_address ?? "—"}
                    </td>
                    <td className="px-4 py-3 text-xs tabular-nums" style={{ color: "var(--sea-ink-soft)" }}>
                      {entry.created_at ? new Date(entry.created_at).toLocaleString() : "—"}
                    </td>
                  </tr>
                )
              })}
              {(!data || data.length === 0) && (
                <tr>
                  <td colSpan={6} className="px-4 py-10 text-center text-sm" style={{ color: "var(--sea-ink-soft)" }}>
                    <ClipboardList className="mx-auto mb-2 size-6 opacity-40" />
                    No audit events yet.
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
