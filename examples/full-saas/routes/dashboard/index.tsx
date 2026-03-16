import { createFileRoute } from "@tanstack/react-router";
import { dashboardGuard } from "~/middleware/authGuard";
import { useQuery } from "@tanstack/react-query";
import { organizationsQueryOptions } from "~/hooks/use-organizations";
import { usersQueryOptions } from "~/hooks/use-users";
import { useSession, signOut } from "~/lib/auth-client";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/dashboard/")({
  middleware: () => [dashboardGuard()],
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.ensureQueryData(organizationsQueryOptions),
      queryClient.ensureQueryData(usersQueryOptions),
    ]),
  component: DashboardPage,
});

function DashboardPage() {
  const { data: session } = useSession();
  const { data: orgs } = useQuery(organizationsQueryOptions);
  const { data: users } = useQuery(usersQueryOptions);

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Dashboard</h1>
          <p className="text-muted-foreground">Welcome, {session?.user.name}</p>
        </div>
        <Button variant="outline" onClick={() => signOut()}>Sign Out</Button>
      </div>

      <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
        {[
          { label: "Organizations", value: orgs?.length ?? 0 },
          { label: "Users",         value: users?.length ?? 0 },
          { label: "Plan",          value: "Pro" },
          { label: "Storage",       value: "2.4 GB" },
        ].map((stat) => (
          <div key={stat.label} className="rounded-lg border p-4">
            <p className="text-sm text-muted-foreground">{stat.label}</p>
            <p className="text-3xl font-bold">{stat.value}</p>
          </div>
        ))}
      </div>
    </div>
  );
}
