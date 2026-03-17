import { createFileRoute, Link, Outlet, useLocation } from "@tanstack/react-router"
import { useEffect, useRef, useState } from "react"
import hljs from "highlight.js/lib/core"
import typescript from "highlight.js/lib/languages/typescript"
import javascript from "highlight.js/lib/languages/javascript"
import bash from "highlight.js/lib/languages/bash"
import json from "highlight.js/lib/languages/json"
import rust from "highlight.js/lib/languages/rust"
import toml from "highlight.js/lib/languages/ini"
import { Menu, X, ChevronLeft, ChevronRight, ExternalLink } from "lucide-react"

hljs.registerLanguage("typescript", typescript)
hljs.registerLanguage("javascript", javascript)
hljs.registerLanguage("bash", bash)
hljs.registerLanguage("json", json)
hljs.registerLanguage("rust", rust)
hljs.registerLanguage("toml", toml)

export const Route = createFileRoute("/docs")({
  component: DocsLayout,
})

const sidebar = [
  {
    group: "Introduction",
    links: [
      { to: "/docs/getting-started", label: "Getting Started", file: "docs/getting-started.tsx" },
      { to: "/docs/installation", label: "Installation", file: "docs/installation.tsx" },
    ],
  },
  {
    group: "CLI",
    links: [
      { to: "/docs/cli", label: "Overview", file: "docs/cli.tsx" },
      { to: "/docs/cli/install", label: "tsx install", file: "docs/cli/install.tsx" },
      { to: "/docs/cli/search", label: "tsx search", file: "docs/cli/search.tsx" },
      { to: "/docs/cli/info", label: "tsx info", file: "docs/cli/info.tsx" },
      { to: "/docs/cli/framework", label: "tsx framework", file: "docs/cli/framework.tsx" },
      { to: "/docs/cli/stack", label: "tsx stack", file: "docs/cli/stack.tsx" },
    ],
  },
  {
    group: "Framework Packages",
    links: [
      { to: "/docs/fpf", label: "FPF Format", file: "docs/fpf.tsx" },
      { to: "/docs/fpf/manifest", label: "stack.json", file: "docs/fpf/manifest.tsx" },
      { to: "/docs/fpf/publishing", label: "Publishing", file: "docs/fpf/publishing.tsx" },
    ],
  },
  {
    group: "Registry",
    links: [
      { to: "/docs/registry", label: "Overview", file: "docs/registry.tsx" },
      { to: "/docs/registry/self-hosting", label: "Self-hosting", file: "docs/registry/self-hosting.tsx" },
      { to: "/docs/registry/api", label: "API Reference", file: "docs/registry/api.tsx" },
    ],
  },
  {
    group: "Resources",
    links: [
      { to: "/docs/examples", label: "Examples", file: "docs/examples.tsx" },
      { to: "/docs/packages", label: "Official Packages", file: "docs/packages.tsx" },
      { to: "/docs/troubleshooting", label: "Troubleshooting", file: "docs/troubleshooting.tsx" },
    ],
  },
]

const allLinks = sidebar.flatMap((s) => s.links)
const GITHUB_BASE = "https://github.com/ateeq1999/tsx/blob/main/apps/docs/src/routes"

function usePrevNext(pathname: string) {
  const idx = allLinks.findIndex((l) => l.to === pathname)
  return {
    prev: idx > 0 ? allLinks[idx - 1] : null,
    next: idx < allLinks.length - 1 ? allLinks[idx + 1] : null,
  }
}

function useBreadcrumb(pathname: string) {
  for (const section of sidebar) {
    const link = section.links.find((l) => l.to === pathname)
    if (link) return { group: section.group, label: link.label, file: link.file }
  }
  return null
}

interface TocEntry { id: string; text: string; level: number }

function SidebarNav({ onLinkClick }: { onLinkClick?: () => void }) {
  return (
    <nav className="space-y-6">
      {sidebar.map((section) => (
        <div key={section.group}>
          <p className="island-kicker mb-2">{section.group}</p>
          <ul className="space-y-1">
            {section.links.map((link) => (
              <li key={link.to}>
                <Link
                  to={link.to}
                  onClick={onLinkClick}
                  className="nav-link block py-1 text-sm"
                  activeProps={{ className: "nav-link is-active block py-1 text-sm font-semibold" }}
                >
                  {link.label}
                </Link>
              </li>
            ))}
          </ul>
        </div>
      ))}
    </nav>
  )
}

function DocsLayout() {
  const location = useLocation()
  const articleRef = useRef<HTMLElement>(null)
  const [mobileOpen, setMobileOpen] = useState(false)
  const [toc, setToc] = useState<TocEntry[]>([])
  const [activeId, setActiveId] = useState("")
  const { prev, next } = usePrevNext(location.pathname)
  const crumb = useBreadcrumb(location.pathname)

  useEffect(() => {
    const el = articleRef.current
    if (!el) return

    // Syntax highlight + copy buttons
    el.querySelectorAll("pre code").forEach((block) => {
      if (block.getAttribute("data-highlighted")) return
      hljs.highlightElement(block as HTMLElement)
      const pre = block.parentElement!
      if (pre.style.position !== "relative") pre.style.position = "relative"
      const btn = document.createElement("button")
      btn.textContent = "copy"
      btn.className = "hljs-copy-btn"
      btn.addEventListener("click", () => {
        navigator.clipboard.writeText((block as HTMLElement).innerText)
        btn.textContent = "✓"
        setTimeout(() => { btn.textContent = "copy" }, 2000)
      })
      pre.appendChild(btn)
    })

    // Build ToC from headings
    const headings = Array.from(el.querySelectorAll("h2, h3")) as HTMLElement[]
    headings.forEach((h, i) => {
      if (!h.id) h.id = `heading-${i}`
    })
    setToc(headings.map((h) => ({
      id: h.id,
      text: h.innerText,
      level: parseInt(h.tagName[1]),
    })))

    // Active heading on scroll
    const obs = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (e.isIntersecting) setActiveId(e.target.id)
        }
      },
      { rootMargin: "-80px 0px -60% 0px" }
    )
    headings.forEach((h) => obs.observe(h))

    setMobileOpen(false)
    return () => obs.disconnect()
  }, [location.pathname])

  return (
    <div className="page-wrap py-10">
      {/* Mobile toolbar */}
      <div className="mb-6 flex items-center gap-3 lg:hidden">
        <button
          onClick={() => setMobileOpen(true)}
          className="flex items-center gap-2 rounded-lg border px-3 py-1.5 text-sm"
          style={{ borderColor: "var(--line)", color: "var(--sea-ink-soft)" }}
        >
          <Menu className="size-4" /> Menu
        </button>
        {crumb && (
          <span className="text-sm truncate" style={{ color: "var(--sea-ink-soft)" }}>
            {crumb.group} › {crumb.label}
          </span>
        )}
      </div>

      {/* Mobile overlay */}
      {mobileOpen && (
        <div
          className="fixed inset-0 z-40 lg:hidden"
          onClick={() => setMobileOpen(false)}
          style={{ background: "rgba(0,0,0,0.4)" }}
        />
      )}
      <div
        className={`fixed inset-y-0 left-0 z-50 w-64 overflow-y-auto p-6 shadow-xl transition-transform duration-200 lg:hidden ${mobileOpen ? "translate-x-0" : "-translate-x-full"}`}
        style={{ background: "var(--surface-strong, #fff)" }}
      >
        <div className="mb-4 flex items-center justify-between">
          <span className="font-semibold text-sm" style={{ color: "var(--sea-ink)" }}>Documentation</span>
          <button onClick={() => setMobileOpen(false)}>
            <X className="size-5" style={{ color: "var(--sea-ink-soft)" }} />
          </button>
        </div>
        <SidebarNav onLinkClick={() => setMobileOpen(false)} />
      </div>

      <div className="flex gap-10">
        {/* Desktop sidebar */}
        <aside className="hidden w-52 shrink-0 lg:block">
          <div className="sticky top-20">
            <SidebarNav />
          </div>
        </aside>

        {/* Content */}
        <div className="min-w-0 flex-1">
          {/* Breadcrumb + Edit on GitHub */}
          {crumb && (
            <div className="mb-4 hidden items-center justify-between lg:flex">
              <p className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                Docs › {crumb.group} › {crumb.label}
              </p>
              <a
                href={`${GITHUB_BASE}/${crumb.file}`}
                target="_blank"
                rel="noreferrer"
                className="flex items-center gap-1 text-xs hover:underline"
                style={{ color: "var(--sea-ink-soft)" }}
              >
                Edit on GitHub <ExternalLink className="size-3" />
              </a>
            </div>
          )}

          <article ref={articleRef} className="doc-content">
            <Outlet />
          </article>

          {/* Prev / Next */}
          {(prev || next) && (
            <div className="mt-12 flex items-center justify-between gap-4 border-t pt-6" style={{ borderColor: "var(--line)" }}>
              {prev ? (
                <Link
                  to={prev.to}
                  className="group flex items-center gap-2 rounded-lg border px-4 py-3 text-sm transition-colors hover:no-underline"
                  style={{ borderColor: "var(--line)", color: "var(--sea-ink)" }}
                >
                  <ChevronLeft className="size-4 transition-transform group-hover:-translate-x-0.5" style={{ color: "var(--lagoon)" }} />
                  <div>
                    <p className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>Previous</p>
                    <p className="font-medium">{prev.label}</p>
                  </div>
                </Link>
              ) : <div />}
              {next ? (
                <Link
                  to={next.to}
                  className="group flex items-center gap-2 rounded-lg border px-4 py-3 text-sm transition-colors hover:no-underline text-right"
                  style={{ borderColor: "var(--line)", color: "var(--sea-ink)" }}
                >
                  <div>
                    <p className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>Next</p>
                    <p className="font-medium">{next.label}</p>
                  </div>
                  <ChevronRight className="size-4 transition-transform group-hover:translate-x-0.5" style={{ color: "var(--lagoon)" }} />
                </Link>
              ) : <div />}
            </div>
          )}
        </div>

        {/* Sticky ToC */}
        {toc.length > 1 && (
          <aside className="hidden w-44 shrink-0 xl:block">
            <div className="sticky top-20">
              <p className="island-kicker mb-3">On this page</p>
              <nav className="space-y-1">
                {toc.map((entry) => (
                  <a
                    key={entry.id}
                    href={`#${entry.id}`}
                    className="block truncate text-xs leading-5 transition-colors hover:no-underline"
                    style={{
                      paddingLeft: entry.level === 3 ? "0.75rem" : "0",
                      color: activeId === entry.id ? "var(--lagoon-deep)" : "var(--sea-ink-soft)",
                      fontWeight: activeId === entry.id ? "600" : "400",
                    }}
                  >
                    {entry.text}
                  </a>
                ))}
              </nav>
            </div>
          </aside>
        )}
      </div>
    </div>
  )
}
