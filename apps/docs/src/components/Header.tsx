import { Link } from "@tanstack/react-router"
import { BaseHeader } from "@tsx/ui/base-header"
import { ThemeToggle } from "./ThemeToggle"
import { DocSearch, DocSearchTrigger } from "./DocSearch"
import { BookOpen } from "lucide-react"

export function Header() {
  return (
    <>
      <BaseHeader
        logo={
          <Link to="/" className="flex items-center gap-2 font-bold" style={{ color: "var(--sea-ink)" }}>
            <BookOpen className="size-5" style={{ color: "var(--lagoon)" }} />
            <span>tsx docs</span>
          </Link>
        }
        nav={
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
        }
        right={
          <>
            <DocSearchTrigger />
            <ThemeToggle />
          </>
        }
      />
      <DocSearch />
    </>
  )
}
