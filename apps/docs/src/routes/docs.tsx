import { createFileRoute, Link, Outlet, useLocation } from "@tanstack/react-router"
import { useEffect, useRef, useState } from "react"
import hljs from "highlight.js/lib/core"
import typescript from "highlight.js/lib/languages/typescript"
import javascript from "highlight.js/lib/languages/javascript"
import bash from "highlight.js/lib/languages/bash"
import json from "highlight.js/lib/languages/json"
import rust from "highlight.js/lib/languages/rust"
import toml from "highlight.js/lib/languages/ini"
import { Menu, X, ChevronLeft, ChevronRight } from "lucide-react"

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
      { to: "/docs/getting-started", label: "Getting Started" },
      { to: "/docs/installation", label: "Installation" },
    ],
  },
  {
    group: "CLI",
    links: [
      { to: "/docs/cli", label: "Overview" },
      { to: "/docs/cli/install", label: "tsx install" },
      { to: "/docs/cli/search", label: "tsx search" },
      { to: "/docs/cli/info", label: "tsx info" },
      { to: "/docs/cli/framework", label: "tsx framework" },
      { to: "/docs/cli/stack", label: "tsx stack" },
    ],
  },
  {
    group: "Framework Packages",
    links: [
      { to: "/docs/fpf", label: "FPF Format" },
      { to: "/docs/fpf/manifest", label: "stack.json" },
      { to: "/docs/fpf/publishing", label: "Publishing" },
    ],
  },
  {
    group: "Registry",
    links: [
      { to: "/docs/registry", label: "Overview" },
      { to: "/docs/registry/self-hosting", label: "Self-hosting" },
      { to: "/docs/registry/api", label: "API Reference" },
    ],
  },
]

const allLinks = sidebar.flatMap((s) => s.links)

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
    if (link) return [section.group, link.label]
  }
  return []
}

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
  const { prev, next } = usePrevNext(location.pathname)
  const breadcrumb = useBreadcrumb(location.pathname)

  // Syntax highlighting + copy buttons on every route change
  useEffect(() => {
    const el = articleRef.current
    if (!el) return

    el.querySelectorAll("pre code").forEach((block) => {
      if (block.getAttribute("data-highlighted")) return
      hljs.highlightElement(block as HTMLElement)

      const pre = block.parentElement!
      if (pre.style.position !== "relative") {
        pre.style.position = "relative"
      }

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

    // close mobile sidebar on route change
    setMobileOpen(false)
  }, [location.pathname])

  return (
    <div className="page-wrap py-10">
      {/* Mobile sidebar toggle */}
      <div className="mb-6 flex items-center gap-3 lg:hidden">
        <button
          onClick={() => setMobileOpen(true)}
          className="flex items-center gap-2 rounded-lg border px-3 py-1.5 text-sm"
          style={{ borderColor: "var(--line)", color: "var(--sea-ink-soft)" }}
        >
          <Menu className="size-4" /> Menu
        </button>
        {breadcrumb.length > 0 && (
          <span className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>
            {breadcrumb.join(" › ")}
          </span>
        )}
      </div>

      {/* Mobile sidebar overlay */}
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
          {/* Breadcrumb (desktop) */}
          {breadcrumb.length > 1 && (
            <p className="mb-4 hidden text-xs lg:block" style={{ color: "var(--sea-ink-soft)" }}>
              Docs › {breadcrumb.join(" › ")}
            </p>
          )}

          <article ref={articleRef} className="doc-content">
            <Outlet />
          </article>

          {/* Prev / Next nav */}
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
      </div>
    </div>
  )
}
