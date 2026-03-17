import { createFileRoute, Link } from "@tanstack/react-router"
import { useState } from "react"
import { toast } from "sonner"
import { requireRole } from "@/middleware/role-guard"
import { recentPackagesQueryOptions } from "@/features/packages/hooks/use-packages"
import { useQuery, queryOptions } from "@tanstack/react-query"
import { registryApi } from "@/lib/api"
import { Shield, Trash2, Star, AlertTriangle } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"

const allPackagesQueryOptions = queryOptions({
  queryKey: ["admin", "packages"],
  queryFn: () => registryApi.search("", 1, 50),
  staleTime: 30_000,
})

export const Route = createFileRoute("/_protected/admin/packages")({
  beforeLoad: async () => requireRole("admin"),
  loader: ({ context: { queryClient } }) =>
    queryClient.prefetchQuery(allPackagesQueryOptions),
  head: () => ({ meta: [{ title: "Admin: Packages — tsx registry" }] }),
  component: AdminPackagesPage,
})

function AdminPackagesPage() {
  const { data } = useQuery(allPackagesQueryOptions)
  const [confirmYank, setConfirmYank] = useState<string | null>(null)

  function handleAction(action: string, name: string) {
    toast.info(`Admin action "${action}" on ${name} — not yet wired to backend`)
  }

  return (
    <div className="page-wrap py-12 rise-in">
      <div className="mb-8 flex items-center gap-3">
        <Shield className="size-5" style={{ color: "var(--lagoon)" }} />
        <div>
          <h1 className="text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>Package Moderation</h1>
          <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>{data?.total ?? "…"} total packages</p>
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

      <div className="island-shell rounded-xl overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr style={{ borderBottom: "1px solid var(--line)", background: "var(--code-bg)" }}>
              {["Package", "Version", "Author", "Downloads", "Published", "Actions"].map((h) => (
                <th key={h} className="px-4 py-3 text-left text-xs font-semibold" style={{ color: "var(--sea-ink-soft)" }}>{h}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {(data?.packages ?? []).map((pkg) => (
              <tr key={pkg.name} style={{ borderBottom: "1px solid var(--line)" }} className="hover:bg-black/[0.02]">
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
                <td className="px-4 py-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>v{pkg.version}</td>
                <td className="px-4 py-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>{pkg.author}</td>
                <td className="px-4 py-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>{pkg.download_count.toLocaleString()}</td>
                <td className="px-4 py-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                  {new Date(pkg.created_at).toLocaleDateString()}
                </td>
                <td className="px-4 py-3">
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => handleAction("feature", pkg.name)}
                      title="Feature package"
                      className="rounded p-1 hover:bg-black/10"
                    >
                      <Star className="size-3.5" style={{ color: "var(--lagoon)" }} />
                    </button>
                    <button
                      onClick={() => setConfirmYank(pkg.name)}
                      title="Yank package"
                      className="rounded p-1 hover:bg-black/10"
                    >
                      <AlertTriangle className="size-3.5" style={{ color: "#f59e0b" }} />
                    </button>
                    <button
                      onClick={() => handleAction("delete", pkg.name)}
                      title="Delete package"
                      className="rounded p-1 hover:bg-black/10"
                    >
                      <Trash2 className="size-3.5" style={{ color: "#ef4444" }} />
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Yank confirm dialog */}
      <Dialog open={!!confirmYank} onOpenChange={() => setConfirmYank(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Yank package?</DialogTitle>
            <DialogDescription>
              Yanking <code>{confirmYank}</code> will prevent new installs but won't remove existing ones.
              This requires a backend admin endpoint to take effect.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirmYank(null)}>Cancel</Button>
            <Button
              variant="destructive"
              onClick={() => {
                handleAction("yank", confirmYank!)
                setConfirmYank(null)
              }}
            >
              Yank
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
