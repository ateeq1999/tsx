import { createFileRoute, useRouter } from "@tanstack/react-router"
import { queryOptions, useQuery, useQueryClient } from "@tanstack/react-query"
import { toast } from "sonner"
import { requireAuth } from "@/middleware/auth-guard"
import { listSessionsFn, revokeSessionFn } from "@/server/auth/mutations"
import { Monitor, Smartphone, Globe } from "lucide-react"
import { Button } from "@/components/ui/button"

const sessionsQueryOptions = queryOptions({
  queryKey: ["account", "sessions"],
  queryFn: () => listSessionsFn(),
  staleTime: 30_000,
})

export const Route = createFileRoute("/_protected/account/sessions")({
  beforeLoad: async () => requireAuth(),
  loader: ({ context: { queryClient } }) => queryClient.prefetchQuery(sessionsQueryOptions),
  head: () => ({ meta: [{ title: "Sessions — tsx registry" }] }),
  component: SessionsPage,
})

function deviceIcon(ua: string | null) {
  if (!ua) return Globe
  const lower = ua.toLowerCase()
  if (lower.includes("mobile") || lower.includes("android") || lower.includes("iphone")) return Smartphone
  return Monitor
}

function SessionsPage() {
  const { user } = Route.useRouteContext()
  const { data: sessions, isLoading } = useQuery(sessionsQueryOptions)
  const queryClient = useQueryClient()
  const router = useRouter()

  async function revoke(token: string) {
    try {
      await revokeSessionFn({ data: { token } })
      toast.success("Session revoked")
      queryClient.invalidateQueries({ queryKey: ["account", "sessions"] })
      router.invalidate()
    } catch {
      toast.error("Failed to revoke session")
    }
  }

  return (
    <div className="page-wrap py-12 rise-in">
      <h1 className="mb-2 text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>Active sessions</h1>
      <p className="mb-8 text-sm" style={{ color: "var(--sea-ink-soft)" }}>
        These devices are currently signed in as <strong>{user.email}</strong>.
      </p>

      {isLoading ? (
        <div className="space-y-3 max-w-lg">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="island-shell h-20 animate-pulse rounded-xl" />
          ))}
        </div>
      ) : (
        <div className="max-w-lg space-y-3">
          {(sessions ?? []).map((s) => {
            const Icon = deviceIcon(s.userAgent ?? null)
            const isExpired = new Date(s.expiresAt) < new Date()
            return (
              <div
                key={s.id}
                className="island-shell flex items-center justify-between gap-4 rounded-xl p-4"
              >
                <div className="flex items-center gap-3">
                  <Icon className="size-5 shrink-0" style={{ color: "var(--lagoon)" }} />
                  <div>
                    <p className="text-sm font-medium truncate max-w-xs" style={{ color: "var(--sea-ink)" }}>
                      {s.userAgent ? s.userAgent.slice(0, 60) : "Unknown device"}
                    </p>
                    <p className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                      {s.ipAddress ?? "Unknown IP"} ·{" "}
                      {isExpired ? (
                        <span style={{ color: "#f87171" }}>Expired</span>
                      ) : (
                        `Expires ${new Date(s.expiresAt).toLocaleDateString()}`
                      )}
                    </p>
                  </div>
                </div>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => revoke(s.token)}
                  className="shrink-0 text-xs"
                  style={{ color: "#ef4444", borderColor: "#ef4444" }}
                >
                  Revoke
                </Button>
              </div>
            )
          })}
          {sessions?.length === 0 && (
            <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>No active sessions found.</p>
          )}
        </div>
      )}
    </div>
  )
}
