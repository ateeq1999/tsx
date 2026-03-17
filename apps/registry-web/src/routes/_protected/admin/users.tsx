import { createFileRoute, Link } from "@tanstack/react-router"
import { useQuery, queryOptions } from "@tanstack/react-query"
import { toast } from "sonner"
import { requireRole } from "@/middleware/role-guard"
import { getAdminUsers } from "@/server/admin/queries"
import { Users, CheckCircle, XCircle } from "lucide-react"

const adminUsersQueryOptions = queryOptions({
  queryKey: ["admin", "users"],
  queryFn: () => getAdminUsers(),
  staleTime: 30_000,
})

export const Route = createFileRoute("/_protected/admin/users")({
  beforeLoad: async () => requireRole("admin"),
  loader: ({ context: { queryClient } }) =>
    queryClient.prefetchQuery(adminUsersQueryOptions),
  head: () => ({ meta: [{ title: "Admin: Users — tsx registry" }] }),
  component: AdminUsersPage,
})

function AdminUsersPage() {
  const { data: users, isLoading } = useQuery(adminUsersQueryOptions)

  function handleAction(action: string, email: string) {
    toast.info(`"${action}" on ${email} — not yet wired to backend`)
  }

  return (
    <div className="page-wrap py-12 rise-in">
      <div className="mb-8 flex items-center gap-3">
        <Users className="size-5" style={{ color: "var(--lagoon)" }} />
        <div>
          <h1 className="text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>User Management</h1>
          <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>{users?.length ?? "…"} registered users</p>
        </div>
      </div>

      {/* Nav */}
      <div className="mb-6 flex flex-wrap gap-2">
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
                {["Name", "Email", "Verified", "Joined", "Actions"].map((h) => (
                  <th key={h} className="px-4 py-3 text-left text-xs font-semibold" style={{ color: "var(--sea-ink-soft)" }}>{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {(users ?? []).map((u) => (
                <tr key={u.id} style={{ borderBottom: "1px solid var(--line)" }} className="hover:bg-black/[0.02]">
                  <td className="px-4 py-3 font-medium" style={{ color: "var(--sea-ink)" }}>{u.name}</td>
                  <td className="px-4 py-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>{u.email}</td>
                  <td className="px-4 py-3">
                    {u.emailVerified
                      ? <CheckCircle className="size-4" style={{ color: "#22c55e" }} />
                      : <XCircle className="size-4" style={{ color: "#ef4444" }} />}
                  </td>
                  <td className="px-4 py-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                    {new Date(u.createdAt).toLocaleDateString()}
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex gap-3">
                      <button
                        onClick={() => handleAction("promote to admin", u.email)}
                        className="text-xs hover:underline"
                        style={{ color: "var(--lagoon-deep)" }}
                      >
                        Make admin
                      </button>
                      <button
                        onClick={() => handleAction("suspend", u.email)}
                        className="text-xs hover:underline"
                        style={{ color: "#ef4444" }}
                      >
                        Suspend
                      </button>
                    </div>
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
