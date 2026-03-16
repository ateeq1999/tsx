import { createFileRoute } from "@tanstack/react-router"
import { dashboardGuard } from "#/middleware/authGuard"
import { useOrganizations, organizationsQueryOptions } from "#/features/organizations/hooks/use-organizations"
import { useSubscription, subscriptionQueryOptions } from "#/features/billing/hooks/use-billing"
import { useSession, signOut } from "#/lib/auth-client"
import { Button } from "#/components/ui/button"
import { Building2, Users, CreditCard, HardDrive } from "lucide-react"

export const Route = createFileRoute("/dashboard/")({
  middleware: () => [dashboardGuard()],
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.ensureQueryData(organizationsQueryOptions),
      queryClient.ensureQueryData(subscriptionQueryOptions),
    ]),
  component: DashboardPage,
})

function DashboardPage() {
  const { data: session } = useSession()
  const { data: orgs = [] } = useOrganizations()
  const { data: sub } = useSubscription()

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Dashboard</h1>
          <p className="text-sm text-muted-foreground">Welcome, {session?.user.name}</p>
        </div>
        <Button variant="outline" size="sm" onClick={() => signOut()}>Sign Out</Button>
      </div>

      <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
        {[
          { label: "Organizations", value: orgs.length, icon: Building2 },
          { label: "Members", value: orgs.reduce((acc, o) => acc + (o.memberCount ?? 0), 0), icon: Users },
          { label: "Plan", value: sub?.plan ?? "Free", icon: CreditCard },
          { label: "Storage", value: sub?.storageUsed ?? "—", icon: HardDrive },
        ].map(({ label, value, icon: Icon }) => (
          <div key={label} className="rounded-lg border p-4">
            <div className="flex items-center gap-2 mb-2">
              <Icon className="size-4 text-muted-foreground" />
              <p className="text-sm text-muted-foreground">{label}</p>
            </div>
            <p className="text-3xl font-bold">{value}</p>
          </div>
        ))}
      </div>

      {/* Orgs table */}
      <div className="rounded-lg border">
        <div className="p-4 border-b">
          <h2 className="font-semibold">Organizations</h2>
        </div>
        <div className="divide-y">
          {orgs.length === 0 ? (
            <p className="p-4 text-sm text-muted-foreground">No organizations yet.</p>
          ) : (
            orgs.map((org) => (
              <div key={org.id} className="flex items-center justify-between p-4">
                <div>
                  <p className="font-medium">{org.name}</p>
                  <p className="text-xs text-muted-foreground">{org.slug}</p>
                </div>
                <span className="text-sm text-muted-foreground">{org.memberCount ?? 0} members</span>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  )
}
