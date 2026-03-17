import { Link } from "@tanstack/react-router"
import { ThemeToggle } from "./ThemeToggle"
import { DocSearch, DocSearchTrigger } from "./DocSearch"
import { BookOpen } from "lucide-react"

export function Header() {
  return (
    <header
      style={{ background: "var(--header-bg)", borderBottom: "1px solid var(--line)" }}
      className="sticky top-0 z-50 backdrop-blur-md"
    >
      <div className="page-wrap flex h-14 items-center justify-between gap-4">
        <Link to="/" className="flex items-center gap-2 font-bold" style={{ color: "var(--sea-ink)" }}>
          <BookOpen className="size-5" style={{ color: "var(--lagoon)" }} />
          <span>tsx docs</span>
        </Link>

        <nav className="flex items-center gap-6 text-sm">
          <Link to="/docs/getting-started" className="nav-link" activeProps={{ className: "nav-link is-active" }}>
            Getting Started
          </Link>
          <Link to="/docs/cli" className="nav-link" activeProps={{ className: "nav-link is-active" }}>
            CLI
          </Link>
          <Link to="/docs/registry" className="nav-link" activeProps={{ className: "nav-link is-active" }}>
            Registry
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

        <DocSearchTrigger />
        <ThemeToggle />
      </div>
    </header>
    <DocSearch />
  )
}
