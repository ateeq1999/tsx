import { createFileRoute, redirect } from "@tanstack/react-router";
import { dashboardGuard } from "~/middleware/authGuard";
import { postsQueryOptions } from "~/hooks/use-posts";
import { useQuery } from "@tanstack/react-query";
import { signOut, useSession } from "~/lib/auth-client";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/dashboard/")({
  middleware: () => [dashboardGuard()],
  loader: ({ context: { queryClient } }) =>
    queryClient.ensureQueryData(postsQueryOptions),

  component: DashboardPage,
});

function DashboardPage() {
  const { data: session } = useSession();
  const { data: posts } = useQuery(postsQueryOptions);

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Dashboard</h1>
          <p className="text-muted-foreground">Welcome, {session?.user.name}</p>
        </div>
        <Button variant="outline" onClick={() => signOut()}>
          Sign Out
        </Button>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <div className="rounded-lg border p-4">
          <p className="text-sm text-muted-foreground">Total Posts</p>
          <p className="text-3xl font-bold">{posts?.length ?? 0}</p>
        </div>
      </div>

      <div>
        <h2 className="text-lg font-semibold mb-3">Recent Posts</h2>
        <div className="divide-y rounded-lg border">
          {posts?.map((post) => (
            <div key={post.id} className="flex items-center justify-between p-4">
              <p className="font-medium">{post.title}</p>
              <p className="text-sm text-muted-foreground">
                {new Date(post.createdAt).toLocaleDateString()}
              </p>
            </div>
          ))}
          {!posts?.length && (
            <p className="p-4 text-sm text-muted-foreground">No posts yet.</p>
          )}
        </div>
      </div>
    </div>
  );
}
