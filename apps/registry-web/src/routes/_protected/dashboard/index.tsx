import { createFileRoute } from "@tanstack/react-router"
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { requireAuth } from "@/middleware/auth-guard"

export const Route = createFileRoute("/_protected/dashboard/")({
  beforeLoad: async () => {
    return requireAuth()
  },
  component: Dashboard,
})

function Dashboard() {
  const { user } = Route.useRouteContext()

  return (
    <div className="space-y-6">
      <div className="text-center">
        <h1 className="text-3xl font-bold text-foreground">
          Welcome, {user.name}!
        </h1>
        <p className="text-muted-foreground">Here's your dashboard overview</p>
      </div>

      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        <Card className="h-full">
          <CardHeader>
            <CardTitle>Overview</CardTitle>
            <CardDescription>Summary of your account activity</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-2xl font-semibold">{user.email}</h2>
                <p className="text-sm text-muted-foreground">
                  Active since{" "}
                  {new Date(user.createdAt || Date.now()).toLocaleDateString()}
                </p>
              </div>
              <Badge variant="secondary" className="text-xs">
                Member
              </Badge>
            </div>
            <div className="text-sm text-muted-foreground">
              <p>Status: Active</p>
              <p>Last login: Today</p>
            </div>
          </CardContent>
          <CardFooter>
            <Button variant="outline" size="sm">
              View Profile
            </Button>
          </CardFooter>
        </Card>

        <Card className="h-full">
          <CardHeader>
            <CardTitle>Quick Actions</CardTitle>
            <CardDescription>Common tasks you can perform</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <Button variant="outline" className="w-full justify-start">
              Create New Item
            </Button>
            <Button variant="outline" className="w-full justify-start">
              View Analytics
            </Button>
            <Button variant="outline" className="w-full justify-start">
              Export Data
            </Button>
            <Button variant="outline" className="w-full justify-start">
              Settings
            </Button>
          </CardContent>
        </Card>

        <Card className="h-full">
          <CardHeader>
            <CardTitle>Statistics</CardTitle>
            <CardDescription>Your key metrics at a glance</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="text-center">
                <div className="text-lg font-semibold">12</div>
                <div className="text-xs text-muted-foreground">Projects</div>
              </div>
              <div className="text-center">
                <div className="text-lg font-semibold">24</div>
                <div className="text-xs text-muted-foreground">Tasks</div>
              </div>
              <div className="text-center">
                <div className="text-lg font-semibold">5</div>
                <div className="text-xs text-muted-foreground">Teams</div>
              </div>
              <div className="text-center">
                <div className="text-lg font-semibold">98%</div>
                <div className="text-xs text-muted-foreground">Completion</div>
              </div>
            </div>
          </CardContent>
          <CardFooter className="flex justify-end">
            <Button variant="ghost" size="sm">
              View All
            </Button>
          </CardFooter>
        </Card>
      </div>
    </div>
  )
}
