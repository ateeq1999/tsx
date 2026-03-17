import { useEffect, useRef, useState } from "react"
import { useNavigate } from "@tanstack/react-router"
import Fuse from "fuse.js"
import { Search, FileText, X } from "lucide-react"

interface DocEntry {
  title: string
  path: string
  section: string
  keywords: string
}

const DOC_INDEX: DocEntry[] = [
  { title: "Getting Started", path: "/docs/getting-started", section: "Introduction", keywords: "install setup first steps quickstart" },
  { title: "Installation", path: "/docs/installation", section: "Introduction", keywords: "cargo install binary windows macos linux completions shell" },
  { title: "CLI Overview", path: "/docs/cli", section: "CLI", keywords: "commands registry framework stack overview reference" },
  { title: "tsx install", path: "/docs/cli/install", section: "CLI", keywords: "registry install package force dir offline version pinning" },
  { title: "tsx search", path: "/docs/cli/search", section: "CLI", keywords: "registry search query lang json results" },
  { title: "tsx info", path: "/docs/cli/info", section: "CLI", keywords: "registry info metadata provides integrates json" },
  { title: "tsx framework", path: "/docs/cli/framework", section: "CLI", keywords: "framework init validate preview publish authoring" },
  { title: "tsx stack", path: "/docs/cli/stack", section: "CLI", keywords: "stack init show add remove detect apply aliases" },
  { title: "FPF Format", path: "/docs/fpf", section: "Framework Packages", keywords: "framework package format directory layout generators templates" },
  { title: "stack.json manifest", path: "/docs/fpf/manifest", section: "Framework Packages", keywords: "manifest stack json fields provides integrates generators style paths id name version" },
  { title: "Publishing", path: "/docs/fpf/publishing", section: "Framework Packages", keywords: "publish upload api key tarball validate preview" },
  { title: "Registry Overview", path: "/docs/registry", section: "Registry", keywords: "registry self host API endpoints overview" },
  { title: "Self-hosting", path: "/docs/registry/self-hosting", section: "Registry", keywords: "docker fly io sqlite data dir env vars port binary deploy" },
  { title: "API Reference", path: "/docs/registry/api", section: "Registry", keywords: "REST API endpoints search packages versions tarball publish health stats" },
  { title: "Examples", path: "/docs/examples", section: "Resources", keywords: "examples crud auth shadcn saas tanstack drizzle" },
  { title: "First-party Packages", path: "/docs/packages", section: "Resources", keywords: "official packages with-auth tanstack-start drizzle shadcn crud saas provides tokens" },
  { title: "Troubleshooting", path: "/docs/troubleshooting", section: "Resources", keywords: "error fix problem install fails timeout not found permission path" },
]

const fuse = new Fuse(DOC_INDEX, {
  keys: ["title", "keywords", "section"],
  threshold: 0.35,
  includeScore: true,
})

export function DocSearch() {
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState("")
  const inputRef = useRef<HTMLInputElement>(null)
  const navigate = useNavigate()

  const results = query.trim()
    ? fuse.search(query).slice(0, 8).map((r) => r.item)
    : DOC_INDEX.slice(0, 6)

  // Cmd+K / Ctrl+K to open
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault()
        setOpen((v) => !v)
      }
      if (e.key === "Escape") setOpen(false)
    }
    window.addEventListener("keydown", onKey)
    return () => window.removeEventListener("keydown", onKey)
  }, [])

  useEffect(() => {
    if (open) {
      setQuery("")
      setTimeout(() => inputRef.current?.focus(), 50)
    }
  }, [open])

  function go(path: string) {
    setOpen(false)
    navigate({ to: path as "/docs" })
  }

  if (!open) return null

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-24"
      style={{ background: "rgba(0,0,0,0.5)" }}
      onClick={() => setOpen(false)}
    >
      <div
        className="w-full max-w-lg rounded-2xl shadow-2xl overflow-hidden"
        style={{ background: "var(--surface-strong, #fff)" }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Input */}
        <div className="flex items-center gap-3 border-b px-4 py-3" style={{ borderColor: "var(--line)" }}>
          <Search className="size-4 shrink-0" style={{ color: "var(--sea-ink-soft)" }} />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search docs…"
            className="flex-1 bg-transparent text-sm outline-none"
            style={{ color: "var(--sea-ink)" }}
          />
          {query && (
            <button onClick={() => setQuery("")}>
              <X className="size-4" style={{ color: "var(--sea-ink-soft)" }} />
            </button>
          )}
          <kbd
            className="hidden rounded px-1.5 py-0.5 text-[10px] sm:block"
            style={{ background: "var(--line)", color: "var(--sea-ink-soft)" }}
          >
            Esc
          </kbd>
        </div>

        {/* Results */}
        <div className="max-h-72 overflow-y-auto py-2">
          {results.length === 0 ? (
            <p className="px-4 py-6 text-center text-sm" style={{ color: "var(--sea-ink-soft)" }}>
              No results for &ldquo;{query}&rdquo;
            </p>
          ) : (
            results.map((item) => (
              <button
                key={item.path}
                onClick={() => go(item.path)}
                className="flex w-full items-center gap-3 px-4 py-2.5 text-left transition-colors hover:no-underline"
                style={{ background: "transparent" }}
                onMouseEnter={(e) => (e.currentTarget.style.background = "var(--line)")}
                onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
              >
                <FileText className="size-4 shrink-0" style={{ color: "var(--lagoon)" }} />
                <div>
                  <p className="text-sm font-medium" style={{ color: "var(--sea-ink)" }}>{item.title}</p>
                  <p className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>{item.section}</p>
                </div>
              </button>
            ))
          )}
        </div>

        <div className="border-t px-4 py-2 text-xs" style={{ borderColor: "var(--line)", color: "var(--sea-ink-soft)" }}>
          {query ? `${results.length} result${results.length !== 1 ? "s" : ""}` : "Recent pages"} · Press Enter to navigate
        </div>
      </div>
    </div>
  )
}

/** Button that opens the search modal */
export function DocSearchTrigger() {
  return (
    <button
      onClick={() => window.dispatchEvent(new KeyboardEvent("keydown", { key: "k", metaKey: true, bubbles: true }))}
      className="flex items-center gap-2 rounded-lg border px-3 py-1.5 text-sm transition-colors hover:opacity-80"
      style={{ borderColor: "var(--line)", color: "var(--sea-ink-soft)" }}
    >
      <Search className="size-3.5" />
      <span className="hidden sm:inline">Search</span>
      <kbd className="hidden rounded px-1 py-0.5 text-[10px] sm:block" style={{ background: "var(--line)" }}>
        ⌘K
      </kbd>
    </button>
  )
}
