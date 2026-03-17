import { createFileRoute, Link } from "@tanstack/react-router"
import { queryOptions, useQuery } from "@tanstack/react-query"
import { requireRole } from "@/middleware/role-guard"
import { registryApi } from "@/lib/api"
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
  // Proxy the registry's recent packages as publish events (full audit log requires a backend endpoint)
  queryFn: () => registryApi.search("", 1, 100),
  staleTime: 30_000,
})

export const Route = createFileRoute("/_protected/admin/audit-log")({
  beforeLoad: async () => requireRole("admin"),
  loader: ({ context: { queryClient } }) => queryClient.prefetchQuery(auditQueryOptions),
  head: () => ({ meta: [{ title: "Admin: Audit Log — tsx registry" }] }),
  component: AdminAuditLogPage,
})

function AdminAuditLogPage() {
  const { data } = useQuery(auditQueryOptions)

  return (
    <div className="page-wrap py-12 rise-in">
      <div className="mb-8 flex items-center gap-3">
        <Shield className="size-5" style={{ color: "var(--lagoon)" }} />
        <div>
          <h1 className="text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>Publish Audit Log</h1>
          <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>
            All package publish events, most recent first.
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

      <div className="island-shell rounded-xl overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr style={{ borderBottom: "1px solid var(--line)", background: "var(--code-bg)" }}>
              {["#", "Package", "Version", "Author", "Published at", "Downloads"].map((h) => (
                <th key={h} className="px-4 py-3 text-left text-xs font-semibold" style={{ color: "var(--sea-ink-soft)" }}>
                  {h}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {(data?.packages ?? []).map((pkg, idx) => (
              <tr key={`${pkg.name}-${pkg.version}`} style={{ borderBottom: "1px solid var(--line)" }} className="hover:bg-black/[0.02]">
                <td className="px-4 py-3 text-xs tabular-nums" style={{ color: "var(--sea-ink-soft)" }}>{idx + 1}</td>
                <td className="px-4 py-3">
                  <Link
                    to="/packages/$name"
                    params={{ name: pkg.name }}
                    className="font-mono font-bold hover:underline"
                    style={{ color: "var(--lagoon-deep)" }}
                  >
                    {pkg.name}
                  </Link>
                </td>
                <td className="px-4 py-3 font-mono text-xs" style={{ color: "var(--sea-ink)" }}>v{pkg.version}</td>
                <td className="px-4 py-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>{pkg.author}</td>
                <td className="px-4 py-3 text-xs tabular-nums" style={{ color: "var(--sea-ink-soft)" }}>
                  {new Date(pkg.created_at).toLocaleString()}
                </td>
                <td className="px-4 py-3 text-xs tabular-nums" style={{ color: "var(--sea-ink-soft)" }}>
                  {pkg.download_count.toLocaleString()}
                </td>
              </tr>
            ))}
            {(!data?.packages || data.packages.length === 0) && (
              <tr>
                <td colSpan={6} className="px-4 py-8 text-center text-sm" style={{ color: "var(--sea-ink-soft)" }}>
                  <ClipboardList className="mx-auto mb-2 size-6 opacity-40" />
                  No publish events found.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      <p className="mt-4 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
        Full audit log with IP addresses and detailed event history requires a backend endpoint
        (<code>GET /v1/admin/audit-log</code>). Currently showing all published packages ordered by publish date.
      </p>
    </div>
  )
}
