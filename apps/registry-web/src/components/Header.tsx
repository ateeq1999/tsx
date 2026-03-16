import { Link } from "@tanstack/react-router"
import { Package2 } from "lucide-react"
import { ThemeToggle } from "./ThemeToggle"

export function Header() {
  return (
    <header
      style={{ background: "var(--header-bg)", borderBottom: "1px solid var(--line)" }}
      className="sticky top-0 z-50 backdrop-blur-md"
    >
      <div className="page-wrap flex h-14 items-center justify-between gap-4">
        <Link to="/" className="flex items-center gap-2 font-bold" style={{ color: "var(--sea-ink)" }}>
          <Package2 className="size-5" style={{ color: "var(--lagoon)" }} />
          <span>tsx registry</span>
        </Link>

        <nav className="flex items-center gap-6 text-sm">
          <Link to="/browse" className="nav-link" activeProps={{ className: "nav-link is-active" }}>
            Browse
          </Link>
          <Link to="/dashboard" className="nav-link" activeProps={{ className: "nav-link is-active" }}>
            Dashboard
          </Link>
          <a
            href="https://github.com/ateeq1999/tsx"
            target="_blank"
            rel="noreferrer"
            className="nav-link"
          >
            GitHub
          </a>
        </nav>

        <div className="flex items-center gap-2">
          <ThemeToggle />
        </div>
      </div>
    </header>
  )
}
