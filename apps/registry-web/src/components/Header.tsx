import { Link, useNavigate, useRouter } from "@tanstack/react-router"
import { Menu, Package2 } from "lucide-react"
import { useQueryClient } from "@tanstack/react-query"
import { toast } from "sonner"
import { BaseHeader } from "@tsx/ui/base-header"
import { ThemeToggle } from "./ThemeToggle"
import { useSession } from "@/hooks/use-session"
import { logoutFn } from "@/server/auth/mutations"
import { Avatar, AvatarFallback } from "@/components/ui/avatar"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet"

const NAV_LINKS = [
  { to: "/browse" as const, label: "Browse" },
]

export function Header() {
  const { data: session } = useSession()
  const queryClient = useQueryClient()
  const navigate = useNavigate()
  const router = useRouter()

  async function handleLogout() {
    await logoutFn()
    await queryClient.invalidateQueries({ queryKey: ["session"] })
    router.invalidate()
    toast.success("Signed out")
    navigate({ to: "/" })
  }

  const initials = session?.user?.name
    ? session.user.name.split(" ").map((n: string) => n[0]).join("").toUpperCase().slice(0, 2)
    : "?"

  return (
    <BaseHeader
      logo={
        <Link to="/" className="flex items-center gap-2 font-bold" style={{ color: "var(--sea-ink)" }}>
          <Package2 className="size-5" style={{ color: "var(--lagoon)" }} />
          <span>tsx registry</span>
        </Link>
      }
      nav={
        <nav className="hidden items-center gap-6 text-sm lg:flex">
          {NAV_LINKS.map(({ to, label }) => (
            <Link key={to} to={to} className="nav-link" activeProps={{ className: "nav-link is-active" }}>
              {label}
            </Link>
          ))}
          {session ? (
            <Link to="/dashboard" className="nav-link" activeProps={{ className: "nav-link is-active" }}>
              Dashboard
            </Link>
          ) : null}
          <a
            href="https://github.com/ateeq1999/tsx"
            target="_blank"
            rel="noreferrer"
            className="nav-link"
          >
            GitHub
          </a>
        </nav>
      }
      right={
        <>
          <ThemeToggle />

          {session ? (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="icon" className="rounded-full">
                  <Avatar className="size-7">
                    <AvatarFallback className="text-xs" style={{ background: "var(--lagoon)", color: "#fff" }}>
                      {initials}
                    </AvatarFallback>
                  </Avatar>
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-44">
                <DropdownMenuLabel className="text-xs font-normal" style={{ color: "var(--sea-ink-soft)" }}>
                  {session.user.email}
                </DropdownMenuLabel>
                <DropdownMenuSeparator />
                <DropdownMenuItem asChild>
                  <Link to="/dashboard">Dashboard</Link>
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem
                  onClick={handleLogout}
                  className="cursor-pointer text-red-500 focus:text-red-500"
                >
                  Sign out
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          ) : (
            <Button asChild size="sm" variant="outline" className="hidden lg:inline-flex">
              <Link to="/auth/login">Sign in</Link>
            </Button>
          )}

          <Sheet>
            <SheetTrigger asChild>
              <Button variant="ghost" size="icon" className="lg:hidden">
                <Menu className="size-5" />
                <span className="sr-only">Open menu</span>
              </Button>
            </SheetTrigger>
            <SheetContent side="left" className="w-64 pt-10">
              <nav className="flex flex-col gap-1">
                {NAV_LINKS.map(({ to, label }) => (
                  <Link
                    key={to}
                    to={to}
                    className="nav-link rounded-md px-3 py-2 text-sm"
                    activeProps={{ className: "nav-link is-active rounded-md px-3 py-2 text-sm font-semibold" }}
                  >
                    {label}
                  </Link>
                ))}
                {session ? (
                  <Link
                    to="/dashboard"
                    className="nav-link rounded-md px-3 py-2 text-sm"
                    activeProps={{ className: "nav-link is-active rounded-md px-3 py-2 text-sm font-semibold" }}
                  >
                    Dashboard
                  </Link>
                ) : null}
                <a
                  href="https://github.com/ateeq1999/tsx"
                  target="_blank"
                  rel="noreferrer"
                  className="nav-link rounded-md px-3 py-2 text-sm"
                >
                  GitHub
                </a>
                <div className="my-2" style={{ borderTop: "1px solid var(--line)" }} />
                {session ? (
                  <button
                    onClick={handleLogout}
                    className="rounded-md px-3 py-2 text-left text-sm text-red-500 hover:bg-red-50 dark:hover:bg-red-950/20"
                  >
                    Sign out
                  </button>
                ) : (
                  <Link
                    to="/auth/login"
                    className="nav-link rounded-md px-3 py-2 text-sm"
                  >
                    Sign in
                  </Link>
                )}
              </nav>
            </SheetContent>
          </Sheet>
        </>
      }
    />
  )
}
