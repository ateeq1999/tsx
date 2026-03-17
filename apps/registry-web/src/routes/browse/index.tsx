import { Link, createFileRoute, useNavigate } from "@tanstack/react-router"
import { useEffect, useRef, useState } from "react"
import { Search, SlidersHorizontal, X } from "lucide-react"
import { z } from "zod"
import { packagesQueryOptions, usePackages } from "@/features/packages/hooks/use-packages"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"

const LANGS = ["typescript", "python", "rust", "go"] as const
const SORTS = [
  { value: "relevant", label: "Relevance" },
  { value: "downloads", label: "Most downloaded" },
  { value: "newest", label: "Newest" },
  { value: "name", label: "Name A–Z" },
] as const

const searchSchema = z.object({
  q: z.string().default(""),
  page: z.number().int().min(1).default(1),
  lang: z.string().default(""),
  sort: z.string().default("relevant"),
})

export const Route = createFileRoute("/browse/")({
  validateSearch: searchSchema,
  loaderDeps: ({ search }) => search,
  loader: ({ context: { queryClient }, deps: { q, page } }) =>
    queryClient.prefetchQuery(packagesQueryOptions(q, page)),
  component: BrowsePage,
  head: () => ({
    meta: [
      { title: "Browse packages — tsx registry" },
      { name: "description", content: "Search and discover reusable code patterns for TanStack Start projects." },
    ],
  }),
})

function LangBadge({ lang }: { lang: string }) {
  const colours: Record<string, string> = {
    typescript: "#3178c6",
    python: "#3776ab",
    rust: "#ce422b",
    go: "#00add8",
  }
  return (
    <span
      className="rounded px-1.5 py-0.5 text-[10px] font-bold uppercase text-white"
      style={{ background: colours[lang] ?? "var(--lagoon)" }}
    >
      {lang === "typescript" ? "TS" : lang.slice(0, 2).toUpperCase()}
    </span>
  )
}

function PackageCard({ pkg }: { pkg: { name: string; version: string; description: string; tags: string[]; download_count: number; license: string; lang?: string } }) {
  const isOfficial = pkg.name.startsWith("@tsx-pkg/")
  return (
    <Link
      to="/packages/$name"
      params={{ name: pkg.name }}
      className="island-shell flex flex-col rounded-xl p-4 hover:no-underline"
    >
      <div className="mb-1 flex items-start justify-between gap-2">
        <span className="min-w-0 truncate font-mono text-sm font-bold" style={{ color: "var(--sea-ink)" }}>
          {pkg.name}
        </span>
        <div className="flex shrink-0 items-center gap-1.5">
          {pkg.lang && <LangBadge lang={pkg.lang} />}
          {isOfficial && (
            <span className="rounded px-1.5 py-0.5 text-[10px] font-semibold" style={{ background: "var(--lagoon)", color: "#fff" }}>
              official
            </span>
          )}
          <span className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>v{pkg.version}</span>
        </div>
      </div>
      <p className="mb-3 flex-1 text-xs leading-relaxed line-clamp-2" style={{ color: "var(--sea-ink-soft)" }}>
        {pkg.description}
      </p>
      <div className="flex flex-wrap gap-1">
        {pkg.tags.slice(0, 3).map((tag) => (
          <span key={tag} className="pkg-tag">{tag}</span>
        ))}
      </div>
      <div className="mt-3 flex items-center gap-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
        <span>{pkg.download_count.toLocaleString()} installs</span>
        <span>{pkg.license}</span>
      </div>
    </Link>
  )
}

function BrowsePage() {
  const { q, page, lang, sort } = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const { data, isLoading } = usePackages(q, page)
  const searchRef = useRef<HTMLInputElement>(null)
  const [showFilters, setShowFilters] = useState(!!(lang || sort !== "relevant"))

  // keyboard shortcut: / to focus search
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "/" && document.activeElement?.tagName !== "INPUT" && document.activeElement?.tagName !== "TEXTAREA") {
        e.preventDefault()
        searchRef.current?.focus()
      }
    }
    window.addEventListener("keydown", onKey)
    return () => window.removeEventListener("keydown", onKey)
  }, [])

  function setSearch(updates: Partial<z.infer<typeof searchSchema>>) {
    navigate({ search: (prev) => ({ ...prev, ...updates, page: "page" in updates ? updates.page ?? 1 : 1 }) })
  }

  // client-side sort (API returns by relevance/recent)
  const packages = (() => {
    const pkgs = data?.packages ?? []
    const filtered = lang ? pkgs.filter((p: { lang?: string }) => p.lang === lang) : pkgs
    switch (sort) {
      case "downloads": return [...filtered].sort((a, b) => b.download_count - a.download_count)
      case "newest": return [...filtered].sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
      case "name": return [...filtered].sort((a, b) => a.name.localeCompare(b.name))
      default: return filtered
    }
  })()

  const totalPages = data ? Math.max(1, Math.ceil(data.total / (data.per_page || 20))) : 1

  return (
    <div className="page-wrap py-12 rise-in">
      <h1 className="mb-2 text-3xl font-bold" style={{ color: "var(--sea-ink)" }}>Browse</h1>
      <p className="mb-8 text-sm" style={{ color: "var(--sea-ink-soft)" }}>
        {data ? `${data.total.toLocaleString()} packages available` : "Loading…"}
      </p>

      {/* Search + filter toggle */}
      <div className="mb-4 flex gap-2">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 size-4 -translate-y-1/2 pointer-events-none" style={{ color: "var(--sea-ink-soft)" }} />
          <Input
            ref={searchRef}
            className="pl-9 pr-16"
            placeholder="Search packages…"
            value={q}
            onChange={(e) => setSearch({ q: e.target.value })}
          />
          {q && (
            <button
              onClick={() => setSearch({ q: "" })}
              className="absolute right-3 top-1/2 -translate-y-1/2 opacity-60 hover:opacity-100"
            >
              <X className="size-4" style={{ color: "var(--sea-ink-soft)" }} />
            </button>
          )}
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => setShowFilters((v) => !v)}
          className="gap-1.5"
        >
          <SlidersHorizontal className="size-4" />
          <span className="hidden sm:inline">Filters</span>
          {(lang || sort !== "relevant") && (
            <Badge className="ml-1 size-4 p-0 flex items-center justify-center text-[10px]">!</Badge>
          )}
        </Button>
      </div>

      {/* Filter panel */}
      {showFilters && (
        <div className="island-shell mb-6 rounded-xl p-4">
          <div className="flex flex-wrap gap-6">
            <div>
              <p className="island-kicker mb-2">Language</p>
              <div className="flex flex-wrap gap-1.5">
                <button
                  onClick={() => setSearch({ lang: "" })}
                  className={`rounded-md px-3 py-1 text-xs font-medium transition-colors ${!lang ? "text-white" : "hover:opacity-80"}`}
                  style={{ background: !lang ? "var(--lagoon)" : "var(--line)", color: !lang ? "#fff" : "var(--sea-ink)" }}
                >
                  All
                </button>
                {LANGS.map((l) => (
                  <button
                    key={l}
                    onClick={() => setSearch({ lang: lang === l ? "" : l })}
                    className="rounded-md px-3 py-1 text-xs font-medium transition-colors hover:opacity-80"
                    style={{ background: lang === l ? "var(--lagoon)" : "var(--line)", color: lang === l ? "#fff" : "var(--sea-ink)" }}
                  >
                    {l}
                  </button>
                ))}
              </div>
            </div>
            <div>
              <p className="island-kicker mb-2">Sort by</p>
              <div className="flex flex-wrap gap-1.5">
                {SORTS.map(({ value, label }) => (
                  <button
                    key={value}
                    onClick={() => setSearch({ sort: value })}
                    className="rounded-md px-3 py-1 text-xs font-medium transition-colors hover:opacity-80"
                    style={{ background: sort === value ? "var(--lagoon)" : "var(--line)", color: sort === value ? "#fff" : "var(--sea-ink)" }}
                  >
                    {label}
                  </button>
                ))}
              </div>
            </div>
          </div>
          {(lang || sort !== "relevant") && (
            <button
              onClick={() => setSearch({ lang: "", sort: "relevant" })}
              className="mt-3 text-xs hover:underline"
              style={{ color: "var(--sea-ink-soft)" }}
            >
              Clear filters
            </button>
          )}
        </div>
      )}

      {/* Results */}
      {isLoading ? (
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 9 }).map((_, i) => (
            <div key={i} className="island-shell h-32 animate-pulse rounded-xl" />
          ))}
        </div>
      ) : packages.length === 0 ? (
        <div className="py-16 text-center">
          <p className="mb-2 text-4xl">📦</p>
          <p className="font-semibold" style={{ color: "var(--sea-ink)" }}>No packages found</p>
          {q ? (
            <p className="mt-1 text-sm" style={{ color: "var(--sea-ink-soft)" }}>
              No results for &ldquo;{q}&rdquo;.{" "}
              <button className="hover:underline" style={{ color: "var(--lagoon-deep)" }} onClick={() => setSearch({ q: "" })}>Clear search</button>
            </p>
          ) : (
            <p className="mt-1 text-sm" style={{ color: "var(--sea-ink-soft)" }}>Try adjusting your filters.</p>
          )}
        </div>
      ) : (
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {packages.map((pkg) => (
            <PackageCard key={pkg.name} pkg={pkg} />
          ))}
        </div>
      )}

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="mt-10 flex items-center justify-center gap-2">
          <Button
            variant="outline"
            size="sm"
            disabled={page <= 1}
            onClick={() => setSearch({ page: page - 1 })}
          >
            ← Prev
          </Button>
          <span className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>
            Page {page} of {totalPages}
          </span>
          <Button
            variant="outline"
            size="sm"
            disabled={page >= totalPages}
            onClick={() => setSearch({ page: page + 1 })}
          >
            Next →
          </Button>
        </div>
      )}

      <p className="mt-6 text-center text-xs" style={{ color: "var(--sea-ink-soft)" }}>
        Press <kbd className="rounded px-1 py-0.5 font-mono text-[10px]" style={{ background: "var(--line)" }}>/</kbd> to focus search
      </p>
    </div>
  )
}
